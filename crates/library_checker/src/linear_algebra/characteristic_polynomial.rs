use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353};

#[verify::library_checker("characteristic_polynomial")]
pub fn characteristic_polynomial(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [[MInt998244353; n]; n]);
    let p = Matrix::<AddMulOperation<_>>::from_vec(a).characteristic_polynomial();
    iter_print!(writer, @it p);
}
