use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, graph::DinicBuilder};

#[verify::aizu_online_judge("GRL_6_A")]
pub fn grl_6_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, edges: [(usize, usize, u64)]);
    let mut builder = DinicBuilder::new(vs, es);
    builder.extend(edges.take(es));
    let graph = builder.gen_graph();
    let mut dinic = builder.build(&graph);
    writeln!(writer, "{}", dinic.maximum_flow(0, vs - 1)).ok();
}
