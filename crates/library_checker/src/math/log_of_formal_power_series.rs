use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{math::FPS998244353, num::mint_basic::MInt998244353};

#[verify::verify("https://judge.yosupo.jp/problem/log_of_formal_power_series")]
pub fn log_of_formal_power_series(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [MInt998244353; n]);
    let f = FPS998244353::from_vec(a);
    let g = f.log(n);
    echo(writer, g.data, ' ').ok();
}
