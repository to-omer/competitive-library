use competitive::graph::StronglyConnectedComponent;
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_3_C", include("StronglyConnectedComponent"))]
pub fn grl_3_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, edges: [(usize, usize); es]);
    let ids = StronglyConnectedComponent::from_edges(vs, edges.iter().copied()).component_ids();
    sc!(q);
    for (u, v) in sv!([(usize, usize); iter q]) {
        pp!((ids[u] == ids[v]) as u32);
    }
}
