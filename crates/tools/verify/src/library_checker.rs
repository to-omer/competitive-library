use crate::{app_cache_directory, BoxResult, ProblemNotFound, TestCase, VerifyStatus};
use serde::{Deserialize, Serialize};
use std::{
    env::consts::OS,
    error::Error,
    fmt::{self, Display},
    fs::{create_dir, read_dir, read_to_string, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

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

pub fn get_testcases_and_checker(problem_id: &str) -> BoxResult<(Vec<TestCase>, CheckerBinary)> {
    let rootdir = app_cache_directory().join("library-checker-problems");

    if rootdir.exists() {
        Command::new("git")
            .arg("-C")
            .arg(rootdir.as_os_str())
            .arg("pull")
            .output()?;
    } else {
        Command::new("git")
            .args([
                "clone",
                "https://github.com/yosupo06/library-checker-problems",
            ])
            .arg(rootdir.as_os_str())
            .output()?;
    }

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
                    name: name.clone(),
                    input: input.clone(),
                    output: output.clone(),
                });
            }
        }
    }

    Command::new(option_env!("PYTHON").unwrap_or("python"))
        .arg(rootdir.join("generate.py"))
        .arg(problem.problemdir.join("info.toml"))
        .output()?;

    let checker = problem
        .problemdir
        .join("checker")
        .with_extension(if OS != "windows" { "" } else { "exe" });
    if !checker.is_file() {
        Err(CheckerBinaryNotFound)?;
    }

    Ok((cases, CheckerBinary { checker }))
}

#[test]
fn test_aplusb() -> BoxResult<()> {
    let res = get_testcases_and_checker("aplusb")?;
    eprintln!("res = {:?}", res);
    Ok(())
}
