use competitive::algorithm;
use competitive::prelude::*;

#[verify::library_checker("static_range_count_distinct")]
pub fn static_range_count_distinct(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, a: [u32; n], queries: [(usize, usize); q]);
    pp!(@lf @it algorithm::static_range_count_distinct(&a, &queries));
}
