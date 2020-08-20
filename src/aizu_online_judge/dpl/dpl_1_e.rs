pub use crate::combinatorial_optimization::levenshtein_distance;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/1/DPL_1_E")]
pub fn dpl_1_e(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s1: chars, s2: chars);
    writeln!(writer, "{}", levenshtein_distance(&s1, &s2)).ok();
}
