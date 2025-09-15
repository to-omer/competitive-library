use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    math::{BlackBoxMIntMatrix, SparseMatrix},
    num::mint_basic::MInt998244353,
};

#[verify::library_checker("sparse_matrix_det")]
pub fn sparse_matrix_det(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, k, abc: [(usize, usize, MInt998244353); k]);
    let s = SparseMatrix::from_nonzero((n, n), abc);
    let ans = s.black_box_determinant();
    iter_print!(writer, ans);
}
