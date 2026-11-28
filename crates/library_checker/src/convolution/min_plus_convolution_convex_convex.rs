use competitive::math::min_plus_convolution_convex_merge;
use competitive::prelude::*;

#[verify::library_checker(
    "min_plus_convolution_convex_convex",
    include("min_plus_convolution"),
    huge_pages
)]
pub fn min_plus_convolution_convex_convex(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [i32; n; capacity n], b: [i32; m; capacity m]);
    let c = min_plus_convolution_convex_merge(&a, &b);
    let c: Vec<_> = c.into_iter().map(i32::cast_unsigned).collect();
    pp!(@padded_u32 c);
}
