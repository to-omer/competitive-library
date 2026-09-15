use competitive::combinatorial_optimization::levenshtein_distance;
use competitive::prelude::*;

#[verify::aizu_online_judge("DPL_1_E")]
pub fn dpl_1_e(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(s1: Chars, s2: Chars);
    pp!(levenshtein_distance(&s1, &s2));
}
