pub use crate::combinatorial_optimization::LongestIncreasingSubsequence;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_D")]
pub fn dpl_1_d(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [u64; n]);
    let mut lis = LongestIncreasingSubsequence::new();
    lis.extend(a);
    writeln!(writer, "{}", lis.len()).ok();
}
