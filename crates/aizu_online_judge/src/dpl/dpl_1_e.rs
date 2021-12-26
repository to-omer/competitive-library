#[doc(no_inline)]
pub use competitive::combinatorial_optimization::levenshtein_distance;
use competitive::prelude::*;

#[cfg_attr(nightly, verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_E"))]
pub fn dpl_1_e(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s1: Chars, s2: Chars);
    writeln!(writer, "{}", levenshtein_distance(&s1, &s2)).ok();
}
