use competitive::graph::{DirectedGraphScanner, StronglyConnectedComponent};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_3_C")]
pub fn grl_3_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, (graph, _): @DirectedGraphScanner::<usize, ()>::new(vs, es));
    let scc = StronglyConnectedComponent::new(&graph);
    sc!(q);
    for (u, v) in sv!([(usize, usize); iter q]) {
        pp!((scc[u] == scc[v]) as u32);
    }
}
