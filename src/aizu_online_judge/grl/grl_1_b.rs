pub use crate::algebra::AdditiveOperation;
pub use crate::graph::{
    AdjacencyGraphAbstraction, AdjacencyGraphBellmanFordExt, DirectedGraphScanner,
};
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/1/GRL_1_B")]
pub fn grl_1_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, r, (graph, _, d): { DirectedGraphScanner::<usize, i64>::new(vs, es) });
    let (cost, is_neg) = graph.bellman_ford(r, AdditiveOperation::new(), |eid| d[eid]);
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
