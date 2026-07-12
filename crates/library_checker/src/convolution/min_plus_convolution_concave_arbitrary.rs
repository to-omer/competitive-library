use competitive::math::min_plus_convolution_concave_envelope;
use competitive::prelude::*;

#[verify::library_checker("min_plus_convolution_concave_arbitrary")]
pub fn min_plus_convolution_concave_arbitrary(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [i64; n], b: [i64; m]);
    iter_print!(writer, @it min_plus_convolution_concave_envelope(&a, &b));
}
