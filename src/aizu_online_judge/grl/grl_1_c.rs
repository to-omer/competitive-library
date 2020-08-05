pub use crate::algebra::AdditiveOperation;
pub use crate::graph::GraphScanner;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_C")]
pub fn grl_1_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, d) = scanner.mscan(GraphScanner::<usize, i64>::new(vs, es, true));
    let cost = graph.warshall_floyd(AdditiveOperation::new(), |eid| d[eid]);
    if graph.vertices().any(|u| cost[u][u].unwrap() < 0) {
        writeln!(writer, "NEGATIVE CYCLE").ok();
    } else {
        for u in graph.vertices() {
            for v in graph.vertices() {
                match cost[u][v] {
                    Some(d) => write!(writer, "{}", d),
                    None => write!(writer, "INF"),
                }
                .ok();
                write!(writer, "{}", if v + 1 == vs { '\n' } else { ' ' }).ok();
            }
        }
    }
}
