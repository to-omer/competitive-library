use competitive::graph::{LowLink, UndirectedGraphScanner};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_3_B")]
pub fn grl_3_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, (graph, _): @UndirectedGraphScanner::<usize, ()>::new(vs, es));
    let mut bridge = LowLink::new(&graph).bridge;
    bridge.sort_unstable();
    for (u, v) in bridge.into_iter() {
        pp!(u, v);
    }
}
