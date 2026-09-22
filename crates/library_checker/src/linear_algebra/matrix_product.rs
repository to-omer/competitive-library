use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353 as M};

#[verify::library_checker("matrix_product")]
pub fn matrix_product(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, k, a: [[M; m]; n], b: [[M; k]; m]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = Matrix::<AddMulOperation<_>>::from_vec(b);
    let c = a * b;
    pp!(@it2d c.data);
}

#[verify::library_checker("matrix_product")]
pub fn matrix_product_strassen(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, k, a: [[M; m]; n], b: [[M; k]; m]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = Matrix::<AddMulOperation<_>>::from_vec(b);
    let c = a.mul_strassen(&b);
    pp!(@it2d c.data);
}
