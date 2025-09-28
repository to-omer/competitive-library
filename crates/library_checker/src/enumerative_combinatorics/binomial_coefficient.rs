#[doc(no_inline)]
pub use competitive::math::ArbitraryModBinomial;
use competitive::prelude::*;

#[verify::library_checker("binomial_coefficient")]
pub fn binomial_coefficient(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, m: u64);
    let binom = ArbitraryModBinomial::new(m, !0);
    for _ in 0..t {
        scan!(scanner, n: u64, k: u64);
        iter_print!(writer, binom.combination(n, k));
    }
}
