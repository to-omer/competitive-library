use competitive::math::ArbitraryModBinomial;
use competitive::prelude::*;

#[verify::library_checker("binomial_coefficient")]
pub fn binomial_coefficient(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, m: u64);
    let binom = ArbitraryModBinomial::new(m, !0);
    for _ in 0..t {
        sc!(n: u64, k: u64);
        pp!(binom.combination(n, k));
    }
}
