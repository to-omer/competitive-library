use competitive::graph::{LowLink, UndirectedGraphScanner};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_3_A")]
pub fn grl_3_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, (graph, _): @UndirectedGraphScanner::<usize, ()>::new(vs, es));
    let mut articulation = LowLink::new(&graph).articulation;
    articulation.sort_unstable();
    for u in articulation.into_iter() {
        pp!(u);
    }
}
