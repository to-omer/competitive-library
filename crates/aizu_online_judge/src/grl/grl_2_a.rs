use competitive::graph::EdgeListGraphScanner;
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_2_A")]
pub fn grl_2_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(vs, es, (graph, w): @EdgeListGraphScanner::<usize, u32>::new(vs, es));
    let span = graph.minimum_spanning_tree(|&eid| w[eid]);
    let ans = (0..es)
        .map(|eid| u64::from(w[eid]) * span[eid] as u64)
        .sum::<u64>();
    pp!(ans);
}
