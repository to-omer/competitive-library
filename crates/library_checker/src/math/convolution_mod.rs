use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::{Convolve998244353, ConvolveSteps},
    num::{montgomery::MInt998244353, MInt},
};

#[verify::verify("https://judge.yosupo.jp/problem/convolution_mod")]
pub fn convolution_mod(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [MInt998244353; n], b: [MInt998244353; m]);
    let c = Convolve998244353::convolve(a, b);
    iter_print!(writer, @it c);
}
