// verify-helper: PROBLEM https://judge.yosupo.jp/problem/bipartitematching

use competitive::graph::maximum_flow::{Dinic, RevEdge};
use competitive::input;
use std::io::{BufWriter, StdoutLock, Write};

fn solve<'a>(out: &mut BufWriter<StdoutLock<'a>>) -> std::io::Result<()> {
    input! { iter = iter, l, r, m, ab: [(usize, usize); m] };
    let mut dinic = Dinic::new(l + r + 2);
    for a in 0..l {
        dinic.add_edge(0, a + 1, 1);
    }
    for b in 0..r {
        dinic.add_edge(l + b + 1, l + r + 1, 1);
    }
    for (a, b) in ab.into_iter() {
        dinic.add_edge(a + 1, l + b + 1, 1);
    }
    let f = dinic.maximum_flow(0, l + r + 1);
    writeln!(out, "{}", f)?;
    for a in 0..l {
        for i in 0..dinic.graph[a + 1].len() {
            let RevEdge { to, cap, rev: _ } = dinic.graph[a + 1][i];
            if l < to && cap == 0 {
                writeln!(out, "{} {}", a, to - l - 1)?;
            }
        }
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
