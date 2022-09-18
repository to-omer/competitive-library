#[doc(no_inline)]
pub use competitive::combinatorial_optimization::ZeroOneKnapsackProblemSmallItems;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_H")]
pub fn dpl_1_h(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, w: i64, vw: [(i64, i64)]);
    let mut knapsack = ZeroOneKnapsackProblemSmallItems::new();
    knapsack.extend(vw.take(n));
    writeln!(writer, "{}", knapsack.solve(w)).ok();
}
