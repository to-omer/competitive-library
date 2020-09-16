pub use crate::algebra::AdditiveOperation;
pub use crate::graph::PrimalDualBuilder;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/6/GRL_6_B")]
pub fn grl_6_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, f: u64, edges: [(usize, usize, u64, i64)]);
    let mut builder = PrimalDualBuilder::new(vs, es);
    builder.extend(edges.take(es));
    let graph = builder.gen_graph();
    let mut pd = builder.build(&graph);
    let (flow, cost) = pd.minimum_cost_flow_limited(0, vs - 1, f);
    writeln!(writer, "{}", if flow < f { -1 } else { cost }).ok();
}
