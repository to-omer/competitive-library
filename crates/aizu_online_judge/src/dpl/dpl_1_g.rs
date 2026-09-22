use competitive::combinatorial_optimization::KnapsackProblemSmallWeight;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_G")]
pub fn dpl_1_g(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, w, vwm: [(i64, usize, usize); iter n]);
    let mut knapsack = KnapsackProblemSmallWeight::new(w);
    knapsack.extend_limitation(vwm);
    pp!(knapsack.solve().unwrap_or_default());
}
