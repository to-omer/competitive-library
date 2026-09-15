use competitive::prelude::*;
use competitive::{
    math::{ConvolveSteps, MIntConvolve},
    num::mint_basic::{MInt1000000007, Modulo1000000007},
};

#[verify::library_checker("convolution_mod_1000000007")]
pub fn convolution_mod_1000000007(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    type M = MInt1000000007;
    sc!(n, m, a: [M; n], b: [M; m]);
    let c = MIntConvolve::<Modulo1000000007>::convolve(a, b);
    pp!(@it c);
}
