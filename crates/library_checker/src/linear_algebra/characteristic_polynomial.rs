use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353};

#[verify::library_checker("characteristic_polynomial")]
pub fn characteristic_polynomial(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [[MInt998244353; n]; n]);
    let p = Matrix::<AddMulOperation<_>>::from_vec(a).characteristic_polynomial();
    pp!(@it p);
}
