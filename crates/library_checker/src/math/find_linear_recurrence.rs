use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::berlekamp_massey, num::mint_basic::MInt998244353};

#[verify::verify("https://judge.yosupo.jp/problem/find_linear_recurrence")]
pub fn find_linear_recurrence(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; n]);
    let c = berlekamp_massey(&a);
    iter_print!(writer, c.len() - 1; @it c.iter().skip(1).map(|x| -x));
}
