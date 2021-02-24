use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::Ntt998244353,
    num::{mint_basic::MInt998244353, MInt},
};

#[verify::verify("https://judge.yosupo.jp/problem/convolution_mod")]
pub fn convolution_mod(reader: impl Read, writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [MInt998244353; n], b: [MInt998244353; m]);
    let c = Ntt998244353::convolve(a, b);
    echo(writer, c, ' ').ok();
}
