use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{BitwisexorConvolve, ConvolveSteps},
    num::montgomery::MInt998244353 as M,
};

#[verify::library_checker("bitwise_xor_convolution")]
pub fn bitwise_xor_convolution(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [M; 1 << n], b: [M; 1 << n]);
    let c = BitwisexorConvolve::<AddMulOperation<_>>::convolve(a, b);
    pp!(@it c);
}
