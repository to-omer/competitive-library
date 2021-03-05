use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::Duration,
};
use tempfile::NamedTempFile;
pub use verify_attr::verify;

lazy_static! {
    static ref OJ_API_RESOURCE: Mutex<()> = Mutex::new(());
}

#[derive(Clone, Debug, Deserialize)]
pub struct OjApiResponse<R> {
    pub status: String,
    pub messages: Vec<String>,
    pub result: R,
}

impl<R> OjApiResponse<R> {
    pub fn into_result(self) -> OjResult<R> {
        if self.status.as_str() == "ok" {
            Ok(self.result)
        } else {
            Err(OjError::StrError(self.messages.join("\n")))
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Problem {
    pub url: String,
    pub tests: Vec<TestCase>,
}

pub fn save_temp_file(buf: &[u8]) -> io::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(buf)?;
    Ok(file)
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub input: String,
    pub output: String,
}
impl TestCase {
    pub fn execute<'a, 'b>(&'a self, buf: &'b mut Vec<u8>, solve: fn(&'a [u8], &'b mut Vec<u8>)) {
        solve(self.input.as_bytes(), buf);
    }
    pub fn judge_with_env(&self, result: &[u8], env: &VerifyEnv) -> VerifyStatus {
        self.judge_with_env_inner(result, env)
            .unwrap_or(VerifyStatus::InternalError)
    }
    pub fn judge_with_env_inner(&self, result: &[u8], env: &VerifyEnv) -> OjResult<VerifyStatus> {
        match env {
            VerifyEnv::LibraryChecker(checker) => {
                let infile = save_temp_file(self.input.as_bytes())?;
                let outfile = save_temp_file(self.output.as_bytes())?;
                let resfile = save_temp_file(result)?;
                checker.check(infile.path(), outfile.path(), resfile.path())
            }
            VerifyEnv::AizuOnlineJudge => Ok((self.output.as_bytes() == result).into()),
        }
    }
    pub fn judge_with_judger<'a, 'b>(
        &'a self,
        result: &'b [u8],
        judger: fn(&'a [u8], &'a [u8], &'b [u8]) -> bool,
    ) -> VerifyStatus {
        judger(self.input.as_bytes(), self.output.as_bytes(), result).into()
    }
    #[allow(dead_code)]
    pub fn judge_with_eps(&self, result: &[u8], eps: f64) -> VerifyStatus {
        match String::from_utf8(result.to_vec()) {
            Ok(res) => {
                let (mut it_out, mut it_res) = (
                    self.output.split_ascii_whitespace(),
                    res.split_ascii_whitespace(),
                );
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
            Err(_) => VerifyStatus::WrongAnswer,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServiceResult {
    pub url: String,
    pub name: String,
}

impl ServiceResult {
    pub fn into_service(self) -> Option<Service> {
        Some(match self.name.as_str() {
            "Library Checker" => Service::LibraryChecker,
            "Aizu Online Judge" => Service::AizuOnlineJudge,
            _ => None?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Service {
    LibraryChecker,
    AizuOnlineJudge,
}
impl Display for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Service::LibraryChecker => write!(f, "Library Checker"),
            Service::AizuOnlineJudge => write!(f, "Aizu Online Judge"),
        }
    }
}

pub struct OjApi {}
impl OjApi {
    pub fn call_with_args<R: DeserializeOwned, I, S>(args: I) -> OjResult<R>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = {
            let _guard = OJ_API_RESOURCE.lock().unwrap();
            Command::new(option_env!("ONLINE_JUDGE_TOOLS_API").unwrap_or("oj-api"))
                .args(args)
                .output()?
        };
        if output.status.success() {
            let response: OjApiResponse<R> = serde_json::from_slice(&output.stdout)?;
            response.into_result()
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::info!("{}", stderr);
            Err(OjError::CommandError(stderr.to_string()))
        }
    }
    pub fn get_testcases(url: &str) -> OjResult<Problem> {
        Self::call_with_args(&["get-problem", "--system", url])
    }
    pub fn get_service(url: &str) -> OjResult<Service> {
        let service: ServiceResult = Self::call_with_args(&["get-service", url])?;
        service.into_service().ok_or(OjError::UnsupportedService)
    }
}

#[derive(Debug)]
pub struct CheckerBinary {
    checker: PathBuf,
}
impl CheckerBinary {
    pub fn from_url(url: &str) -> OjResult<Self> {
        let output = {
            let _guard = OJ_API_RESOURCE.lock().unwrap();
            Command::new("python")
                .args(&[
                    "-c",
                    format!(
                        r#"from onlinejudge import dispatch
problem = dispatch.problem_from_url('{}')
try: print(problem._get_problem_directory_path() / 'checker', end='')
except Exception as e: assert False
"#,
                        url
                    )
                    .as_str(),
                ])
                .output()?
        };
        if output.status.success() {
            let checker = PathBuf::from(String::from_utf8_lossy(&output.stdout).to_string());
            Ok(Self { checker })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::info!("{}", stderr);
            Err(OjError::CommandError(stderr.to_string()))
        }
    }
    pub fn check(&self, input: &Path, output: &Path, result: &Path) -> OjResult<VerifyStatus> {
        let output = Command::new(self.checker.as_os_str())
            .args(&[input.as_os_str(), result.as_os_str(), output.as_os_str()])
            .output()?;
        match output.status.code() {
            Some(0) => Ok(VerifyStatus::Accepted),
            Some(1) => Ok(VerifyStatus::WrongAnswer),
            Some(_) => Ok(VerifyStatus::InternalError),
            None => Err(OjError::StrError("checker broken".to_string())),
        }
    }
}

pub fn get_workspace_root() -> Option<PathBuf> {
    #[derive(Debug, Clone, Deserialize)]
    struct WorkspaceRoot {
        workspace_root: String,
    }
    let output = Command::new(env!("CARGO"))
        .args(&["metadata", "--quiet"])
        .output()
        .ok()?;
    if output.status.success() {
        if let Ok(root) = serde_json::from_slice::<WorkspaceRoot>(&output.stdout) {
            return Some(PathBuf::from(root.workspace_root));
        }
    }
    None
}

#[derive(Debug, thiserror::Error)]
pub enum OjError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("utf8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("error: {0}")]
    StrError(String),
    #[error("command error: {0}")]
    CommandError(String),
    #[error("error: unsupported service")]
    UnsupportedService,
    #[error("verify failed: {0}")]
    VerifyFailed(String),
}

pub type OjResult<T> = Result<T, OjError>;

#[derive(Clone, Debug)]
pub struct VerifyConfig<'t> {
    url: &'static str,
    cur_file: &'static str,
    fn_name: &'static str,
    target: &'t str,
    start: DateTime<Utc>,
}
impl<'t> VerifyConfig<'t> {
    pub fn new(
        url: &'static str,
        cur_file: &'static str,
        fn_name: &'static str,
        target: &'t str,
    ) -> Self {
        Self {
            url,
            cur_file,
            fn_name,
            target: strip_package(target),
            start: Utc::now(),
        }
    }
    pub fn gen_env(&self) -> OjResult<VerifyEnv> {
        let service = OjApi::get_service(self.url)?;
        log::info!("identify the service as `{}`", service);
        let env = match service {
            Service::LibraryChecker => {
                log::info!("download checker binary");
                VerifyEnv::LibraryChecker(CheckerBinary::from_url(self.url)?)
            }
            Service::AizuOnlineJudge => VerifyEnv::AizuOnlineJudge,
        };
        Ok(env)
    }
    pub fn get_testcases(&self) -> OjResult<Problem> {
        log::info!("download testcases: {}", self.url);
        let res = OjApi::get_testcases(self.url);
        match &res {
            Ok(problem) => log::info!("success to download {} testcases", problem.tests.len()),
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
            path
        };
        File::create(path)?.write_all(buf)
    }
    pub fn finalize(&self, result: OjResult<VerifyResults>) -> OjResult<()> {
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
                Err(OjError::VerifyFailed(self.fn_name.to_string()))
            }
        })
    }
    pub fn gen_md_contents(&self, result: &OjResult<VerifyResults>) -> String {
        let head = result
            .as_ref()
            .map(|r| {
                let badge = if r.is_ac() {
                    "✅"
                } else {
                    "❌" // ❎
                };
                format!("{}  {}  {}ms", badge, r.status(), r.elapsed().as_millis())
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
                            "| {} | {} | {} ms |\n",
                            r.name,
                            r.status,
                            r.elapsed.as_millis()
                        )
                        .as_str(),
                    );
                }
                buf
            })
            .unwrap_or_default();
        let tz = FixedOffset::east(9 * 3600);
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
            url = self.url,
            detail = detail,
            meta = meta
        )
    }
}

#[derive(Debug)]
pub enum VerifyEnv {
    LibraryChecker(CheckerBinary),
    AizuOnlineJudge,
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
    pub elapsed: Duration,
}

impl VerifyResult {
    pub fn new(name: String, status: VerifyStatus, elapsed: Duration) -> Self {
        Self {
            name,
            status,
            elapsed,
        }
    }
}

#[derive(Debug, Default)]
pub struct VerifyResults {
    pub results: Vec<VerifyResult>,
}

impl VerifyResults {
    pub fn new() -> Self {
        log::info!("verify start");
        Self {
            results: Vec::new(),
        }
    }
    pub fn push(&mut self, name: String, status: VerifyStatus, elapsed: Duration) {
        log::info!(" - {} {} {}ms", name, status, elapsed.as_millis());
        self.results.push(VerifyResult::new(name, status, elapsed))
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
    pub fn elapsed(&self) -> Duration {
        self.results
            .iter()
            .map(|res| res.elapsed)
            .max()
            .unwrap_or_else(|| Duration::from_secs(0))
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
