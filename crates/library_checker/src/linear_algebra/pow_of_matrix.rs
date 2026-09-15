use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{MIntMatrix, Matrix},
    num::mint_basic::MInt998244353,
};

#[verify::library_checker("pow_of_matrix")]
pub fn pow_of_matrix(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, k, a: [[MInt998244353; n]; n]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = a.pow_frobenius(k);
    pp!(@it2d b.data);
}

#[verify::library_checker("pow_of_matrix")]
pub fn pow_of_matrix_strassen(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, k, a: [[MInt998244353; n]; n]);
    let a = Matrix::<AddMulOperation<_>>::from_vec(a);
    let b = a.pow_strassen(k);
    pp!(@it2d b.data);
}
