#[doc(no_inline)]
pub use competitive::math::discrete_logarithm;
use competitive::prelude::*;

#[verify::library_checker("discrete_logarithm_mod")]
pub fn discrete_logarithm_mod(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, query: [(u64, u64, u64)]);
    for (x, y, m) in query.take(t) {
        let ans = discrete_logarithm(x, y, m).map(|k| k as i64).unwrap_or(-1);
        writeln!(writer, "{}", ans).ok();
    }
}
