use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("matrix_product")]
pub fn matrix_product(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, k, a: [[MInt998244353; m]; n], b: [[MInt998244353; k]; m]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = Matrix::<AddMulOperation<_>>::from_vec(b);
    let c = a * b;
    iter_print!(writer, @it2d c.data);
}
