use competitive::graph::PrimalDualBuilder;
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_6_B")]
pub fn grl_6_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, f: u64, edges: [(usize, usize, u64, i64)]);
    let mut builder = PrimalDualBuilder::new(vs, es);
    builder.extend(edges.take(es));
    let graph = builder.gen_graph();
    let mut pd = builder.build(&graph);
    let (flow, cost) = pd.minimum_cost_flow_limited(0, vs - 1, f);
    pp!(if flow < f { -1 } else { cost });
}
