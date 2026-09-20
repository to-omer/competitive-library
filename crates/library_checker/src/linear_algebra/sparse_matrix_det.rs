use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::SparseMatrix, num::mint_basic::MInt998244353};

#[verify::library_checker("sparse_matrix_det")]
pub fn sparse_matrix_det(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, k, abc: [(usize, usize, MInt998244353); k]);
    let s = SparseMatrix::<AddMulOperation<_>>::from_nonzero((n, n), abc);
    let ans = s.determinant();
    pp!(ans);
}
