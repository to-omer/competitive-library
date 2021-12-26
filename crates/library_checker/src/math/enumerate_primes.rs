#[doc(no_inline)]
pub use competitive::math::segmented_sieve_primes;
use competitive::prelude::*;

#[cfg_attr(
    nightly,
    verify::verify("https://judge.yosupo.jp/problem/enumerate_primes")
)]
pub fn enumerate_primes(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a, b);
    let primes = segmented_sieve_primes(n);
    let iter = primes.iter().skip(b).step_by(a);
    writeln!(writer, "{} {}", primes.len(), iter.clone().len()).ok();
    for p in iter {
        write!(writer, "{} ", p).ok();
    }
    writeln!(writer).ok();
}
