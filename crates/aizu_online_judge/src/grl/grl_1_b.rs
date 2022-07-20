use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation,
    graph::{DirectedGraphScanner, OptionSp, ShortestPathExt},
};

#[verify::aizu_online_judge("GRL_1_B")]
pub fn grl_1_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, d): @DirectedGraphScanner::<usize, i64>::new(vs, es));
    let cost =
        graph.bellman_ford_ss::<OptionSp<AdditiveOperation<_>>, _>(r, &|eid| Some(d[eid]), true);
    if let Some(cost) = cost {
        for u in graph.vertices() {
            match cost[u] {
                Some(d) => writeln!(writer, "{}", d).ok(),
                None => writeln!(writer, "INF").ok(),
            };
        }
    } else {
        writeln!(writer, "NEGATIVE CYCLE").ok();
    }
}
