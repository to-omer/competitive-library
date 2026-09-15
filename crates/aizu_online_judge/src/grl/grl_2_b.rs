use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, graph::EdgeListGraphScanner};

#[verify::aizu_online_judge("GRL_2_B")]
pub fn grl_2_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, root, (graph, w): @EdgeListGraphScanner::<usize, i64>::new(vs, es));
    let res = graph.minimum_spanning_arborescence::<AdditiveOperation<_>, _>(root, |u| w[u]);
    pp!(res.unwrap().0);
}
