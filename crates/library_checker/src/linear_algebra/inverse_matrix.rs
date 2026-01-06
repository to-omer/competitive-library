use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("inverse_matrix")]
pub fn inverse_matrix(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [[MInt998244353; n]; n]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    if let Some(b) = a.inverse() {
        iter_print!(writer, @it2d b.data);
    } else {
        writeln!(writer, "-1").ok();
    }
}
