use competitive::combinatorial_optimization::ZeroOneKnapsackProblemSmallItems;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_H")]
pub fn dpl_1_h(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, w: i64, vw: [(i64, i64)]);
    let mut knapsack = ZeroOneKnapsackProblemSmallItems::new();
    knapsack.extend(vw.take(n));
    pp!(knapsack.solve(w));
}
