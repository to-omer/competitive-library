#[doc(no_inline)]
pub use competitive::combinatorial_optimization::LongestIncreasingSubsequence;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_D")]
pub fn dpl_1_d(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [u64]);
    let mut lis = LongestIncreasingSubsequence::new();
    lis.extend(a.take(n));
    writeln!(writer, "{}", lis.longest_length()).ok();
}
