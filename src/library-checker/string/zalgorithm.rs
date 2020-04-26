// verify-helper: PROBLEM https://judge.yosupo.jp/problem/zalgorithm

use competitive_library::input;
use competitive_library::string::z_algorithm::Zarray;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { iter = iter, s: chars };
    let z = Zarray::new(&s);
    for i in 0..s.len() {
        write!(out, "{}{}", if i == 0 { "" } else { " " }, z[i])?;
    }
    writeln!(out, "")?;

    Ok(())
}
