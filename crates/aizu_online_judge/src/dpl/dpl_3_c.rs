#[doc(no_inline)]
pub use competitive::combinatorial_optimization::largest_rectangle;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/3/DPL_3_C")]
pub fn dpl_3_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, h: [usize; n]);
    writeln!(writer, "{}", largest_rectangle(&h)).ok();
}
