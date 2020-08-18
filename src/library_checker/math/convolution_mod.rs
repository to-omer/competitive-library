pub use crate::math::NumberTheoreticTransform;
pub use crate::num::{modulus::Modulo998244353, MInt};
use crate::tools::{read_all, Scanner};
use crate::{echo, scan};
use std::io::{Read, Write};

type M = MInt<Modulo998244353>;
type NTT = NumberTheoreticTransform<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/convolution_mod")]
pub fn convolution_mod(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [M; n], b: [M; m]);
    let c = NTT::convolve(a, b);
    echo!(writer, c, " ");
}
