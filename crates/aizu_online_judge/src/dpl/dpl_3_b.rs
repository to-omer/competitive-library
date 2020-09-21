pub use competitive::combinatorial_optimization::largest_rectangle_in_grid;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/7/DPL/3/DPL_3_B")]
pub fn dpl_3_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, h, w, c: [[u8; w]; h]);
    let res = largest_rectangle_in_grid(h, w, |i, j| c[i][j] == 0);
    writeln!(writer, "{}", res).ok();
}
