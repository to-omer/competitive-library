pub use competitive::algebra::AdditiveOperation;
pub use competitive::graph::{EdgeListGraph, EdgeListGraphScanner};
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/2/GRL_2_B")]
pub fn grl_2_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, root, (graph, w): { EdgeListGraphScanner::<usize, i64>::new(vs, es) });
    let res = graph.minimum_spanning_arborescence(root, AdditiveOperation::new(), |u| w[u]);
    writeln!(writer, "{}", res.unwrap().0).ok();
}
