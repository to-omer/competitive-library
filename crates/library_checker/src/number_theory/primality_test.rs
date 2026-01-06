#[doc(no_inline)]
pub use competitive::math::miller_rabin;
use competitive::prelude::*;

#[verify::library_checker("primality_test")]
pub fn primality_test(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q);
    for _ in 0..q {
        scan!(scanner, n: u64);
        let ans = if miller_rabin(n) { "Yes" } else { "No" };
        writeln!(writer, "{}", ans).ok();
    }
}
