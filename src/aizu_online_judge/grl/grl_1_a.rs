pub use crate::algebra::AdditiveOperation;
pub use crate::graph::GraphScanner;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_A")]
pub fn grl_1_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r);
    let (graph, d) = scanner.mscan(GraphScanner::<usize, u64>::new(vs, es, true));
    let cost = graph.dijkstra(r, AdditiveOperation::new(), |eid| d[eid]);
    for u in graph.vertices() {
        match cost[u] {
            Some(d) => writeln!(writer, "{}", d),
            None => writeln!(writer, "INF"),
        }
        .ok();
    }
}
