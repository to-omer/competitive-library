use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{BitwiseandConvolve, BitwiseorConvolve, ConvolveSteps},
    num::montgomery::MInt998244353 as M,
};

#[verify::library_checker("bitwise_and_convolution")]
pub fn bitwise_and_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [M; 1 << n], b: [M; 1 << n]);
    let c = BitwiseandConvolve::<AddMulOperation<_>>::convolve(a, b);
    pp!(@it c);
}

#[verify::library_checker("bitwise_and_convolution")]
pub fn bitwise_or_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, mut a: [M; 1 << n], mut b: [M; 1 << n]);
    a.reverse();
    b.reverse();
    let mut c = BitwiseorConvolve::<AddMulOperation<_>>::convolve(a, b);
    c.reverse();
    pp!(@it c);
}
