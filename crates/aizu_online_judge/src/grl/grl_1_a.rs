pub use competitive::algebra::AdditiveOperation;
pub use competitive::graph::DirectedGraphScanner;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_A")]
pub fn grl_1_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, d): { DirectedGraphScanner::<usize, u64>::new(vs, es) });
    let cost = graph.dijkstra(r, AdditiveOperation::new(), |eid| d[eid]);
    for u in graph.vertices() {
        match cost[u] {
            Some(d) => writeln!(writer, "{}", d),
            None => writeln!(writer, "INF"),
        }
        .ok();
    }
}
