#[doc(no_inline)]
pub use competitive::combinatorial_optimization::KnapsackPloblemSmallWeight;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_B")]
pub fn dpl_1_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, w, vw: [(usize, usize)]);
    let mut knapsack = KnapsackPloblemSmallWeight::new(w);
    knapsack.extend01(vw.take(n));
    writeln!(writer, "{}", knapsack.solve()).ok();
}
