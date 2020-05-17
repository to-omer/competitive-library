use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize};
use std::ffi::OsStr;
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
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

impl Problem {
    pub(crate) fn verify<'a>(
        &'a self,
        env: VerifyEnv,
        f: fn(&mut Cursor<&'a [u8]>, &mut Vec<u8>) -> io::Result<()>,
    ) -> OjResult<VerifyResults> {
        let mut res = Vec::with_capacity(self.tests.len());
        for case in self.tests.iter() {
            res.push(env.run_judge(case, f));
        }
        Ok(VerifyResults::new(res))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TestCase {
    pub(crate) name: String,
    pub(crate) input: String,
    pub(crate) output: String,
}
impl TestCase {
    pub(crate) fn save(&self) -> io::Result<(NamedTempFile, NamedTempFile)> {
        let mut infile = NamedTempFile::new()?;
        let mut outfile = NamedTempFile::new()?;
        infile.write_all(self.input.as_bytes())?;
        outfile.write_all(self.output.as_bytes())?;
        Ok((infile, outfile))
    }
    pub(crate) fn execute<'a>(
        &'a self,
        f: fn(&mut Cursor<&'a [u8]>, &mut Vec<u8>) -> io::Result<()>,
    ) -> (io::Result<Vec<u8>>, Duration) {
        let mut buf = Vec::new();
        let bytes = self.input.as_bytes();
        let mut cur = Cursor::new(bytes);
        let start = Instant::now();
        let res = f(&mut cur, &mut buf);
        let d = start.elapsed();
        (res.map(|_| buf), d)
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
        let _guard = OJ_API_RESOURCE.lock().unwrap();
        let output = Command::new(option_env!("ONLINE_JUDGE_TOOLS_API").unwrap_or("oj-api"))
            .args(args)
            .output()?;
        println!("{}", String::from_utf8_lossy(&output.stderr));
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
        let _guard = OJ_API_RESOURCE.lock().unwrap();
        let output = Command::new("python")
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
            .output()?;
        println!("{}", String::from_utf8_lossy(&output.stderr));
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
        std::fs::File::create(path)?.write_all(buf)
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

impl VerifyEnv {
    fn run_judge_innter<'a>(
        &self,
        case: &'a TestCase,
        f: fn(&mut Cursor<&'a [u8]>, &mut Vec<u8>) -> io::Result<()>,
    ) -> OjResult<VerifyResult> {
        let (result, elapsed) = case.execute(f);
        match result {
            Ok(buf) => match self {
                VerifyEnv::LibraryChecker(checker) => {
                    let (input, output) = case.save()?;
                    let mut result = NamedTempFile::new()?;
                    result.write_all(&buf)?;
                    let status = checker.check(input.path(), output.path(), result.path())?;
                    Ok(VerifyResult::new(case.name.clone(), status, elapsed))
                }
                VerifyEnv::AizuOnlineJudge => {
                    let status = if buf == case.output.as_bytes() {
                        VerifyStatus::AC
                    } else {
                        VerifyStatus::WA
                    };
                    Ok(VerifyResult::new(case.name.clone(), status, elapsed))
                }
            },
            Err(_err) => Ok(VerifyResult::new(
                case.name.clone(),
                VerifyStatus::RE,
                elapsed,
            )),
        }
    }
    pub(crate) fn run_judge<'a>(
        &self,
        case: &'a TestCase,
        f: fn(&mut Cursor<&'a [u8]>, &mut Vec<u8>) -> io::Result<()>,
    ) -> VerifyResult {
        self.run_judge_innter(case, f).unwrap_or_else(|_err| {
            VerifyResult::new(case.name.clone(), VerifyStatus::IE, Duration::from_secs(0))
        })
    }
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
impl std::fmt::Display for VerifyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyStatus::AC => write!(f, "AC"),
            VerifyStatus::WA => write!(f, "WA"),
            VerifyStatus::RE => write!(f, "RE"),
            VerifyStatus::IE => write!(f, "IE"),
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
