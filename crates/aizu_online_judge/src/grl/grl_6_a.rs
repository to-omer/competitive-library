use competitive::graph::DinicBuilder;
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_6_A")]
pub fn grl_6_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, edges: [(usize, usize, u64); iter es]);
    let mut builder = DinicBuilder::new(vs, es);
    builder.extend(edges);
    let graph = builder.gen_graph();
    let mut dinic = builder.build(&graph);
    pp!(dinic.maximum_flow(0, vs - 1));
}
