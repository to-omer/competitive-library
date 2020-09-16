pub use crate::algebra::AdditiveOperation;
pub use crate::graph::DinicBuilder;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/6/GRL_6_A")]
pub fn grl_6_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, edges: [(usize, usize, u64)]);
    let mut builder = DinicBuilder::new(vs, es);
    builder.extend(edges.take(es));
    let graph = builder.gen_graph();
    let mut dinic = builder.build(&graph);
    writeln!(writer, "{}", dinic.maximum_flow(0, vs - 1)).ok();
}
