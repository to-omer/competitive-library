use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{graph::UndirectedSparseGraph, tree::HeavyLightDecomposition};

#[verify::library_checker("lca")]
pub fn lca_euler_tour(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let tree = UndirectedSparseGraph::from_edges(n, edges);
    let lca = tree.lca(0);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}

#[verify::library_checker("lca")]
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
