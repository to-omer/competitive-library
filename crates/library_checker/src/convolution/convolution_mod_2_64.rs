use competitive::math::{ConvolveSteps, U64Convolve};
use competitive::prelude::*;

#[verify::library_checker("convolution_mod_2_64")]
pub fn convolution_mod_2_64(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [u64; n], b: [u64; m]);
    let c = U64Convolve::convolve(a, b);
    iter_print!(writer, @it c);
}
