pub use crate::graph::{AdjacencyGraphAbstraction, UndirectedSparseGraph};
use crate::prelude::*;
pub use crate::tree::{EulerTourForRichVertex, HeavyLightDecomposition};

#[verify_attr::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca_euler_tour(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize; n - 1]);
    let graph =
        UndirectedSparseGraph::from_edges(n, p.iter().enumerate().map(|(v, &p)| (v + 1, p)));
    let euler = EulerTourForRichVertex::new(0, &graph);
    let lca = euler.gen_lca();
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}

#[verify_attr::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca_hld(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize; n - 1]);
    let mut graph =
        UndirectedSparseGraph::from_edges(n, p.iter().enumerate().map(|(v, &p)| (v + 1, p)));
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", hld.lca(u, v)).ok();
    }
}
