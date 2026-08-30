use super::*;
use std::arch::x86_64::*;

const LAZY_THRESHOLD: u32 = 1 << 30;

#[inline]
fn normalize_scalar<M>(x: u32) -> u32
where
    M: Montgomery32NttModulus,
{
    if x >= M::MOD { x - M::MOD } else { x }
}

mod convolution_avx2;
mod ntt_avx2;
mod ntt_avx512;

pub use convolution_avx2::{
    convolve_blocks_avx2, inverse_transform_blocks_avx2, multiply_blocks_avx2,
    transform_blocks_avx2,
};
pub use ntt_avx2::{
    intt_avx2, intt_batch_avx2, ntt_avx2, ntt_batch_avx2, pointwise_multiply_add_avx2,
    pointwise_multiply_avx2,
};
pub use ntt_avx512::{
    intt_avx512, intt_batch_avx512, ntt_avx512, ntt_batch_avx512, pointwise_multiply_add_avx512,
    pointwise_multiply_avx512,
};
