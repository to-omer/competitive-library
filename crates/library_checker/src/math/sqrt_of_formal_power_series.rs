use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::verify("https://judge.yosupo.jp/problem/sqrt_of_formal_power_series")]
pub fn sqrt_of_formal_power_series(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; n]);
    let f = Fps998244353::from_vec(a);
    if let Some(g) = f.sqrt(n) {
        iter_print!(writer, @it g.data);
    } else {
        iter_print!(writer, "-1");
    }
}
