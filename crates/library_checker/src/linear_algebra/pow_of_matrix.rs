use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("pow_of_matrix")]
pub fn pow_of_matrix(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, k, a: [[MInt998244353; n]; n]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = a.pow(k);
    iter_print!(writer, @it2d b.data);
}

#[verify::library_checker("pow_of_matrix")]
pub fn pow_of_matrix_strassen(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, k, a: [[MInt998244353; n]; n]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = a.pow_strassen(k);
    iter_print!(writer, @it2d b.data);
}
