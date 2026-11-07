use competitive::combinatorial_optimization::longest_common_subsequence_length;
use competitive::prelude::*;

#[verify::aizu_online_judge("ALDS1_10_C", include("longest_common_subsequence"))]
pub fn alds1_10_c(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q: usize);
    for _ in 0..q {
        sc!(x: Chars, y: Chars);
        pp!(longest_common_subsequence_length(&x, &y));
    }
}
