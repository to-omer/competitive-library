use competitive::graph::UndirectedSparseGraph;
use competitive::prelude::*;
use competitive::tree::LowestCommonAncestor;

#[verify::library_checker("lca")]
pub fn lca(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, p: [usize; iter n - 1]);
    let mut parents = vec![!0];
    parents.extend(p);
    let lca = LowestCommonAncestor::from_parents(&parents);
    for _ in 0..q {
        sc!(u, v);
        pp!(lca.lca(u, v));
    }
}

#[verify::library_checker("lca")]
pub fn lca_hld(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, p: [usize; iter n - 1]);
    let edges = p.enumerate().map(|(i, p)| (i + 1, p)).collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let hld = graph.hld(0);
    for _ in 0..q {
        sc!(u, v);
        pp!(hld.lca(u, v));
    }
}
