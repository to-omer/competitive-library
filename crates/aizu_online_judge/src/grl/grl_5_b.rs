use competitive::prelude::*;
use competitive::{algebra::MaxOperation, graph::TreeGraphScanner, tree::ReRooting};

#[verify::aizu_online_judge("GRL_5_B")]
pub fn grl_5_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, (graph, w): @TreeGraphScanner::<usize, u64>::new(n));
    let re = ReRooting::<MaxOperation<u64>, _>::new(&graph, |d, _vid, eid_opt| {
        d + eid_opt.map_or(0, |eid| w[eid])
    });
    pp!(@sep '\n', @it re.dp);
}
