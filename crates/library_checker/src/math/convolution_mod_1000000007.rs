use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::convolve3, num::mint_basic::Modulo1000000007};

#[verify::verify("https://judge.yosupo.jp/problem/convolution_mod_1000000007")]
pub fn convolution_mod_1000000007(reader: impl Read, writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [u32; n], b: [u32; m]);
    let c = convolve3::<Modulo1000000007, _>(a, b);
    echo(writer, c, ' ').ok();
}
