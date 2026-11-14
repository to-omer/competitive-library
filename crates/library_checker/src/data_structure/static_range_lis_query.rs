use competitive::combinatorial_optimization::RangeLongestIncreasingSubsequence;
use competitive::prelude::*;

#[verify::library_checker("static_range_lis_query", include("RangeLongestIncreasingSubsequence"))]
pub fn static_range_lis_query(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, permutation: [usize; n], queries: [(usize, usize); iter q]);
    let sequence = RangeLongestIncreasingSubsequence::from_permutation(&permutation);
    pp!(@lf @it sequence.longest_length_batch(queries));
}
