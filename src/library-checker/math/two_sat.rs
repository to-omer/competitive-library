// verify-helper: PROBLEM https://judge.yosupo.jp/problem/two_sat

use competitive_library::graph::strongly_connected_component::TwoSatisfiability;
use competitive_library::input;
use std::io::{BufWriter, StdoutLock, Write};

fn solve<'a>(out: &mut BufWriter<StdoutLock<'a>>) -> std::io::Result<()> {
    input! { iter = iter, _p: chars, _cnf: chars, n, m, ab: [(i64, i64, i64); m] };
    let mut two_sat = TwoSatisfiability::new(n);
    for (a, b, _) in ab.into_iter() {
        let u = (a.abs() as usize - 1) * 2;
        let v = (b.abs() as usize - 1) * 2;
        let na = (a < 0) as usize;
        let nb = (b < 0) as usize;
        two_sat.add_inner(u ^ na ^ 1, v ^ nb);
    }
    if let Some(v) = two_sat.build() {
        writeln!(out, "s SATISFIABLE")?;
        write!(out, "v")?;
        for i in 0..n {
            write!(
                out,
                " {}",
                if v[i] { i as i64 + 1 } else { -(i as i64 + 1) }
            )?;
        }
        write!(out, " 0")?;
    } else {
        writeln!(out, "s UNSATISFIABLE")?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || -> std::io::Result<()> {
            let stdout = std::io::stdout();
            let mut out = BufWriter::new(stdout.lock());
            solve(&mut out)
        })?
        .join()
        .unwrap()
}
