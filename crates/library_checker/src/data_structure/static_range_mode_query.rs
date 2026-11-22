use competitive::algorithm::static_range_mode;
use competitive::prelude::*;

#[verify::library_checker("static_range_mode_query", include("static_range_mode"))]
pub fn static_range_mode_query(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, a: [u32; n], queries: [(usize, usize); q]);
    pp!(@ittup static_range_mode(&a, &queries));
}
