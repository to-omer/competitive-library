pub use crate::math::segmented_sieve_primes;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/enumerate_primes")]
pub fn enumerate_primes(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a, b);
    let primes = segmented_sieve_primes(n);
    let iter = primes.iter().skip(b).step_by(a);
    writeln!(writer, "{} {}", primes.len(), iter.clone().len())?;
    for p in iter {
        write!(writer, "{} ", p)?;
    }
    writeln!(writer)?;

    Ok(())
}
