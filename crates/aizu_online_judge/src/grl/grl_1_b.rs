use competitive::graph::{DirectedGraphScanner, ShortestPathExt};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_1_B")]
pub fn grl_1_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, r, (graph, d): @DirectedGraphScanner::<usize, i64>::new(vs, es));
    let cost = graph
        .option_sp_additive()
        .bellman_ford([r], |eid| Some(d[eid]), true);
    if let Some(cost) = cost {
        for u in graph.vertices() {
            match cost[u] {
                Some(d) => pp!(d),
                None => pp!("INF"),
            };
        }
    } else {
        pp!("NEGATIVE CYCLE");
    }
}
