use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{ConvolveSteps, SubsetConvolve},
    num::mint_basic::MInt998244353,
};

#[verify::library_checker("subset_convolution")]
pub fn subset_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [MInt998244353; 1 << n], b: [MInt998244353; 1 << n]);
    let c = SubsetConvolve::<AddMulOperation<_>>::convolve(a, b);
    pp!(@it c);
}
