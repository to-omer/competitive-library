use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{ConvolveSteps, GcdConvolve},
    num::montgomery::MInt998244353 as M,
};

#[verify::library_checker("gcd_convolution")]
pub fn gcd_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, mut a: [M; n], mut b: [M; n]);
    a.insert(0, Default::default());
    b.insert(0, Default::default());
    let c = GcdConvolve::<AddMulOperation<_>>::convolve(a, b);
    pp!(@it &c[1..]);
}
