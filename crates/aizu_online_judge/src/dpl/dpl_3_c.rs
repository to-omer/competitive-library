use competitive::combinatorial_optimization::largest_rectangle;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_3_C")]
pub fn dpl_3_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, h: [usize; n]);
    pp!(largest_rectangle(&h));
}
