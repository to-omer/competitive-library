use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("matrix_rank")]
pub fn matrix_rank(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [[MInt998244353; m]; n]);
    let mut a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let rank = a.rank();
    pp!(rank);
}
