use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation,
    graph::{DirectedGraphScanner, OptionSp, ShortestPathExt},
    num::Saturating,
};

#[verify::aizu_online_judge("GRL_1_C")]
pub fn grl_1_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, d): @DirectedGraphScanner::<usize, i64>::new(vs, es));
    let cost = graph
        .warshall_floyd_ap::<OptionSp<AdditiveOperation<_>>, _>(&|eid| Some(Saturating(d[eid])));
    if graph.vertices().any(|u| cost[u][u].unwrap().0 < 0) {
        writeln!(writer, "NEGATIVE CYCLE").ok();
    } else {
        for u in graph.vertices() {
            for v in graph.vertices() {
                match cost[u][v] {
                    Some(d) => write!(writer, "{}", d.0),
                    None => write!(writer, "INF"),
                }
                .ok();
                write!(writer, "{}", if v + 1 == vs { '\n' } else { ' ' }).ok();
            }
        }
    }
}
