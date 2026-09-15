use competitive::prelude::*;
use competitive::{graph::UndirectedSparseGraph, tools::SizedCollect};

#[verify::aizu_online_judge("GRL_5_C")]
pub fn grl_5_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, c: [SizedCollect<usize>]);
    let edges = c
        .take(n)
        .enumerate()
        .flat_map(|(u, it)| it.into_iter().map(move |v| (u, v)))
        .collect();
    let tree = UndirectedSparseGraph::from_edges(n, edges);
    let lca = tree.lca(0);
    sc!(q, uv: [(usize, usize)]);
    for (u, v) in uv.take(q) {
        pp!(lca.lca(u, v));
    }
}
