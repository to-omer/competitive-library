use competitive::math::min_plus_convolution_convex_divide_and_conquer;
use competitive::prelude::*;

#[verify::library_checker(
    "min_plus_convolution_convex_arbitrary",
    include("min_plus_convolution"),
    huge_pages
)]
pub fn min_plus_convolution_convex_arbitrary(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [i32; n; capacity n], b: [i32; m; capacity m]);
    let c = min_plus_convolution_convex_divide_and_conquer(&a, &b);
    let c: Vec<_> = c.into_iter().map(i32::cast_unsigned).collect();
    pp!(@padded_u32 c);
}
