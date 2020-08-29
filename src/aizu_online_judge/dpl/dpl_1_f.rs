pub use crate::combinatorial_optimization::KnapsackPloblemSmallValue;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_F")]
pub fn dpl_1_f(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, w, vw: [(usize, usize); n]);
    let mut knapsack = KnapsackPloblemSmallValue::new(vw.iter().map(|&(v, _)| v).sum::<usize>());
    knapsack.extend01(vw);
    writeln!(writer, "{}", knapsack.solve(w)).ok();
}