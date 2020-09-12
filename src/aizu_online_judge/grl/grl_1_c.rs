pub use crate::algebra::AdditiveOperation;
pub use crate::graph::{
    AdjacencyGraphAbstraction, AdjacencyGraphWarshallFloydExt, DirectedGraphScanner,
};
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_C")]
pub fn grl_1_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _, d): { DirectedGraphScanner::<usize, i64>::new(vs, es) });
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
