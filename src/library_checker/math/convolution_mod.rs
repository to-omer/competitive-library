pub use crate::math::NTT998244353;
pub use crate::num::{modulus::MInt998244353, MInt};
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/convolution_mod")]
pub fn convolution_mod(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [MInt998244353; n], b: [MInt998244353; m]);
    let c = NTT998244353::convolve(a, b);
    echo(writer, c, ' ').ok();
}
