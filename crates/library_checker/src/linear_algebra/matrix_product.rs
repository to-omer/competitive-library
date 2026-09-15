use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353};

#[verify::library_checker("matrix_product")]
pub fn matrix_product(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, k, a: [[MInt998244353; m]; n], b: [[MInt998244353; k]; m]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = Matrix::<AddMulOperation<_>>::from_vec(b);
    let c = a * b;
    pp!(@it2d c.data);
}

#[verify::library_checker("matrix_product")]
pub fn matrix_product_strassen(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, k, a: [[MInt998244353; m]; n], b: [[MInt998244353; k]; m]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = Matrix::<AddMulOperation<_>>::from_vec(b);
    let c = a.mul_strassen(&b);
    pp!(@it2d c.data);
}
