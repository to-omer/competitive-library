use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{ConvolveSteps, LcmConvolve},
    num::montgomery::MInt998244353 as M,
};

#[verify::library_checker("lcm_convolution")]
pub fn lcm_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, mut a: [M; n], mut b: [M; n]);
    a.insert(0, Default::default());
    b.insert(0, Default::default());
    let c = LcmConvolve::<AddMulOperation<_>>::convolve(a, b);
    pp!(@it &c[1..]);
}
