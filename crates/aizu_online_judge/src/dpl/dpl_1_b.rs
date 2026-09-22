use competitive::combinatorial_optimization::KnapsackProblemSmallWeight;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_B")]
pub fn dpl_1_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, w, vw: [(i64, usize); iter n]);
    let mut knapsack = KnapsackProblemSmallWeight::new(w);
    knapsack.extend01(vw);
    pp!(knapsack.solve().unwrap_or_default());
}
