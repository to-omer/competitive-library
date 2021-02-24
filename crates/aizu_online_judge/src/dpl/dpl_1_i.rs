#[doc(no_inline)]
pub use competitive::combinatorial_optimization::ZeroOneKnapsackPloblemBranchAndBound;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_I")]
pub fn dpl_1_i(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, w: u64, vwm: [(u64, u64, u64)]);
    let mut item = vec![];
    for (v, w, mut m) in vwm.take(n) {
        let mut b = 1;
        while m > 0 {
            let k = b.min(m);
            m -= k;
            item.push((v * k, w * k));
            b *= 2;
        }
    }
    let knapsack = ZeroOneKnapsackPloblemBranchAndBound::new(item);
    writeln!(writer, "{}", knapsack.solve(w)).ok();
}
