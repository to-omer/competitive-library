use competitive::combinatorial_optimization::ZeroOneKnapsackProblemBranchAndBound;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_I")]
pub fn dpl_1_i(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, w: i64, vwm: [(i64, i64, i64); iter n]);
    let mut item = vec![];
    for (v, w, mut m) in vwm {
        let mut b = 1;
        while m > 0 {
            let k = b.min(m);
            m -= k;
            item.push((v * k, w * k));
            b *= 2;
        }
    }
    let knapsack = ZeroOneKnapsackProblemBranchAndBound::new(item);
    pp!(knapsack.solve(w));
}
