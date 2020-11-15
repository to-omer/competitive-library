#[doc(no_inline)]
pub use competitive::num::mint_basic::{DynMIntU32, DynModuloU32};
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/sqrt_mod")]
pub fn sqrt_mod(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q, yp: [(u32, u32)]);
    for (y, p) in yp.take(q) {
        DynModuloU32::set_mod(p);
        if let Some(x) = DynMIntU32::from(y).sqrt() {
            writeln!(writer, "{}", x).ok();
        } else {
            writeln!(writer, "-1").ok();
        }
    }
}
