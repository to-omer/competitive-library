use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::{Convolve, ConvolveSteps, MIntConvolve},
    num::mint_basic::{MInt1000000007, Modulo1000000007},
};

#[verify::verify("https://judge.yosupo.jp/problem/convolution_mod_1000000007")]
pub fn convolution_mod_1000000007(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    type M = MInt1000000007;
    scan!(scanner, n, m, a: [M; n], b: [M; m]);
    let c = MIntConvolve::<Modulo1000000007>::convolve(a, b);
    iter_print!(writer, @it c);
}
