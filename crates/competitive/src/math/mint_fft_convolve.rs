#![allow(unsafe_op_in_unsafe_fn)]

use super::{
    AssociatedValue, Complex, ConvolveRealFft, MInt, MIntConvert,
    fast_fourier_transform::{RotateCache, middle_product_f64_scalar},
};
use std::arch::x86_64::*;
#[derive(Clone, Copy, Default)]
#[repr(C, align(32))]
struct Complex4 {
    re: [f64; 4],
    im: [f64; 4],
}

#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn load4(value: &Complex4) -> (__m256d, __m256d) {
    (
        _mm256_load_pd(value.re.as_ptr()),
        _mm256_load_pd(value.im.as_ptr()),
    )
}

#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn store4(value: &mut Complex4, re: __m256d, im: __m256d) {
    _mm256_store_pd(value.re.as_mut_ptr(), re);
    _mm256_store_pd(value.im.as_mut_ptr(), im);
}

#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn mul4(ar: __m256d, ai: __m256d, br: __m256d, bi: __m256d) -> (__m256d, __m256d) {
    (
        _mm256_fmsub_pd(ar, br, _mm256_mul_pd(ai, bi)),
        _mm256_fmadd_pd(ai, br, _mm256_mul_pd(ar, bi)),
    )
}

#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn multiply_accumulate4(
    rr: &mut __m256d,
    ri: &mut __m256d,
    ar: __m256d,
    ai: __m256d,
    br: __m256d,
    bi: __m256d,
) {
    *rr = _mm256_fmadd_pd(ar, br, *rr);
    *rr = _mm256_fnmadd_pd(ai, bi, *rr);
    *ri = _mm256_fmadd_pd(ai, br, *ri);
    *ri = _mm256_fmadd_pd(ar, bi, *ri);
}

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

#[inline]
fn eval_twiddle(cache: &[Complex<f64>], step: usize, n: usize, k: usize) -> Complex<f64> {
    let k = step * k;
    let w = cache[(k >> 2) << 1].conjugate();
    let w = match k & 3 {
        0 => w,
        1 => Complex::new(-w.re, -w.im),
        2 => Complex::new(-w.im, w.re),
        _ => Complex::new(w.im, -w.re),
    };
    cache[step * n].conjugate() * w
}

#[target_feature(enable = "avx2,fma")]
unsafe fn fft_soa(a: &mut [Complex4]) {
    let n = a.len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let parity = n.trailing_zeros() & 1;
        for leaf in (0..n).step_by(16) {
            let mut level = (n + leaf).trailing_zeros();
            level -= u32::from(level & 1 != parity);
            while level >= 4 {
                let len = 1usize << level;
                let q = leaf >> level;
                let width = len / 16;
                let start = q * width * 4;
                let (a, rest) = a[start..start + width * 4].split_at_mut(width);
                let (b, rest) = rest.split_at_mut(width);
                let (c, d) = rest.split_at_mut(width);
                let w1 = eval_twiddle(cache, 4, n >> level, q);
                let w2 = w1 * w1;
                let w3 = w1 * w2;
                let (w1r, w1i) = (_mm256_set1_pd(w1.re), _mm256_set1_pd(w1.im));
                let (w2r, w2i) = (_mm256_set1_pd(w2.re), _mm256_set1_pd(w2.im));
                let (w3r, w3i) = (_mm256_set1_pd(w3.re), _mm256_set1_pd(w3.im));
                for i in 0..width {
                    let (ar, ai) = load4(&a[i]);
                    let (br, bi) = load4(&b[i]);
                    let (cr, ci) = load4(&c[i]);
                    let (dr, di) = load4(&d[i]);
                    let (br, bi) = mul4(br, bi, w1r, w1i);
                    let (cr, ci) = mul4(cr, ci, w2r, w2i);
                    let (dr, di) = mul4(dr, di, w3r, w3i);
                    let acr = _mm256_add_pd(ar, cr);
                    let aci = _mm256_add_pd(ai, ci);
                    let bdr = _mm256_add_pd(br, dr);
                    let bdi = _mm256_add_pd(bi, di);
                    let acd_r = _mm256_sub_pd(ar, cr);
                    let acd_i = _mm256_sub_pd(ai, ci);
                    let bdd_r = _mm256_sub_pd(br, dr);
                    let bdd_i = _mm256_sub_pd(bi, di);
                    store4(&mut a[i], _mm256_add_pd(acr, bdr), _mm256_add_pd(aci, bdi));
                    store4(&mut b[i], _mm256_sub_pd(acr, bdr), _mm256_sub_pd(aci, bdi));
                    store4(
                        &mut c[i],
                        _mm256_sub_pd(acd_r, bdd_i),
                        _mm256_add_pd(acd_i, bdd_r),
                    );
                    store4(
                        &mut d[i],
                        _mm256_add_pd(acd_r, bdd_i),
                        _mm256_sub_pd(acd_i, bdd_r),
                    );
                }
                level -= 2;
            }
        }
        if parity != 0 {
            let blocks = n / 8;
            for k in 0..blocks {
                let w = eval_twiddle(cache, 2, blocks, k);
                let wr = _mm256_set1_pd(w.re);
                let wi = _mm256_set1_pd(w.im);
                let (ar, ai) = load4(&a[k * 2]);
                let (br, bi) = load4(&a[k * 2 + 1]);
                let (br, bi) = mul4(br, bi, wr, wi);
                store4(&mut a[k * 2], _mm256_add_pd(ar, br), _mm256_add_pd(ai, bi));
                store4(
                    &mut a[k * 2 + 1],
                    _mm256_sub_pd(ar, br),
                    _mm256_sub_pd(ai, bi),
                );
            }
        }
    });
}

#[target_feature(enable = "avx2,fma")]
unsafe fn ifft_soa(a: &mut [Complex4]) {
    let n = a.len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let parity = n.trailing_zeros() & 1;
        if parity != 0 {
            let blocks = n / 8;
            for k in 0..blocks {
                let w = eval_twiddle(cache, 2, blocks, k).conjugate();
                let wr = _mm256_set1_pd(w.re);
                let wi = _mm256_set1_pd(w.im);
                let (ar, ai) = load4(&a[k * 2]);
                let (br, bi) = load4(&a[k * 2 + 1]);
                store4(&mut a[k * 2], _mm256_add_pd(ar, br), _mm256_add_pd(ai, bi));
                let (br, bi) = mul4(_mm256_sub_pd(ar, br), _mm256_sub_pd(ai, bi), wr, wi);
                store4(&mut a[k * 2 + 1], br, bi);
            }
        }
        for leaf in (12..n).step_by(16) {
            let max_level = (leaf + 3).trailing_ones();
            let mut level = 4 + parity;
            while level <= max_level {
                let len = 1usize << level;
                let q = leaf >> level;
                let width = len / 16;
                let start = q * width * 4;
                let (a, rest) = a[start..start + width * 4].split_at_mut(width);
                let (b, rest) = rest.split_at_mut(width);
                let (c, d) = rest.split_at_mut(width);
                let w1 = eval_twiddle(cache, 4, n >> level, q).conjugate();
                let w2 = w1 * w1;
                let w3 = w1 * w2;
                let (w1r, w1i) = (_mm256_set1_pd(w1.re), _mm256_set1_pd(w1.im));
                let (w2r, w2i) = (_mm256_set1_pd(w2.re), _mm256_set1_pd(w2.im));
                let (w3r, w3i) = (_mm256_set1_pd(w3.re), _mm256_set1_pd(w3.im));
                for i in 0..width {
                    let (ar, ai) = load4(&a[i]);
                    let (br, bi) = load4(&b[i]);
                    let (cr, ci) = load4(&c[i]);
                    let (dr, di) = load4(&d[i]);
                    let abr = _mm256_add_pd(ar, br);
                    let abi = _mm256_add_pd(ai, bi);
                    let cdr = _mm256_add_pd(cr, dr);
                    let cdi = _mm256_add_pd(ci, di);
                    let abd_r = _mm256_sub_pd(ar, br);
                    let abd_i = _mm256_sub_pd(ai, bi);
                    let cdd_r = _mm256_sub_pd(cr, dr);
                    let cdd_i = _mm256_sub_pd(ci, di);
                    store4(&mut a[i], _mm256_add_pd(abr, cdr), _mm256_add_pd(abi, cdi));
                    let (br, bi) = mul4(
                        _mm256_add_pd(abd_r, cdd_i),
                        _mm256_sub_pd(abd_i, cdd_r),
                        w1r,
                        w1i,
                    );
                    store4(&mut b[i], br, bi);
                    let (cr, ci) = mul4(_mm256_sub_pd(abr, cdr), _mm256_sub_pd(abi, cdi), w2r, w2i);
                    store4(&mut c[i], cr, ci);
                    let (dr, di) = mul4(
                        _mm256_sub_pd(abd_r, cdd_i),
                        _mm256_add_pd(abd_i, cdd_r),
                        w3r,
                        w3i,
                    );
                    store4(&mut d[i], dr, di);
                }
                level += 2;
            }
        }
        let scale = _mm256_set1_pd(4.0 / n as f64);
        for value in a {
            let (re, im) = load4(value);
            store4(value, _mm256_mul_pd(re, scale), _mm256_mul_pd(im, scale));
        }
    });
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
            let mut value = <M as MIntConvert<u32>>::into(values[i + lane].inner()) as i64;
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
unsafe fn dot_one_soa(a: &mut [Complex4], b: &[Complex4]) {
    let n = a.len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        for i in 0..a.len() {
            let (mut br, mut bi) = load4(&b[i]);
            let mut rr = _mm256_setzero_pd();
            let mut ri = _mm256_setzero_pd();
            let w = eval_twiddle(cache, 1, a.len(), i);
            let wr = _mm256_setr_pd(w.re, 1.0, 1.0, 1.0);
            let wi = _mm256_setr_pd(w.im, 0.0, 0.0, 0.0);
            for lane in 0..4 {
                let ar = _mm256_set1_pd(a[i].re[lane]);
                let ai = _mm256_set1_pd(a[i].im[lane]);
                multiply_accumulate4(&mut rr, &mut ri, ar, ai, br, bi);
                if lane != 3 {
                    br = _mm256_permute4x64_pd::<0x93>(br);
                    bi = _mm256_permute4x64_pd::<0x93>(bi);
                    (br, bi) = mul4(br, bi, wr, wi);
                }
            }
            store4(&mut a[i], rr, ri);
        }
    });
}

#[inline]
fn pack_f64(values: impl Iterator<Item = f64>, n: usize) -> Vec<Complex4> {
    let mut result = vec![Complex4::default(); n / 4];
    for (i, value) in values.enumerate() {
        if i < n {
            result[i >> 2].re[i & 3] = value;
        } else {
            result[(i - n) >> 2].im[i & 3] = value;
        }
    }
    result
}

#[target_feature(enable = "avx2,fma")]
unsafe fn middle_product_f64_avx2(
    a: impl ExactSizeIterator<Item = f64>,
    b: impl ExactSizeIterator<Item = f64>,
) -> Vec<f64> {
    let a_len = a.len();
    let b_len = b.len();
    let n = (a_len.next_power_of_two() / 2).max(4);
    let mut fa = pack_f64(a, n);
    let mut fb = pack_f64(b, n);
    fft_soa(&mut fa);
    fft_soa(&mut fb);
    dot_one_soa(&mut fa, &fb);
    drop(fb);
    ifft_soa(&mut fa);
    (b_len - 1..a_len)
        .map(|i| {
            if i < n {
                fa[i >> 2].re[i & 3]
            } else {
                fa[(i - n) >> 2].im[i & 3]
            }
        })
        .collect()
}

impl ConvolveRealFft {
    /// Returns coefficients `b.len() - 1..a.len()` of the convolution of `a` and `b`.
    /// Panics unless `0 < b.len() <= a.len()`.
    pub fn middle_product_f64(
        a: impl ExactSizeIterator<Item = f64>,
        b: impl ExactSizeIterator<Item = f64>,
    ) -> Vec<f64> {
        assert!(0 < b.len() && b.len() <= a.len());
        crate::avx_helper!(@dispatch_avx2_fma return unsafe {
            middle_product_f64_avx2(a, b)
        }, ());
        middle_product_f64_scalar(a, b)
    }
}

#[target_feature(enable = "avx2,fma")]
unsafe fn split_u64_coefficients(values: &[u64], n: usize, factor: u64) -> [Vec<Complex4>; 4] {
    let mut result = std::array::from_fn(|_| vec![Complex4::default(); n / 4]);
    let mut multiplier = 1u64;
    for (i, &value) in values.iter().enumerate() {
        let mut value = value.wrapping_mul(multiplier);
        let (i, imag) = if i < n { (i, false) } else { (i - n, true) };
        for part in &mut result {
            let digit = value as i16;
            value = (value >> 16).wrapping_add(u64::from(digit < 0));
            if imag {
                part[i >> 2].im[i & 3] = digit as f64;
            } else {
                part[i >> 2].re[i & 3] = digit as f64;
            }
        }
        multiplier = multiplier.wrapping_mul(factor);
    }
    result
}

#[target_feature(enable = "avx2,fma")]
unsafe fn dot_u64_soa(a: &mut [Vec<Complex4>; 4], b: &[Vec<Complex4>; 4]) {
    let n = a[0].len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        for block in 0..a[0].len() {
            let mut br = [_mm256_setzero_pd(); 4];
            let mut bi = br;
            let mut rr = br;
            let mut ri = br;
            for part in 0..4 {
                (br[part], bi[part]) = load4(&b[part][block]);
            }
            let w = eval_twiddle(cache, 1, a[0].len(), block);
            let wr = _mm256_setr_pd(w.re, 1.0, 1.0, 1.0);
            let wi = _mm256_setr_pd(w.im, 0.0, 0.0, 0.0);
            for lane in 0..4 {
                let ar: [__m256d; 4] =
                    std::array::from_fn(|part| _mm256_set1_pd(a[part][block].re[lane]));
                let ai: [__m256d; 4] =
                    std::array::from_fn(|part| _mm256_set1_pd(a[part][block].im[lane]));
                for part in 0..4 {
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
                    for part in 0..4 {
                        br[part] = _mm256_permute4x64_pd::<0x93>(br[part]);
                        bi[part] = _mm256_permute4x64_pd::<0x93>(bi[part]);
                        (br[part], bi[part]) = mul4(br[part], bi[part], wr, wi);
                    }
                }
            }
            for part in 0..4 {
                store4(&mut a[part][block], rr[part], ri[part]);
            }
        }
    });
}

#[target_feature(enable = "avx2,fma")]
pub unsafe fn convolve_u64_avx2(a: Vec<u64>, b: Vec<u64>, factor: u64, inverse: u64) -> Vec<u64> {
    let len = a.len() + b.len() - 1;
    let n = len.next_power_of_two() / 2;
    let mut fa = split_u64_coefficients(&a, n, factor);
    drop(a);
    let mut fb = split_u64_coefficients(&b, n, factor);
    drop(b);
    for part in 0..4 {
        fft_soa(&mut fa[part]);
        fft_soa(&mut fb[part]);
    }
    dot_u64_soa(&mut fa, &fb);
    drop(fb);
    for part in &mut fa {
        ifft_soa(part);
    }
    let mut result = vec![0; len];
    let mut real_multiplier = 1u64;
    let mut imag_multiplier = inverse.wrapping_pow(n as u32);
    for (block, _) in fa[0].iter().enumerate() {
        let real: [[i64; 4]; 4] = std::array::from_fn(|part| round4(&fa[part][block].re));
        let imag: [[i64; 4]; 4] = std::array::from_fn(|part| round4(&fa[part][block].im));
        for lane in 0..4 {
            let i = block * 4 + lane;
            if i < len {
                result[i] = (real[0][lane] as u64)
                    .wrapping_add((real[1][lane] as u64) << 16)
                    .wrapping_add((real[2][lane] as u64) << 32)
                    .wrapping_add((real[3][lane] as u64) << 48)
                    .wrapping_mul(real_multiplier);
            }
            real_multiplier = real_multiplier.wrapping_mul(inverse);
            if i + n < len {
                result[i + n] = (imag[0][lane] as u64)
                    .wrapping_add((imag[1][lane] as u64) << 16)
                    .wrapping_add((imag[2][lane] as u64) << 32)
                    .wrapping_add((imag[3][lane] as u64) << 48)
                    .wrapping_mul(imag_multiplier);
            }
            imag_multiplier = imag_multiplier.wrapping_mul(inverse);
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
    let modulus = modulus as u64;
    let split = split as u64;
    let split2 = split * split % modulus;
    let mut result = vec![MInt::<M>::from(0u32); len];
    for (block, ((a0, a1), b0)) in a0.iter().zip(&a1).zip(&b0).enumerate() {
        let values = [
            [round4(&a0.re), round4(&a1.re), round4(&b0.re)],
            [round4(&a0.im), round4(&a1.im), round4(&b0.im)],
        ];
        for (part, values) in values.into_iter().enumerate() {
            for (lane, value0) in values[0].into_iter().enumerate() {
                let i = block * 4 + lane + part * n;
                if i < len {
                    let value = (value0.rem_euclid(modulus as i64) as u64
                        + values[1][lane].rem_euclid(modulus as i64) as u64 * split
                        + values[2][lane].rem_euclid(modulus as i64) as u64 * split2)
                        % modulus;
                    result[i] = MInt::<M>::from(value as u32);
                }
            }
        }
    }
    result
}
