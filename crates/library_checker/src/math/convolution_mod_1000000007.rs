use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::convolve3, num::modulus::Modulo1000000007};

#[verify::verify("https://judge.yosupo.jp/problem/convolution_mod_1000000007")]
pub fn convolution_mod_1000000007(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [u64; n], b: [u64; m]);
    let c = convolve3::<Modulo1000000007>(a, b);
    echo(writer, c, ' ').ok();
}
