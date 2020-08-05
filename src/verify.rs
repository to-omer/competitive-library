use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    ffi::OsStr,
    fmt::{self, Formatter},
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::Duration,
};
use tempfile::NamedTempFile;

lazy_static! {
    static ref OJ_API_RESOURCE: Mutex<()> = Mutex::new(());
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct OjApiResponse<R> {
    pub(crate) status: String,
    pub(crate) messages: Vec<String>,
    pub(crate) result: R,
}

impl<R> OjApiResponse<R> {
    pub(crate) fn into_result(self) -> OjResult<R> {
        if self.status.as_str() == "ok" {
            Ok(self.result)
        } else {
            Err(OjError::StrError(self.messages.join("\n")))
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Problem {
    pub(crate) url: String,
    pub(crate) tests: Vec<TestCase>,
}

pub fn save_temp_file(buf: &[u8]) -> io::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(buf)?;
    Ok(file)
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TestCase {
    pub(crate) name: String,
    pub(crate) input: String,
    pub(crate) output: String,
}
impl TestCase {
    pub(crate) fn execute<'a, 'b>(
        &'a self,
        buf: &'b mut Vec<u8>,
        solve: fn(&mut &'a [u8], &'b mut Vec<u8>),
    ) {
        let mut bytes = self.input.as_bytes();
        solve(&mut bytes, buf);
    }
    pub(crate) fn judge_with_env(&self, result: &[u8], env: &VerifyEnv) -> VerifyStatus {
        self.judge_with_env_inner(result, env)
            .unwrap_or(VerifyStatus::IE)
    }
    pub(crate) fn judge_with_env_inner(
        &self,
        result: &[u8],
        env: &VerifyEnv,
    ) -> OjResult<VerifyStatus> {
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
    pub(crate) fn judge_with_judger<'a, 'b>(
        &'a self,
        mut result: &'b [u8],
        judger: fn(&mut &'a [u8], &mut &'a [u8], &mut &'b [u8]) -> bool,
    ) -> VerifyStatus {
        judger(
            &mut self.input.as_bytes(),
            &mut self.output.as_bytes(),
            &mut result,
        )
        .into()
    }
    #[allow(dead_code)]
    pub(crate) fn judge_with_eps<'a, 'b>(&'a self, result: &'b [u8], eps: f64) -> VerifyStatus {
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
                                    VerifyStatus::WA
                                } else {
                                    continue;
                                }
                            }
                            _ => VerifyStatus::WA,
                        },
                        (None, None) => VerifyStatus::AC,
                        _ => VerifyStatus::WA,
                    };
                }
            }
            Err(_) => return VerifyStatus::WA,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ServiceResult {
    pub(crate) url: String,
    pub(crate) name: String,
}

impl ServiceResult {
    pub(crate) fn into_service(self) -> Option<Service> {
        Some(match self.name.as_str() {
            "Library Checker" => Service::LibraryChecker,
            "Aizu Online Judge" => Service::AizuOnlineJudge,
            _ => None?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Service {
    LibraryChecker,
    AizuOnlineJudge,
}

pub(crate) struct OjApi {}
impl OjApi {
    pub(crate) fn call_with_args<R: DeserializeOwned, I, S>(args: I) -> OjResult<R>
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
        // eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        if output.status.success() {
            let response: OjApiResponse<R> = serde_json::from_slice(&output.stdout)?;
            response.into_result()
        } else {
            Err(OjError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
    pub(crate) fn get_testcases(url: &str) -> OjResult<Problem> {
        Self::call_with_args(&["get-problem", "--system", url])
    }
    pub(crate) fn get_service(url: &str) -> OjResult<Service> {
        let service: ServiceResult = Self::call_with_args(&["get-service", url])?;
        service.into_service().ok_or(OjError::UnsupportedService)
    }
}

#[derive(Debug)]
pub(crate) struct CheckerBinary {
    checker: PathBuf,
}
impl CheckerBinary {
    pub(crate) fn from_url(url: &str) -> OjResult<Self> {
        let output = {
            let _guard = OJ_API_RESOURCE.lock().unwrap();
            Command::new("python")
                .args(&[
                    "-c",
                    format!(
                        r#"from onlinejudge import dispatch
url = '{}'
problem = dispatch.problem_from_url(url)
try:
    print(problem.download_checker_binary(), end='')
except RuntimeError as e:
    assert False
"#,
                        url
                    )
                    .as_str(),
                ])
                .output()?
        };
        // eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        if output.status.success() {
            let checker = PathBuf::from(String::from_utf8_lossy(&output.stdout).to_string());
            Ok(Self { checker })
        } else {
            Err(OjError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
    pub(crate) fn check(
        &self,
        input: &Path,
        output: &Path,
        result: &Path,
    ) -> OjResult<VerifyStatus> {
        let output = Command::new(self.checker.as_os_str())
            .args(&[input.as_os_str(), result.as_os_str(), output.as_os_str()])
            .output()?;
        match output.status.code() {
            Some(0) => Ok(VerifyStatus::AC),
            Some(1) => Ok(VerifyStatus::WA),
            Some(_) => Ok(VerifyStatus::IE),
            None => Err(OjError::StrError("checker broken".to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum OjError {
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
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

pub(crate) type OjResult<T> = Result<T, OjError>;

#[derive(Clone, Debug)]
pub(crate) struct VerifyConfig {
    url: &'static str,
    cur_file: &'static str,
    fn_name: &'static str,
}
impl VerifyConfig {
    pub(crate) fn new(url: &'static str, cur_file: &'static str, fn_name: &'static str) -> Self {
        Self {
            url,
            cur_file,
            fn_name,
        }
    }
    pub(crate) fn gen_env(&self) -> OjResult<VerifyEnv> {
        Ok(match OjApi::get_service(self.url)? {
            Service::LibraryChecker => {
                VerifyEnv::LibraryChecker(CheckerBinary::from_url(self.url)?)
            }
            Service::AizuOnlineJudge => VerifyEnv::AizuOnlineJudge,
        })
    }
    pub(crate) fn get_testcases(&self) -> OjResult<Problem> {
        OjApi::get_testcases(self.url)
    }
    pub(crate) fn emit_md(&self, buf: &[u8]) -> io::Result<()> {
        let path = Path::new(self.cur_file)
            .with_file_name(self.fn_name)
            .with_extension("md");
        File::create(path)?.write_all(buf)
    }
    pub(crate) fn finalize(&self, result: OjResult<VerifyResults>) -> OjResult<()> {
        self.emit_md(self.gen_md_contents(&result).as_bytes())?;
        result.and_then(|r| {
            if r.is_ac() {
                Ok(())
            } else {
                Err(OjError::VerifyFailed(self.fn_name.to_string()))
            }
        })
    }
    pub(crate) fn gen_md_contents(&self, result: &OjResult<VerifyResults>) -> String {
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
            .unwrap_or("❌".to_string());
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
        format!(
            r###"{head}

problem [here]({url})

{detail}

"###,
            head = head,
            url = self.url,
            detail = detail
        )
    }
}

#[derive(Debug)]
pub(crate) enum VerifyEnv {
    LibraryChecker(CheckerBinary),
    AizuOnlineJudge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum VerifyStatus {
    /// Accepted
    AC,
    // /// Time Limit Exceeded
    // TLE,
    /// Wrong Answer
    WA,
    /// Runtime Error
    RE,
    /// Internal Error
    IE,
}
impl From<bool> for VerifyStatus {
    fn from(b: bool) -> Self {
        if b {
            Self::AC
        } else {
            Self::WA
        }
    }
}
impl fmt::Display for VerifyStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AC => write!(f, "AC"),
            Self::WA => write!(f, "WA"),
            Self::RE => write!(f, "RE"),
            Self::IE => write!(f, "IE"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct VerifyResult {
    pub(crate) name: String,
    pub(crate) status: VerifyStatus,
    pub(crate) elapsed: Duration,
}

impl VerifyResult {
    pub(crate) fn new(name: String, status: VerifyStatus, elapsed: Duration) -> Self {
        Self {
            name,
            status,
            elapsed,
        }
    }
}

#[derive(Debug)]
pub(crate) struct VerifyResults {
    pub(crate) results: Vec<VerifyResult>,
}

impl VerifyResults {
    pub(crate) fn new(results: Vec<VerifyResult>) -> Self {
        Self { results }
    }
    pub(crate) fn status(&self) -> VerifyStatus {
        self.results
            .iter()
            .map(|res| res.status)
            .max()
            .unwrap_or(VerifyStatus::AC)
    }
    pub(crate) fn is_ac(&self) -> bool {
        self.status() == VerifyStatus::AC
    }
    pub(crate) fn elapsed(&self) -> Duration {
        self.results
            .iter()
            .map(|res| res.elapsed)
            .max()
            .unwrap_or(Duration::from_secs(0))
    }
}
impl std::iter::FromIterator<VerifyResult> for VerifyResults {
    fn from_iter<T: IntoIterator<Item = VerifyResult>>(iter: T) -> Self {
        Self {
            results: Vec::from_iter(iter),
        }
    }
}
