use competitive::prelude::*;
use competitive::{
    graph::{DirectedGraphScanner, ShortestPathExt},
    num::Saturating,
};

#[verify::aizu_online_judge("GRL_1_C")]
pub fn grl_1_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, (graph, d): @DirectedGraphScanner::<usize, i64>::new(vs, es));
    let cost = graph
        .option_sp_additive()
        .warshall_floyd_ap(|eid| Some(Saturating(d[eid])));
    if graph.vertices().any(|u| cost[u][u].unwrap().0 < 0) {
        pp!("NEGATIVE CYCLE");
    } else {
        for u in graph.vertices() {
            for v in graph.vertices() {
                match cost[u][v] {
                    Some(d) => pp!(d.0, !),
                    None => pp!("INF", !),
                };
                pp!(if v + 1 == vs { '\n' } else { ' ' }, !);
            }
        }
    }
}
