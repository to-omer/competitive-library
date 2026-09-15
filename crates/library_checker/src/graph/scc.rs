use competitive::graph::{DirectedSparseGraph, StronglyConnectedComponent};
use competitive::prelude::*;

#[verify::library_checker("scc")]
pub fn scc(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, edges: [(usize, usize); es]);
    let graph = DirectedSparseGraph::from_edges(vs, edges);
    let scc = StronglyConnectedComponent::new(&graph);
    let comp = scc.components();
    pp!(comp.len());
    for vs in comp.into_iter() {
        pp!(vs.len(), @it vs);
    }
}
