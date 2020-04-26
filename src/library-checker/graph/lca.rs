// verify-helper: PROBLEM https://judge.yosupo.jp/problem/lca

use competitive_library::graph::Graph;
use competitive_library::input;
use competitive_library::tree::euler_tour::EulerTourForRichVertex;
use std::io::{BufWriter, StdoutLock, Write};

fn solve<'a>(out: &mut BufWriter<StdoutLock<'a>>) -> std::io::Result<()> {
    input! { iter = iter, n, q, p: [usize; n - 1], uv: [(usize, usize); q] };
    let mut graph = Graph::new(n);
    for v in 0..n - 1 {
        graph.add_undirected_edge(v + 1, p[v]);
    }
    let mut euler = EulerTourForRichVertex::new(n);
    euler.vertex_tour(0, n, &graph);
    let lca = euler.gen_lca(&graph);
    for (u, v) in uv.into_iter() {
        writeln!(out, "{}", lca.lca(u, v))?;
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
