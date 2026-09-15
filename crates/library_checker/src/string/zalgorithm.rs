use competitive::prelude::*;
use competitive::string::{Mersenne61x1, RollingHasher, Zarray};

#[verify::library_checker("zalgorithm")]
pub fn zalgorithm(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(s: Chars);
    let z = Zarray::new(&s);
    pp!(@it (0..s.len()).map(|i| z[i]));
}

#[verify::library_checker("zalgorithm")]
pub fn zalgorithm_rolling_hash(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(s: Bytes);
    Mersenne61x1::init(s.len());
    let h = Mersenne61x1::hash_sequence(s.iter().map(|&c| c as _));
    let ans = (0..s.len()).map(|i| h.range(..).longest_common_prefix(&h.range(i..)));
    pp!(@it ans);
}
