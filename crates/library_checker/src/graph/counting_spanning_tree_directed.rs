use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation, graph::EdgeListGraph, num::mint_basic::MInt998244353 as M,
};

#[verify::library_checker("counting_spanning_tree_directed")]
pub fn counting_spanning_tree_directed(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, r: usize, edges: [(usize, usize); m]);
    let graph = EdgeListGraph::from_edges(n, edges);
    pp!(graph.count_spanning_arborescences::<AddMulOperation<M>>(r));
}
