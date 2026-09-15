use competitive::combinatorial_optimization::largest_square;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_3_A")]
pub fn dpl_3_a(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(h, w, c: [[u8; w]; h]);
    let res = largest_square(h, w, |i, j| c[i][j] == 0);
    pp!(res);
}
