use competitive::prelude::*;
use competitive::{
    math::{Convolve998244353, ConvolveSteps},
    num::montgomery::MInt998244353 as M,
};

#[verify::library_checker("convolution_mod", include("NumberTheoreticTransform"), huge_pages)]
pub fn convolution_mod(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m);
    let size = (n + m).saturating_sub(1).max(1).next_power_of_two();
    sc!(a: [M; n; capacity size], b: [M; m; capacity size]);
    let c = Convolve998244353::convolve(a, b);
    let c: Vec<_> = c.into_iter().map(M::inner).collect();
    pp!(@padded_u32 c);
}
