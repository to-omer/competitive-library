use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("matrix_det")]
pub fn matrix_det(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [[MInt998244353; n]; n]);
    let mut a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let det = a.determinant();
    writeln!(writer, "{}", det).ok();
}
