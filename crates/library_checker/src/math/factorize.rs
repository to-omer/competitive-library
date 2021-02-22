#[doc(no_inline)]
pub use competitive::math::prime_factors_rho;
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/factorize")]
pub fn factorize(reader: impl Read, mut writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    for a in scanner.iter::<u64>().take(q) {
        let x = prime_factors_rho(a);
        write!(writer, "{}", x.len()).ok();
        for x in x.into_iter() {
            write!(writer, " {}", x).ok();
        }
        writeln!(writer).ok();
    }
}
