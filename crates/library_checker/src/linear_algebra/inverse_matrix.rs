use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353 as M};

#[verify::library_checker("inverse_matrix")]
pub fn inverse_matrix(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [[M; n]; n]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    if let Some(b) = a.inverse() {
        pp!(@it2d b.data);
    } else {
        pp!("-1");
    }
}
