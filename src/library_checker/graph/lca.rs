pub use crate::graph::Graph;
use crate::prelude::*;
pub use crate::tree::{EulerTourForRichVertex, HeavyLightDecomposition};

#[verify_attr::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca_euler_tour(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize; n - 1]);
    let mut graph = Graph::new(n);
    for v in 0..n - 1 {
        graph.add_undirected_edge(v + 1, p[v]);
    }
    let mut euler = EulerTourForRichVertex::new(n);
    euler.vertex_tour(0, n, &graph);
    let lca = euler.gen_lca(&graph);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}

#[verify_attr::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca_hld(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize; n - 1]);
    let mut graph = Graph::new(n);
    for v in 0..n - 1 {
        graph.add_undirected_edge(v + 1, p[v]);
    }
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", hld.lca(u, v)).ok();
    }
}
