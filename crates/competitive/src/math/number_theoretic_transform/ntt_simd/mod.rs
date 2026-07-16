use super::*;
use std::arch::x86_64::*;

const LAZY_THRESHOLD: u32 = 1 << 30;

mod convolution_avx2;
mod ntt_avx2;
mod ntt_avx512;

pub(super) use convolution_avx2::convolve_blocks_avx2;
pub(super) use ntt_avx2::{intt_avx2, ntt_avx2, pointwise_multiply_avx2};
pub(super) use ntt_avx512::{intt_avx512, ntt_avx512, pointwise_multiply_avx512};
