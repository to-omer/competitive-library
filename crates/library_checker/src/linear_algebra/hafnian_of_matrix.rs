use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353 as M};

#[verify::library_checker("hafnian_of_matrix", include("hafnian", "MInt"))]
pub fn hafnian_of_matrix(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [[M; n]; n]);
    pp!(Matrix::<AddMulOperation<_>>::from_vec(a).hafnian());
}
