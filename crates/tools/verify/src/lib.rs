use self::library_checker::CheckerBinary;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use dirs::cache_dir;
use rand::prelude::*;
use reqwest::{blocking, Client};
use serde::Deserialize;
use std::{
    borrow::Borrow,
    collections::hash_map::RandomState,
    env::{current_dir, temp_dir},
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    hash::BuildHasher,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tempfile::NamedTempFile;
use tokio::{io::AsyncWriteExt, time::sleep};
pub use verify_attr::{aizu_online_judge, library_checker};

mod aizu_online_judge;
mod library_checker;

const APP_NAME: &str = "competitive-library";

fn save_temp_file(buf: &[u8]) -> io::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(buf)?;
    Ok(file)
}

fn get_workspace_root() -> Option<PathBuf> {
    #[derive(Debug, Clone, Deserialize)]
    struct WorkspaceRoot {
        workspace_root: String,
    }
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--quiet", "--no-deps"])
        .output()
        .ok()?;
    if output.status.success() {
        if let Ok(root) = serde_json::from_slice::<WorkspaceRoot>(&output.stdout) {
            return Some(PathBuf::from(root.workspace_root));
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::info!("{}", stderr);
    }
    None
}

fn app_cache_directory() -> PathBuf {
    let mut path = cache_dir().unwrap_or_else(temp_dir);
    path.push(APP_NAME);
    path
}

fn build_client() -> reqwest::Result<blocking::Client> {
    blocking::Client::builder().user_agent(APP_NAME).build()
}

fn build_async_client() -> reqwest::Result<Client> {
    Client::builder().user_agent(APP_NAME).build()
}

async fn gen_case(url: String, file: PathBuf) -> BoxResult<()> {
    async fn gen_case_inner(url: &str, file: &PathBuf) -> BoxResult<()> {
        use tokio::fs::File;
        let client = build_async_client()?;
        let bytes = client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        File::create(file).await?.write_all(bytes.borrow()).await?;
        Ok(())
    }
    let seed = RandomState::new().hash_one(&url);
    let mut rng = StdRng::seed_from_u64(seed);
    for _ in 0..2 {
        if let Ok(()) = gen_case_inner(&url, &file).await {
            return Ok(());
        };
        sleep(Duration::from_secs_f64(rng.gen_range(1f64..5f64))).await;
    }
    gen_case_inner(&url, &file).await
}

#[derive(Debug, Clone)]
pub enum Service {
    LibraryChecker,
    AizuOnlineJudge,
}

impl Service {
    pub fn url(&self, problem: &str) -> String {
        match self {
            Service::LibraryChecker => {
                format!("https://judge.yosupo.jp/problem/{}", problem)
            }
            Service::AizuOnlineJudge => {
                format!("https://onlinejudge.u-aizu.ac.jp/problems/{}", problem)
            }
        }
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Service::LibraryChecker => write!(f, "Library Checker"),
            Service::AizuOnlineJudge => write!(f, "Aizu Online Judge"),
        }
    }
}

pub type Checker = Option<CheckerBinary>;

type BoxResult<T> = Result<T, Box<dyn 'static + std::error::Error>>;

#[derive(Debug, Clone)]
pub struct Problem {
    pub service: Service,
    pub problem: String,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestCaseRef {
    pub case: TestCase,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct ProblemNotFound;

impl Error for ProblemNotFound {}
impl Display for ProblemNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("problem not found")
    }
}

#[derive(Debug, Clone, Copy)]
struct VerifyFailed;

impl Error for VerifyFailed {}
impl Display for VerifyFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("verify failed")
    }
}

impl TestCase {
    pub fn load_testcase(self) -> io::Result<TestCaseRef> {
        let mut input = vec![];
        let mut output = vec![];
        File::open(&self.input)?.read_to_end(&mut input)?;
        File::open(&self.output)?.read_to_end(&mut output)?;
        Ok(TestCaseRef {
            case: self,
            input,
            output,
        })
    }
}

impl TestCaseRef {
    pub fn judge_with_checker(&self, result: &[u8], checker: &Checker) -> VerifyStatus {
        self.judge_with_checker_inner(result, checker)
            .unwrap_or(VerifyStatus::InternalError)
    }
    fn judge_with_checker_inner(
        &self,
        result: &[u8],
        checker: &Checker,
    ) -> BoxResult<VerifyStatus> {
        match checker {
            Some(checker) => {
                let resfile = save_temp_file(result)?;
                checker.check(&self.case.input, &self.case.output, resfile.path())
            }
            None => Ok((self.output == result).into()),
        }
    }
    pub fn judge_with_judger<'a, 'b>(
        &'a self,
        result: &'b [u8],
        judger: fn(&'a [u8], &'a [u8], &'b [u8]) -> bool,
    ) -> VerifyStatus {
        judger(&self.input, &self.output, result).into()
    }
    #[allow(dead_code)]
    pub fn judge_with_eps(&self, result: &[u8], eps: f64) -> VerifyStatus {
        let out = String::from_utf8_lossy(&self.output);
        let res = String::from_utf8_lossy(result);
        let (mut it_out, mut it_res) = (out.split_ascii_whitespace(), res.split_ascii_whitespace());
        loop {
            return match (it_out.next(), it_res.next()) {
                (Some(x1), Some(x2)) => match (x1.parse::<f64>(), x2.parse::<f64>()) {
                    (Ok(x1), Ok(x2)) => {
                        if (x1 - x2).abs() > eps {
                            VerifyStatus::WrongAnswer
                        } else {
                            continue;
                        }
                    }
                    _ => VerifyStatus::WrongAnswer,
                },
                (None, None) => VerifyStatus::Accepted,
                _ => VerifyStatus::WrongAnswer,
            };
        }
    }
}

#[derive(Clone, Debug)]
pub struct VerifyConfig<'t> {
    service: Service,
    problem: &'static str,
    cur_file: &'static str,
    fn_name: &'static str,
    target: &'t str,
    start: DateTime<Utc>,
}

impl<'t> VerifyConfig<'t> {
    pub fn new(
        service: Service,
        problem: &'static str,
        cur_file: &'static str,
        fn_name: &'static str,
        target: &'t str,
    ) -> Self {
        Self {
            service,
            problem,
            cur_file,
            fn_name,
            target: strip_package(target),
            start: Utc::now(),
        }
    }
    pub fn get_testcases_and_checker(&self) -> BoxResult<(Vec<TestCase>, Checker)> {
        log::info!("download testcases: {} {}", self.service, self.problem);
        let start = Instant::now();
        let res = match self.service {
            Service::LibraryChecker => library_checker::get_testcases_and_checker(self.problem)
                .map(|(cases, checker)| (cases, Some(checker))),
            Service::AizuOnlineJudge => {
                aizu_online_judge::get_testcases(self.problem).map(|cases| (cases, None))
            }
        };
        match &res {
            Ok((cases, _)) => log::info!(
                "success to download {} testcases in {:.2}s",
                cases.len(),
                start.elapsed().as_secs_f64()
            ),
            Err(_) => log::info!("failed to download testcases"),
        }
        res
    }
    pub fn emit_md(&self, buf: &[u8]) -> io::Result<()> {
        let path = Path::new(self.cur_file)
            .with_file_name(self.fn_name)
            .with_extension("md");
        let path = if let Some(root) = get_workspace_root() {
            root.join(path)
        } else {
            current_dir()?.join(path)
        };
        log::info!("emit results to {}", path.display());
        File::create(path)?.write_all(buf)
    }
    pub fn finalize(&self, result: BoxResult<VerifyResults>) -> BoxResult<()> {
        if let Ok(results) = &result {
            let mut map = std::collections::BTreeMap::<_, usize>::new();
            for result in results.results.iter() {
                *map.entry(result.status).or_default() += 1;
            }
            let res = map
                .iter()
                .map(|(k, v)| format!("{} × {};", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            log::info!("{}", res);
        };
        self.emit_md(self.gen_md_contents(&result).as_bytes())?;
        result.and_then(|r| {
            if r.is_ac() {
                Ok(())
            } else {
                Err(VerifyFailed.into())
            }
        })
    }
    pub fn gen_md_contents(&self, result: &BoxResult<VerifyResults>) -> String {
        let head = result
            .as_ref()
            .map(|r| {
                let badge = if r.is_ac() {
                    "✅"
                } else {
                    "❌" // ❎
                };
                format!("{}  {}  {}ms", badge, r.status(), r.elapsed())
            })
            .unwrap_or_else(|_| "❌".to_string());
        let detail = result
            .as_ref()
            .map(|res| {
                let mut buf = String::from(
                    r###"# Detail

| Case Name | Status | Exec Time |
|:---------:|:------:|---------:|
"###,
                );
                for r in res.results.iter() {
                    buf.push_str(
                        format!(
                            "| {} | {} | {} ± {} ms |\n",
                            r.name, r.status, r.elapsed_mean, r.elapsed_var
                        )
                        .as_str(),
                    );
                }
                buf
            })
            .unwrap_or_default();
        let tz = FixedOffset::east_opt(9 * 3600).unwrap();
        let end = Utc::now();
        let meta = format!(
            r#"
VERIFY_TARGET: {}
VERIFY_START: {}
VERIFY_END: {}
"#,
            self.target,
            self.start
                .with_timezone(&tz)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            end.with_timezone(&tz)
                .to_rfc3339_opts(SecondsFormat::Millis, true)
        );
        format!(
            r###"{head}

problem [here]({url})

{detail}

<!-- {meta} -->
"###,
            head = head,
            url = self.service.url(self.problem),
            detail = detail,
            meta = meta
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerifyStatus {
    /// Accepted
    Accepted,
    // /// Time Limit Exceeded
    // TLE,
    /// Wrong Answer
    WrongAnswer,
    /// Runtime Error
    RuntimeError,
    /// Internal Error
    InternalError,
}
impl From<bool> for VerifyStatus {
    fn from(b: bool) -> Self {
        if b {
            Self::Accepted
        } else {
            Self::WrongAnswer
        }
    }
}
impl Display for VerifyStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accepted => write!(f, "AC"),
            Self::WrongAnswer => write!(f, "WA"),
            Self::RuntimeError => write!(f, "RE"),
            Self::InternalError => write!(f, "IE"),
        }
    }
}

#[derive(Debug)]
pub struct VerifyResult {
    pub name: String,
    pub status: VerifyStatus,
    pub elapsed_mean: u128,
    pub elapsed_var: u128,
}

#[derive(Debug, Default)]
pub struct VerifyResults {
    pub results: Vec<VerifyResult>,
}

fn mean_and_var(elapseds: Vec<Duration>) -> (f64, f64) {
    if elapseds.is_empty() {
        return (0., 0.);
    }
    let mut m1 = 0f64;
    let mut m2 = 0f64;
    for &elapsed in &elapseds {
        let e = elapsed.as_millis() as f64;
        m1 += e;
        m2 += e * e;
    }
    let n = elapseds.len() as f64;
    (m1 / n, (m2 / n - (m1 / n).powi(2)).sqrt())
}

impl VerifyResults {
    pub fn new() -> Self {
        log::info!("verify start");
        Self {
            results: Vec::new(),
        }
    }
    pub fn push(&mut self, name: String, status: VerifyStatus, elapseds: Vec<Duration>) {
        let (mean, var) = mean_and_var(elapseds);
        let mean = mean.round() as u128;
        let var = var.round() as u128;
        log::info!(" - {} {} {} ± {} ms", name, status, mean, var);
        self.results.push(VerifyResult {
            name,
            status,
            elapsed_mean: mean,
            elapsed_var: var,
        })
    }
    pub fn status(&self) -> VerifyStatus {
        self.results
            .iter()
            .map(|res| res.status)
            .max()
            .unwrap_or(VerifyStatus::Accepted)
    }
    pub fn is_ac(&self) -> bool {
        self.status() == VerifyStatus::Accepted
    }
    pub fn elapsed(&self) -> u128 {
        self.results
            .iter()
            .map(|res| res.elapsed_mean)
            .max()
            .unwrap_or_default()
    }
}

pub fn strip_package(target: &str) -> &str {
    if let Some(k) = target.find("::").map(|i| i + 2) {
        &target[k..]
    } else {
        target
    }
}

pub fn log_formatter(
    buf: &mut env_logger::fmt::Formatter,
    record: &log::Record,
    target: &str,
) -> io::Result<()> {
    let target = strip_package(target);
    writeln!(buf, "test {} ... {}", target, record.args())
}

pub fn init_logger(target: String) -> Result<(), log::SetLoggerError> {
    env_logger::builder()
        .format(move |buf, record| log_formatter(buf, record, &target))
        .is_test(true)
        .try_init()
}
