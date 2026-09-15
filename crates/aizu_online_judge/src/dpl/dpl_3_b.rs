use competitive::combinatorial_optimization::largest_rectangle_in_grid;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_3_B")]
pub fn dpl_3_b(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(h, w, c: [[u8; w]; h]);
    let res = largest_rectangle_in_grid(h, w, |i, j| c[i][j] == 0);
    pp!(res);
}
