use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    graph::UndirectedSparseGraph,
    tree::{EulerTourForRichVertex, HeavyLightDecomposition},
};

#[verify::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca_euler_tour(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let euler = EulerTourForRichVertex::new(0, &graph);
    let lca = euler.gen_lca();
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}

#[verify::verify("https://judge.yosupo.jp/problem/lca")]
pub fn lca_hld(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let mut graph = UndirectedSparseGraph::from_edges(n, edges);
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", hld.lca(u, v)).ok();
    }
}
