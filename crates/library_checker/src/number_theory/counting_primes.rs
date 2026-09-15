use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, math::QuotientArray};

#[verify::library_checker("counting_primes")]
pub fn counting_primes(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n: u64);
    let qa = QuotientArray::from_fn(n, |i| i as i64 - 1).lucy_dp::<AdditiveOperation<_>>(|x, _p| x);
    pp!(qa[n]);
}
