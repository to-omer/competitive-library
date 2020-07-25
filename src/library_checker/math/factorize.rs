pub use crate::math::prime_factors_rho;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/factorize")]
pub fn factorize(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    for a in scanner.iter::<u64>().take(q) {
        let x = prime_factors_rho(a);
        write!(writer, "{}", x.len())?;
        for x in x.into_iter() {
            write!(writer, " {}", x)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}
