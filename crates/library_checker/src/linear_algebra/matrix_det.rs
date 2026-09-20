use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353};

#[verify::library_checker("matrix_det")]
pub fn matrix_det(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [[MInt998244353; n]; n]);
    let mut a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let det = a.determinant();
    pp!(det);
}
