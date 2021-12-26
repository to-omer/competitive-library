#[doc(no_inline)]
pub use competitive::math::floor_sum;
use competitive::prelude::*;

#[cfg_attr(nightly, verify::verify("https://judge.yosupo.jp/problem/sum_of_floor_of_linear"))]
pub fn sum_of_floor_of_linear(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, query: [(u64, u64, u64, u64)]);
    for (n, m, a, b) in query.take(t) {
        writeln!(writer, "{}", floor_sum(n, m, a, b)).ok();
    }
}
