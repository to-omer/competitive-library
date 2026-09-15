use competitive::prelude::*;
use competitive::{
    graph::{DirectedGraphScanner, ShortestPathExt},
    num::Bounded,
};

#[verify::aizu_online_judge("GRL_1_A")]
pub fn grl_1_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, r, (graph, d): @DirectedGraphScanner::<usize, u64>::new(vs, es));
    let cost = graph.standard_sp_additive().dijkstra([r], |eid| d[eid]);
    for u in graph.vertices() {
        if cost[u].is_maximum() {
            pp!("INF");
        } else {
            pp!(cost[u]);
        }
    }
}

#[verify::aizu_online_judge("GRL_1_A")]
pub fn grl_1_a_option(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, r, (graph, d): @DirectedGraphScanner::<usize, u64>::new(vs, es));
    let cost = graph.option_sp_additive().dijkstra([r], |eid| Some(d[eid]));
    for u in graph.vertices() {
        match cost[u] {
            Some(d) => pp!(d),
            None => pp!("INF"),
        };
    }
}
