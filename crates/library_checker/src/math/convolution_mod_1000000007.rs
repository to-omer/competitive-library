use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::convolve_mint, num::mint_basic::MInt1000000007};

#[verify::verify("https://judge.yosupo.jp/problem/convolution_mod_1000000007")]
pub fn convolution_mod_1000000007(reader: impl Read, writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    type M = MInt1000000007;
    scan!(scanner, n, m, a: [M; n], b: [M; m]);
    let c = convolve_mint(&a, &b);
    echo(writer, c, ' ').ok();
}
