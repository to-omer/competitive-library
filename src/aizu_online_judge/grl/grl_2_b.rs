pub use crate::algebra::AdditiveOperation;
pub use crate::graph::GraphScanner;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/2/GRL_2_B")]
pub fn grl_2_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, root);
    let (graph, w) = scanner.mscan(GraphScanner::<usize, i64>::new(vs, es, true));
    let res = graph.minimum_spanning_arborescence(root, AdditiveOperation::new(), |u| w[u]);
    writeln!(writer, "{}", res.unwrap().0).ok();
}
