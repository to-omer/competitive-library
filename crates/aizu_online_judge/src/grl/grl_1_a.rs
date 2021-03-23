use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, graph::DirectedGraphScanner};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_A")]
pub fn grl_1_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, d): { DirectedGraphScanner::<usize, u64>::new(vs, es) });
    let cost = graph.dijkstra::<AdditiveOperation<_>, _>(r, |eid| d[eid]);
    for u in graph.vertices() {
        match cost[u] {
            Some(d) => writeln!(writer, "{}", d),
            None => writeln!(writer, "INF"),
        }
        .ok();
    }
}
