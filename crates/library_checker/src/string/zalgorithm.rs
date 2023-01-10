use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::{Mersenne61x1, RollingHasher, Zarray};

#[verify::library_checker("zalgorithm")]
pub fn zalgorithm(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let z = Zarray::new(&s);
    iter_print!(writer, @it (0..s.len()).map(|i| z[i]));
}

#[verify::library_checker("zalgorithm")]
pub fn zalgorithm_rolling_hash(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Bytes);
    Mersenne61x1::init_with_time(s.len());
    let h = Mersenne61x1::hash_sequence(s.iter().map(|&c| c as _));
    let ans = (0..s.len()).map(|i| h.range(..).longest_common_prefix(&h.range(i..)));
    iter_print!(writer, @it ans);
}
