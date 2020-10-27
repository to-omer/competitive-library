#[doc(no_inline)]
pub use competitive::num::modulus::{set_dyn_modulus, DynMInt};
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/sqrt_mod")]
pub fn sqrt_mod(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q, yp: [(u32, u32)]);
    for (y, p) in yp.take(q) {
        set_dyn_modulus(p);
        if let Some(x) = DynMInt::new_unchecked(y).sqrt() {
            writeln!(writer, "{}", x).ok();
        } else {
            writeln!(writer, "-1").ok();
        }
    }
}
