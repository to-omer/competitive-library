pub use crate::combinatorial_optimization::largest_rectangle;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/3/DPL_3_C")]
pub fn dpl_3_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, h: [usize; n]);
    writeln!(writer, "{}", largest_rectangle(&h)).ok();
}
