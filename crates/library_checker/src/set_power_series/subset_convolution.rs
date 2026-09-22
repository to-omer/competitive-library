use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{ConvolveSteps, SubsetConvolve},
    num::mint_basic::MInt998244353 as M,
};

#[verify::library_checker("subset_convolution")]
pub fn subset_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [M; 1 << n], b: [M; 1 << n]);
    let c = SubsetConvolve::<AddMulOperation<_>>::convolve(a, b);
    pp!(@it c);
}
