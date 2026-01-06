use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("matrix_rank")]
pub fn matrix_rank(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [[MInt998244353; m]; n]);
    let mut a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let rank = a.rank();
    writeln!(writer, "{}", rank).ok();
}
