use competitive::combinatorial_optimization::LongestIncreasingSubsequence;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_D")]
pub fn dpl_1_d(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [u64; iter n]);
    let mut lis = LongestIncreasingSubsequence::new();
    lis.extend(a);
    pp!(lis.longest_length());
}
