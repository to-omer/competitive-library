use competitive::algorithm::static_range_inversions;
use competitive::prelude::*;

#[verify::library_checker("static_range_inversions_query", include("static_range_inversions"))]
pub fn static_range_inversions_query(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [u32; n], queries: [(usize, usize); q]);
    pp!(@lf @it static_range_inversions(&a, &queries));
}
