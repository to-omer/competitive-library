use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::Fps998244353, num::montgomery::MInt998244353};

#[verify::verify("https://judge.yosupo.jp/problem/multipoint_evaluation")]
pub fn multipoint_evaluation(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, c: [MInt998244353; n], p: [MInt998244353; m]);
    let f = Fps998244353::from_vec(c);
    let res = f.multipoint_evaluation(&p);
    iter_print!(writer, @it res);
}
