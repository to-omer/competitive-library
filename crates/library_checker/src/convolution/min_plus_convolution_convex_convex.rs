use competitive::math::min_plus_convolution_convex_merge;
use competitive::prelude::*;

#[verify::library_checker("min_plus_convolution_convex_convex")]
pub fn min_plus_convolution_convex_convex(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [i64; n], b: [i64; m]);
    pp!(@it min_plus_convolution_convex_merge(&a, &b));
}
