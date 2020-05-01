// verify-helper: PROBLEM https://judge.yosupo.jp/problem/scc

use competitive::graph::strongly_connected_component::StronglyConnectedComponent;
use competitive::input;
use std::io::{BufWriter, StdoutLock, Write};

fn solve<'a>(out: &mut BufWriter<StdoutLock<'a>>) -> std::io::Result<()> {
    input! { iter = iter, n, m, ab: [(usize, usize); m] };
    let mut scc = StronglyConnectedComponent::new(n);
    for (a, b) in ab.into_iter() {
        scc.add_edge(a, b);
    }
    scc.build();
    let comp = scc.component();
    writeln!(out, "{}", comp.len())?;
    for vs in comp.into_iter() {
        write!(out, "{}", vs.len())?;
        for v in vs.into_iter() {
            write!(out, " {}", v)?;
        }
        writeln!(out, "")?;
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
