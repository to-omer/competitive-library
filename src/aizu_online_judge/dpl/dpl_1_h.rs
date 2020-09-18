pub use crate::combinatorial_optimization::ZeroOneKnapsackProblemSmallItems;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_H")]
pub fn dpl_1_h(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, w: u64, vw: [(u64, u64)]);
    let mut knapsack = ZeroOneKnapsackProblemSmallItems::new();
    knapsack.extend(vw.take(n));
    writeln!(writer, "{}", knapsack.solve(w)).ok();
}
