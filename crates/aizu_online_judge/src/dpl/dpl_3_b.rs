#[doc(no_inline)]
pub use competitive::combinatorial_optimization::largest_rectangle_in_grid;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_3_B")]
pub fn dpl_3_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, h, w, c: [[u8; w]; h]);
    let res = largest_rectangle_in_grid(h, w, |i, j| c[i][j] == 0);
    writeln!(writer, "{}", res).ok();
}
