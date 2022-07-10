#[doc(no_inline)]
pub use competitive::combinatorial_optimization::KnapsackPloblemSmallWeight;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_C")]
pub fn dpl_1_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, w, vw: [(usize, usize)]);
    let mut knapsack = KnapsackPloblemSmallWeight::new(w);
    knapsack.extend(vw.take(n));
    writeln!(writer, "{}", knapsack.solve()).ok();
}
