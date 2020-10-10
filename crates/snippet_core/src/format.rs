use std::{
    io::Write as _,
    path::Path,
    process::{Command, Stdio},
};

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
