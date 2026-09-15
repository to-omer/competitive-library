use competitive::math::{ConvolveSteps, U64Convolve};
use competitive::prelude::*;

#[verify::library_checker("convolution_mod_2_64")]
pub fn convolution_mod_2_64(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [u64; n], b: [u64; m]);
    let c = U64Convolve::convolve(a, b);
    pp!(@it c);
}
