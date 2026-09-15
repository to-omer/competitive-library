use competitive::prelude::*;
use competitive::{
    math::{Convolve998244353, ConvolveSteps},
    num::montgomery::MInt998244353,
};

#[cfg_attr(
    not(feature = "verify_doc"),
    verify::library_checker("convolution_mod_large")
)]
pub fn convolution_mod_large(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [MInt998244353; n], b: [MInt998244353; m]);
    let c = Convolve998244353::convolve(a, b);
    pp!(@it c);
}
