use competitive::math::PrimeList;
use competitive::prelude::*;

#[verify::library_checker("enumerate_primes")]
pub fn enumerate_primes(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n: u32, a, b);
    let primes = PrimeList::new(n);
    let iter = primes.primes().skip(b).step_by(a);
    writeln!(
        writer,
        "{} {}",
        primes.len(),
        primes.len().saturating_sub(b).div_ceil(a)
    )
    .ok();
    for p in iter {
        write!(writer, "{} ", p).ok();
    }
    writeln!(writer).ok();
}
