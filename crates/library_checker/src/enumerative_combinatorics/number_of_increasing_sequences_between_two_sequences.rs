use competitive::algorithm::number_of_increasing_sequences_between_998244353;
use competitive::prelude::*;

#[verify::library_checker("number_of_increasing_sequences_between_two_sequences")]
pub fn number_of_increasing_sequences_between_two_sequences(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, _m, a: [usize; n], b: [usize; n]);
    let ans = number_of_increasing_sequences_between_998244353(&a, &b);
    pp!(ans);
}
