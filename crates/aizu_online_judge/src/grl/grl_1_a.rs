use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    graph::{DirectedGraphScanner, ShortestPathExt},
    num::Bounded,
};

#[verify::aizu_online_judge("GRL_1_A")]
pub fn grl_1_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, d): @DirectedGraphScanner::<usize, u64>::new(vs, es));
    let cost = graph.standard_sp_additive().dijkstra([r], &d);
    for u in graph.vertices() {
        if cost[u].is_maximum() {
            writeln!(writer, "INF").ok();
        } else {
            writeln!(writer, "{}", cost[u]).ok();
        }
    }
}

#[verify::aizu_online_judge("GRL_1_A")]
pub fn grl_1_a_option(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, d): @DirectedGraphScanner::<usize, u64>::new(vs, es));
    let cost = graph
        .option_sp_additive()
        .dijkstra([r], &|eid| Some(d[eid]));
    for u in graph.vertices() {
        match cost[u] {
            Some(d) => writeln!(writer, "{}", d).ok(),
            None => writeln!(writer, "INF").ok(),
        };
    }
}
