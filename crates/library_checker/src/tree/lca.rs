use competitive::graph::UndirectedSparseGraph;
use competitive::prelude::*;
use competitive::tree::LowestCommonAncestor;

#[verify::library_checker("lca")]
pub fn lca(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize]);
    let mut parents = vec![!0];
    parents.extend(p.take(n - 1));
    let lca = LowestCommonAncestor::from_parents(&parents);
    for _ in 0..q {
        scan!(scanner, u, v);
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}

#[verify::library_checker("lca")]
pub fn lca_hld(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let hld = graph.hld(0);
    for _ in 0..q {
        scan!(scanner, u, v);
        writeln!(writer, "{}", hld.lca(u, v)).ok();
    }
}
