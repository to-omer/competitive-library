#![allow(unsafe_op_in_unsafe_fn)]

use super::{
    AssociatedValue, MInt, MIntConvert,
    fast_fourier_transform::{RotateCache, simd::*},
};
use std::arch::x86_64::*;

#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn round4(value: &[f64; 4]) -> [i64; 4] {
    let magic = _mm256_set1_pd((3i64 << 51) as f64);
    let rounded = _mm256_sub_epi64(
        _mm256_castpd_si256(_mm256_add_pd(_mm256_load_pd(value.as_ptr()), magic)),
        _mm256_castpd_si256(magic),
    );
    let mut result = [0; 4];
    _mm256_storeu_si256(result.as_mut_ptr().cast(), rounded);
    result
}

#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn reduce_mod4(value: __m256d, modulus: __m256d, inverse: __m256d) -> __m256d {
    let quotient = _mm256_round_pd::<{ _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC }>(
        _mm256_mul_pd(value, inverse),
    );
    _mm256_fnmadd_pd(quotient, modulus, value)
}

#[target_feature(enable = "avx2,fma")]
unsafe fn split_coefficients<M>(
    values: Vec<MInt<M>>,
    n: usize,
    modulus: i64,
    split: i64,
) -> (Vec<Complex4>, Vec<Complex4>)
where
    M: MIntConvert + MIntConvert<u32>,
{
    let mut low = vec![Complex4::default(); n / 4];
    let mut high = low.clone();
    let divisor = _mm256_set1_pd(split as f64);
    let split = _mm256_set1_pd(split as f64);
    for i in (0..values.len()).step_by(4) {
        let mut centered = [0i32; 4];
        for lane in 0..4.min(values.len() - i) {
            let mut value = u32::from(values[i + lane]) as i64;
            if value * 2 > modulus {
                value -= modulus;
            }
            centered[lane] = value as i32;
        }
        let value = _mm256_cvtepi32_pd(_mm_loadu_si128(centered.as_ptr().cast()));
        let upper = _mm256_round_pd::<{ _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC }>(
            _mm256_div_pd(value, divisor),
        );
        let lower = _mm256_fnmadd_pd(upper, split, value);
        if i < n {
            _mm256_store_pd(low[i >> 2].re.as_mut_ptr(), lower);
            _mm256_store_pd(high[i >> 2].re.as_mut_ptr(), upper);
        } else {
            _mm256_store_pd(low[(i - n) >> 2].im.as_mut_ptr(), lower);
            _mm256_store_pd(high[(i - n) >> 2].im.as_mut_ptr(), upper);
        }
    }
    (low, high)
}

#[target_feature(enable = "avx2,fma")]
unsafe fn dot_soa(a0: &mut [Complex4], a1: &mut [Complex4], b0: &mut [Complex4], b1: &[Complex4]) {
    let n = a0.len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        for i in 0..a0.len() {
            let (mut cr, mut ci) = load4(&b0[i]);
            let (mut dr, mut di) = load4(&b1[i]);
            let mut c0r = _mm256_setzero_pd();
            let mut c0i = _mm256_setzero_pd();
            let mut c1r = _mm256_setzero_pd();
            let mut c1i = _mm256_setzero_pd();
            let mut c2r = _mm256_setzero_pd();
            let mut c2i = _mm256_setzero_pd();
            let w = eval_twiddle(cache, 1, a0.len(), i);
            let wr = _mm256_setr_pd(w.re, 1.0, 1.0, 1.0);
            let wi = _mm256_setr_pd(w.im, 0.0, 0.0, 0.0);
            for lane in 0..4 {
                let ar = _mm256_set1_pd(a0[i].re[lane]);
                let ai = _mm256_set1_pd(a0[i].im[lane]);
                let br = _mm256_set1_pd(a1[i].re[lane]);
                let bi = _mm256_set1_pd(a1[i].im[lane]);
                multiply_accumulate4(&mut c0r, &mut c0i, ar, ai, cr, ci);
                multiply_accumulate4(&mut c1r, &mut c1i, ar, ai, dr, di);
                multiply_accumulate4(&mut c1r, &mut c1i, br, bi, cr, ci);
                multiply_accumulate4(&mut c2r, &mut c2i, br, bi, dr, di);
                if lane != 3 {
                    cr = _mm256_permute4x64_pd::<0x93>(cr);
                    ci = _mm256_permute4x64_pd::<0x93>(ci);
                    dr = _mm256_permute4x64_pd::<0x93>(dr);
                    di = _mm256_permute4x64_pd::<0x93>(di);
                    (cr, ci) = mul4(cr, ci, wr, wi);
                    (dr, di) = mul4(dr, di, wr, wi);
                }
            }
            store4(&mut a0[i], c0r, c0i);
            store4(&mut a1[i], c1r, c1i);
            store4(&mut b0[i], c2r, c2i);
        }
    });
}

#[target_feature(enable = "avx2,fma")]
unsafe fn split_u64_coefficients(values: &[u64], n: usize) -> [Vec<Complex4>; 5] {
    let mut result = std::array::from_fn(|_| vec![Complex4::default(); n / 4]);
    for (i, mut value) in values.iter().copied().enumerate() {
        let (i, imag) = if i < n { (i, false) } else { (i - n, true) };
        for part in &mut result {
            let digit = ((value << 51) as i64) >> 51;
            value = (value >> 13).wrapping_add(u64::from(digit < 0));
            if imag {
                part[i >> 2].im[i & 3] = digit as f64;
            } else {
                part[i >> 2].re[i & 3] = digit as f64;
            }
        }
    }
    result
}

#[target_feature(enable = "avx2,fma")]
unsafe fn dot_u64_soa(a: &mut [Vec<Complex4>; 5], b: &[Vec<Complex4>; 5]) {
    let n = a[0].len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        for block in 0..a[0].len() {
            let mut br = [_mm256_setzero_pd(); 5];
            let mut bi = br;
            let mut rr = br;
            let mut ri = br;
            for part in 0..5 {
                (br[part], bi[part]) = load4(&b[part][block]);
            }
            let w = eval_twiddle(cache, 1, a[0].len(), block);
            let wr = _mm256_setr_pd(w.re, 1.0, 1.0, 1.0);
            let wi = _mm256_setr_pd(w.im, 0.0, 0.0, 0.0);
            for lane in 0..4 {
                let ar: [__m256d; 5] =
                    std::array::from_fn(|part| _mm256_set1_pd(a[part][block].re[lane]));
                let ai: [__m256d; 5] =
                    std::array::from_fn(|part| _mm256_set1_pd(a[part][block].im[lane]));
                for part in 0..5 {
                    for left in 0..=part {
                        multiply_accumulate4(
                            &mut rr[part],
                            &mut ri[part],
                            ar[left],
                            ai[left],
                            br[part - left],
                            bi[part - left],
                        );
                    }
                }
                if lane != 3 {
                    for part in 0..5 {
                        br[part] = _mm256_permute4x64_pd::<0x93>(br[part]);
                        bi[part] = _mm256_permute4x64_pd::<0x93>(bi[part]);
                        (br[part], bi[part]) = mul4(br[part], bi[part], wr, wi);
                    }
                }
            }
            for part in 0..5 {
                store4(&mut a[part][block], rr[part], ri[part]);
            }
        }
    });
}

#[target_feature(enable = "avx2,fma")]
pub unsafe fn convolve_u64_avx2(a: Vec<u64>, b: Vec<u64>) -> Vec<u64> {
    let len = a.len() + b.len() - 1;
    let n = len.next_power_of_two() / 2;
    let a_parts = if a.iter().all(|&value| value <= u32::MAX as u64) {
        3
    } else {
        5
    };
    let mut fa = split_u64_coefficients(&a, n);
    drop(a);
    let b_parts = if b.iter().all(|&value| value <= u32::MAX as u64) {
        3
    } else {
        5
    };
    let mut fb = split_u64_coefficients(&b, n);
    drop(b);
    for part in 0..3 {
        fft_soa(&mut fa[part]);
        fft_soa(&mut fb[part]);
    }
    for part in &mut fa[3..a_parts] {
        fft_soa(part);
    }
    for part in &mut fb[3..b_parts] {
        fft_soa(part);
    }
    dot_u64_soa(&mut fa, &fb);
    drop(fb);
    for part in &mut fa {
        ifft_soa(part);
    }
    let mut result = vec![0; len];
    for (block, _) in fa[0].iter().enumerate() {
        let real: [[i64; 4]; 5] = std::array::from_fn(|part| round4(&fa[part][block].re));
        let imag: [[i64; 4]; 5] = std::array::from_fn(|part| round4(&fa[part][block].im));
        for lane in 0..4 {
            let i = block * 4 + lane;
            if i < len {
                result[i] = (real[0][lane] as u64)
                    .wrapping_add((real[1][lane] as u64) << 13)
                    .wrapping_add((real[2][lane] as u64) << 26)
                    .wrapping_add((real[3][lane] as u64) << 39)
                    .wrapping_add((real[4][lane] as u64) << 52);
            }
            if i + n < len {
                result[i + n] = (imag[0][lane] as u64)
                    .wrapping_add((imag[1][lane] as u64) << 13)
                    .wrapping_add((imag[2][lane] as u64) << 26)
                    .wrapping_add((imag[3][lane] as u64) << 39)
                    .wrapping_add((imag[4][lane] as u64) << 52);
            }
        }
    }
    result
}

#[target_feature(enable = "avx2,fma")]
pub unsafe fn convolve_mint_avx2<M>(a: Vec<MInt<M>>, b: Vec<MInt<M>>) -> Vec<MInt<M>>
where
    M: MIntConvert + MIntConvert<u32>,
{
    let len = a.len() + b.len() - 1;
    let n = len.next_power_of_two() / 2;
    let modulus = <M as MIntConvert<u32>>::mod_into() as i64;
    let split = (modulus as f64).sqrt() as i64 + 1;
    let (mut a0, mut a1) = split_coefficients(a, n, modulus, split);
    let (mut b0, mut b1) = split_coefficients(b, n, modulus, split);
    fft_soa(&mut a0);
    fft_soa(&mut a1);
    fft_soa(&mut b0);
    fft_soa(&mut b1);
    dot_soa(&mut a0, &mut a1, &mut b0, &b1);
    drop(b1);
    ifft_soa(&mut a0);
    ifft_soa(&mut a1);
    ifft_soa(&mut b0);
    let split2 = (split * split % modulus) as f64;
    let split = _mm256_set1_pd(split as f64);
    let split2 = _mm256_set1_pd(split2);
    let inverse = _mm256_set1_pd(1.0 / modulus as f64);
    let modulus = _mm256_set1_pd(modulus as f64);
    let magic = _mm256_set1_pd((3i64 << 51) as f64);
    let mut result = vec![MInt::<M>::from(0u32); len];
    for (block, ((a0, a1), b0)) in a0.iter().zip(&a1).zip(&b0).enumerate() {
        for (part, (a0, a1, b0)) in [(&a0.re, &a1.re, &b0.re), (&a0.im, &a1.im, &b0.im)]
            .into_iter()
            .enumerate()
        {
            let a0 = _mm256_round_pd::<{ _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC }>(
                _mm256_load_pd(a0.as_ptr()),
            );
            let a1 = _mm256_round_pd::<{ _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC }>(
                _mm256_load_pd(a1.as_ptr()),
            );
            let b0 = _mm256_round_pd::<{ _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC }>(
                _mm256_load_pd(b0.as_ptr()),
            );
            let a0 = reduce_mod4(a0, modulus, inverse);
            let a1 = reduce_mod4(a1, modulus, inverse);
            let b0 = reduce_mod4(b0, modulus, inverse);
            let value = _mm256_fmadd_pd(b0, split2, _mm256_fmadd_pd(a1, split, a0));
            let value = reduce_mod4(value, modulus, inverse);
            let value = _mm256_add_pd(
                value,
                _mm256_and_pd(
                    _mm256_cmp_pd::<_CMP_LT_OQ>(value, _mm256_setzero_pd()),
                    modulus,
                ),
            );
            let value = _mm256_sub_epi64(
                _mm256_castpd_si256(_mm256_add_pd(value, magic)),
                _mm256_castpd_si256(magic),
            );
            let mut lanes = [0i64; 4];
            _mm256_storeu_si256(lanes.as_mut_ptr().cast(), value);
            for (lane, value) in lanes.into_iter().enumerate() {
                let i = block * 4 + lane + part * n;
                if i < len {
                    result[i] = MInt::<M>::from(value as u32);
                }
            }
        }
    }
    result
}
