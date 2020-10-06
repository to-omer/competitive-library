use crate::config::Config;
use serde::Serialize;
use serde_json::to_writer;
use std::{
    fs,
    io::{self, Write as _},
    path::Path,
    process::{Command, Stdio},
};

pub fn emit<T: Serialize>(value: &T, config: &Config) -> io::Result<()> {
    match &config.output {
        Some(file) => {
            let f = fs::File::create(file)?;
            to_writer(f, value)?;
        }
        _ => {
            to_writer(io::stdout().lock(), value)?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
pub struct VSCode {
    prefix: String,
    body: String,
    scope: String,
}
impl From<(String, String)> for VSCode {
    fn from((prefix, contents): (String, String)) -> Self {
        Self {
            prefix,
            body: contents.replace("$", "\\$"),
            scope: "rust".to_string(),
        }
    }
}

pub fn rustfmt_exits() -> bool {
    let rustfmt = Path::new(env!("CARGO_HOME")).join("bin").join("rustfmt");
    let output = Command::new(rustfmt).arg("--version").output();
    output
        .map(|output| output.status.success())
        .unwrap_or_default()
}

pub fn format_with_rustfmt(s: &str) -> Option<String> {
    let rustfmt = Path::new(env!("CARGO_HOME")).join("bin").join("rustfmt");
    let mut command = Command::new(rustfmt)
        .args(&[
            "--quiet",
            "--config",
            "unstable_features=true,normalize_doc_attributes=true,newline_style=Unix",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    command.stdin.take().unwrap().write_all(s.as_bytes()).ok()?;
    let output = command.wait_with_output().ok()?;
    if output.status.success() {
        Some(unsafe { String::from_utf8_unchecked(output.stdout) })
    } else {
        None
    }
}

#[test]
fn test_format_contents() {
    assert_eq!(
        format_with_rustfmt("fn  main ( ) { }"),
        Some("fn main() {}\n".to_string())
    )
}
