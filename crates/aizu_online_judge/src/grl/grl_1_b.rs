use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, graph::DirectedGraphScanner};

#[verify::aizu_online_judge("GRL_1_B")]
pub fn grl_1_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, d): @DirectedGraphScanner::<usize, i64>::new(vs, es));
    let (cost, is_neg) = graph.bellman_ford::<AdditiveOperation<_>, _>(r, |eid| d[eid]);
    if is_neg {
        writeln!(writer, "NEGATIVE CYCLE").ok();
    } else {
        for u in graph.vertices() {
            match cost[u] {
                Some(d) => writeln!(writer, "{}", d),
                None => writeln!(writer, "INF"),
            }
            .ok();
        }
    }
}
