// verify-helper: PROBLEM https://judge.yosupo.jp/problem/many_aplusb

use competitive_library::input;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    input! { t, ab: [(usize, usize); t] };
    for (a, b) in ab {
        writeln!(out, "{}", a + b)?;
    }

    Ok(())
}
