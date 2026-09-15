use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, graph::TreeGraphScanner};

#[verify::aizu_online_judge("GRL_5_A")]
pub fn grl_5_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, (graph, w): @TreeGraphScanner::<usize, u64>::new(n));
    let d = graph.weighted_tree_depth::<AdditiveOperation<_>, _>(0, |eid| w[eid]);
    let r = (0..n).max_by_key(|&u| d[u]).unwrap();
    let ans = graph
        .weighted_tree_depth::<AdditiveOperation<_>, _>(r, |eid| w[eid])
        .into_iter()
        .max()
        .unwrap();
    pp!(ans);
}
