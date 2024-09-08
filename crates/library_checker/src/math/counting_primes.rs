use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, math::QuotientArray};

#[verify::library_checker("counting_primes")]
pub fn counting_primes(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n: u64);
    let qa = QuotientArray::from_fn(n, |i| i as i64 - 1).lucy_dp::<AdditiveOperation<_>>(|x, _p| x);
    writeln!(writer, "{}", qa[n]).ok();
}
