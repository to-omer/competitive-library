use competitive::math::count_primes;
use competitive::prelude::*;

#[verify::library_checker("counting_primes", include("count_primes"))]
pub fn counting_primes(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n: u64);
    pp!(count_primes(n));
}
