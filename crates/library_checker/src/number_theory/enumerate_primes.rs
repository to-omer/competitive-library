use competitive::math::PrimeList;
use competitive::prelude::*;

#[verify::library_checker("enumerate_primes")]
pub fn enumerate_primes(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n: u32, a, b);
    let primes = PrimeList::new(n);
    let iter = primes.primes().skip(b).step_by(a);
    pp!(primes.len(), primes.len().saturating_sub(b).div_ceil(a); @it iter);
}
