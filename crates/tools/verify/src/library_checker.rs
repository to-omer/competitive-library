use crate::{BoxResult, ProblemNotFound, TestCase, VerifyStatus, app_cache_directory};
use fd_lock::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    env::consts::OS,
    error::Error,
    fmt::{self, Display},
    fs::{File, create_dir, read_dir, read_to_string, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime},
};

const LIBRARY_CHECKER_PROBLEMS_REPO: &str = "library-checker-problems";
const LIBRARY_CHECKER_PROBLEMS_REPO_LOCK: &str = "library-checker-problems-file-lock";
const LIBRARY_CHECKER_PROBLEMS_REPO_URL: &str =
    "https://github.com/yosupo06/library-checker-problems";
const LIBRARY_CHECKER_PROBLEMS_PULL_STAMP: &str = "library-checker-problems-pull.stamp";
const LIBRARY_CHECKER_PROBLEMS_PULL_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Debug)]
struct LibraryCheckerProblem {
    problemdir: PathBuf,
    info: LibraryCheckerProblemInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryCheckerProblemInfo {
    tests: Vec<LibraryCheckerProblemInfoTestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryCheckerProblemInfoTestCase {
    name: String,
    number: usize,
}

#[derive(Debug)]
pub struct CheckerBinary {
    pub checker: PathBuf,
}

#[derive(Debug, Clone, Copy)]
struct CheckerBinaryBroken;

impl Error for CheckerBinaryBroken {}
impl Display for CheckerBinaryBroken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("checker binary broken")
    }
}

#[derive(Debug, Clone, Copy)]
struct CheckerBinaryNotFound;

impl Error for CheckerBinaryNotFound {}
impl Display for CheckerBinaryNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("checker binary not found")
    }
}

#[derive(Debug)]
struct LibraryCheckerProblemsPrepareFailed {
    status: Option<i32>,
    stderr: String,
}

impl Error for LibraryCheckerProblemsPrepareFailed {}
impl Display for LibraryCheckerProblemsPrepareFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Some(code) => write!(
                f,
                "library-checker-problems update failed with exit code {}",
                code
            )?,
            None => f.write_str("library-checker-problems update failed with signal")?,
        }
        if !self.stderr.is_empty() {
            write!(f, ": {}", self.stderr.trim_end())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TestcaseGenerationFailed {
    status: Option<i32>,
    stderr: String,
}

impl Error for TestcaseGenerationFailed {}
impl Display for TestcaseGenerationFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Some(code) => write!(f, "testcase generation failed with exit code {}", code)?,
            None => f.write_str("testcase generation failed with signal")?,
        }
        if !self.stderr.is_empty() {
            write!(f, ": {}", self.stderr.trim_end())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TestcaseFileNotFound {
    path: PathBuf,
}

impl Error for TestcaseFileNotFound {}
impl Display for TestcaseFileNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "testcase file not found: {}", self.path.display())
    }
}

impl CheckerBinary {
    pub fn check(
        &self,
        input: &Path,
        output: &Path,
        result: &Path,
    ) -> Result<VerifyStatus, Box<dyn 'static + std::error::Error>> {
        let output = Command::new(self.checker.as_os_str())
            .args([input.as_os_str(), result.as_os_str(), output.as_os_str()])
            .output()?;
        match output.status.code() {
            Some(0) => Ok(VerifyStatus::Accepted),
            Some(1) => Ok(VerifyStatus::WrongAnswer),
            Some(_) => Ok(VerifyStatus::InternalError),
            None => Err(CheckerBinaryBroken)?,
        }
    }
}

fn casename(name: &str, i: usize) -> Option<String> {
    let mut iter = name.rsplitn(2, '.');
    let after = iter.next();
    let before = iter.next();
    if before == Some("") {
        Some(name)
    } else {
        before.or(after)
    }
    .map(|name| format!("{}_{:02}", name, i))
}

fn find_problem(rootdir: &PathBuf, problem: &str) -> BoxResult<LibraryCheckerProblem> {
    for entry in read_dir(rootdir)?.flatten() {
        let mut path = entry.path();
        path.push(problem);
        path.push("info.toml");
        if path.is_file() {
            let data = read_to_string(&path)?;
            let info: LibraryCheckerProblemInfo = toml::from_str(&data)?;
            path.pop();
            return Ok(LibraryCheckerProblem {
                problemdir: path,
                info,
            });
        }
    }
    Err(ProblemNotFound)?
}

fn prepare_library_checker_problems() -> BoxResult<PathBuf> {
    let lock_file = File::create(app_cache_directory().join(LIBRARY_CHECKER_PROBLEMS_REPO_LOCK))?;
    let mut lock = RwLock::new(lock_file);
    let _lock_guard = lock.write()?;

    let rootdir = app_cache_directory().join(LIBRARY_CHECKER_PROBLEMS_REPO);
    let stamp_path = app_cache_directory().join(LIBRARY_CHECKER_PROBLEMS_PULL_STAMP);
    let last_prepared_at = stamp_path
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok());
    if rootdir.exists()
        && last_prepared_at.is_some_and(|last_prepared_at| {
            SystemTime::now()
                .duration_since(last_prepared_at)
                .unwrap_or_default()
                < LIBRARY_CHECKER_PROBLEMS_PULL_TTL
        })
    {
        return Ok(rootdir);
    }

    let output = if rootdir.exists() {
        Command::new("git")
            .arg("-C")
            .arg(rootdir.as_os_str())
            .arg("pull")
            .output()?
    } else {
        Command::new("git")
            .args(["clone", LIBRARY_CHECKER_PROBLEMS_REPO_URL])
            .arg(rootdir.as_os_str())
            .output()?
    };
    if !output.status.success() {
        Err(LibraryCheckerProblemsPrepareFailed {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })?;
    }
    File::create(stamp_path)?;
    Ok(rootdir)
}

pub fn get_testcases_and_checker(problem_id: &str) -> BoxResult<(Vec<TestCase>, CheckerBinary)> {
    let rootdir = prepare_library_checker_problems()?;

    let problem_lock_file = File::create(app_cache_directory().join(format!(
        "library-checker-problem-{}.lock",
        problem_id
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                _ => '_',
            })
            .collect::<String>()
    )))?;
    let mut problem_lock = RwLock::new(problem_lock_file);
    let _problem_lock_guard = problem_lock.write()?;

    let repo_lock_file =
        File::create(app_cache_directory().join(LIBRARY_CHECKER_PROBLEMS_REPO_LOCK))?;
    let repo_lock = RwLock::new(repo_lock_file);
    let _repo_lock_guard = repo_lock.read()?;

    let problem = find_problem(&rootdir, problem_id)?;

    let mut cases = vec![];
    let indir = problem.problemdir.join("in");
    let outdir = problem.problemdir.join("out");

    let is_testcases_already_generated = indir.exists() && outdir.exists();
    if !is_testcases_already_generated {
        if indir.exists() {
            remove_dir_all(&indir)?;
        }
        if outdir.exists() {
            remove_dir_all(&outdir)?;
        }
        create_dir(&indir)?;
        create_dir(&outdir)?;
    }

    for case in &problem.info.tests {
        for i in 0..case.number {
            if let Some(name) = casename(&case.name, i) {
                let input = indir.join(&name).with_extension("in");
                let output = outdir.join(&name).with_extension("out");
                cases.push(TestCase {
                    name,
                    input,
                    output,
                });
            }
        }
    }

    let output = Command::new(option_env!("PYTHON").unwrap_or("python3"))
        .arg(rootdir.join("generate.py"))
        .arg(problem.problemdir.join("info.toml"))
        .output()?;
    if !output.status.success() {
        log::error!(
            "Testcase generation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        // Debug
        for case in &cases {
            log::info!(
                "Input: {} ({} bytes), Output: {} ({} bytes)",
                case.input.display(),
                case.input.metadata().map(|m| m.len()).unwrap_or(0),
                case.output.display(),
                case.output.metadata().map(|m| m.len()).unwrap_or(0),
            );
        }

        Err(TestcaseGenerationFailed {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })?;
    }

    for case in &cases {
        if !case.input.is_file() {
            Err(TestcaseFileNotFound {
                path: case.input.clone(),
            })?;
        }
        if !case.output.is_file() {
            Err(TestcaseFileNotFound {
                path: case.output.clone(),
            })?;
        }
    }

    let checker = problem
        .problemdir
        .join("checker")
        .with_extension(if OS != "windows" { "" } else { "exe" });
    if !checker.is_file() {
        Err(CheckerBinaryNotFound)?;
    }

    Ok((cases, CheckerBinary { checker }))
}

pub fn get_problem_list() -> BoxResult<Vec<(String, Vec<String>)>> {
    let rootdir = prepare_library_checker_problems()?;
    let repo_lock_file =
        File::create(app_cache_directory().join(LIBRARY_CHECKER_PROBLEMS_REPO_LOCK))?;
    let repo_lock = RwLock::new(repo_lock_file);
    let _repo_lock_guard = repo_lock.read()?;
    // find . -name "info.toml" -not -path "./test/*"
    // ./category/problem/info.toml
    let mut problems = vec![];
    for entry in read_dir(&rootdir)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let category = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if category == "test" {
            continue;
        }
        let mut category_problems = vec![];
        for entry in read_dir(&path)?.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let problem = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let info_path = path.join("info.toml");
            if !info_path.is_file() {
                continue;
            }
            category_problems.push(problem);
        }
        if category_problems.is_empty() {
            continue;
        }
        problems.push((category, category_problems));
    }
    Ok(problems)
}

#[test]
fn test_aplusb() -> BoxResult<()> {
    let res = get_testcases_and_checker("aplusb")?;
    eprintln!("res = {:?}", res);
    Ok(())
}
