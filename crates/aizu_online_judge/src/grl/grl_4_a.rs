use competitive::graph::{DirectedGraphScanner, TopologicalSortExt};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_4_A")]
pub fn grl_4_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, (graph, _): @DirectedGraphScanner::<usize, ()>::new(vs, es));
    pp!((graph.topological_sort().len() != vs) as u32);
}
