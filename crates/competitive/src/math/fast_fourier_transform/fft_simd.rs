#![allow(unsafe_op_in_unsafe_fn)]

use super::{AssociatedValue, Complex, MInt, MIntConvert, RotateCache};
use std::arch::x86_64::*;
#[derive(Clone, Copy, Default)]
#[repr(C, align(32))]
struct Complex4 {
    re: [f64; 4],
    im: [f64; 4],
}

impl Complex4 {
    #[inline]
    fn get(&self, lane: usize) -> Complex<f64> {
        Complex::new(self.re[lane], self.im[lane])
    }

    #[inline]
    fn set(&mut self, lane: usize, value: Complex<f64>) {
        self.re[lane] = value.re;
        self.im[lane] = value.im;
    }
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
unsafe fn round4(value: &[f64; 4], scale: __m256d) -> [i64; 4] {
    let magic = _mm256_set1_pd((3i64 << 51) as f64);
    let rounded = _mm256_sub_epi64(
        _mm256_castpd_si256(_mm256_add_pd(
            _mm256_mul_pd(_mm256_load_pd(value.as_ptr()), scale),
            magic,
        )),
        _mm256_castpd_si256(magic),
    );
    let mut result = [0; 4];
    _mm256_storeu_si256(result.as_mut_ptr().cast(), rounded);
    result
}

#[target_feature(enable = "avx2,fma")]
unsafe fn fft_soa(a: &mut [Complex4]) {
    let n = a.len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let mut v = n / 2;
        while v >= 8 {
            let l = v / 2;
            let block_len = l / 4;
            for (q, block) in a.chunks_exact_mut(block_len * 4).enumerate() {
                let (a, rest) = block.split_at_mut(block_len);
                let (b, rest) = rest.split_at_mut(block_len);
                let (c, d) = rest.split_at_mut(block_len);
                let w0 = cache[q];
                let w1 = cache[q << 1];
                let w2 = cache[q << 1 | 1];
                let w0r = _mm256_set1_pd(w0.re);
                let w0i = _mm256_set1_pd(w0.im);
                let w1r = _mm256_set1_pd(w1.re);
                let w1i = _mm256_set1_pd(w1.im);
                let w2r = _mm256_set1_pd(w2.re);
                let w2i = _mm256_set1_pd(w2.im);
                for i in 0..block_len {
                    let (ar, ai) = load4(&a[i]);
                    let (br, bi) = load4(&b[i]);
                    let (cr, ci) = load4(&c[i]);
                    let (dr, di) = load4(&d[i]);
                    let (cr, ci) = mul4(cr, ci, w0r, w0i);
                    let (dr, di) = mul4(dr, di, w0r, w0i);
                    let ac0r = _mm256_add_pd(ar, cr);
                    let ac0i = _mm256_add_pd(ai, ci);
                    let ac1r = _mm256_sub_pd(ar, cr);
                    let ac1i = _mm256_sub_pd(ai, ci);
                    let (bd0r, bd0i) = mul4(_mm256_add_pd(br, dr), _mm256_add_pd(bi, di), w1r, w1i);
                    let (bd1r, bd1i) = mul4(_mm256_sub_pd(br, dr), _mm256_sub_pd(bi, di), w2r, w2i);
                    store4(
                        &mut a[i],
                        _mm256_add_pd(ac0r, bd0r),
                        _mm256_add_pd(ac0i, bd0i),
                    );
                    store4(
                        &mut b[i],
                        _mm256_sub_pd(ac0r, bd0r),
                        _mm256_sub_pd(ac0i, bd0i),
                    );
                    store4(
                        &mut c[i],
                        _mm256_add_pd(ac1r, bd1r),
                        _mm256_add_pd(ac1i, bd1i),
                    );
                    store4(
                        &mut d[i],
                        _mm256_sub_pd(ac1r, bd1r),
                        _mm256_sub_pd(ac1i, bd1i),
                    );
                }
            }
            v >>= 2;
        }
        if v == 4 {
            for (block, w) in a.chunks_exact_mut(2).zip(cache) {
                let (ar, ai) = load4(&block[0]);
                let (br, bi) = load4(&block[1]);
                let (br, bi) = mul4(br, bi, _mm256_set1_pd(w.re), _mm256_set1_pd(w.im));
                store4(&mut block[0], _mm256_add_pd(ar, br), _mm256_add_pd(ai, bi));
                store4(&mut block[1], _mm256_sub_pd(ar, br), _mm256_sub_pd(ai, bi));
            }
            v = 2;
        }
        if v == 2 {
            for (value, w) in a.iter_mut().zip(cache) {
                let (re, im) = load4(value);
                let yr = _mm256_permute2f128_pd::<0x01>(re, re);
                let yi = _mm256_permute2f128_pd::<0x01>(im, im);
                let (yr, yi) = mul4(yr, yi, _mm256_set1_pd(w.re), _mm256_set1_pd(w.im));
                let dr =
                    _mm256_permute2f128_pd::<0x01>(_mm256_sub_pd(re, yr), _mm256_sub_pd(re, yr));
                let di =
                    _mm256_permute2f128_pd::<0x01>(_mm256_sub_pd(im, yi), _mm256_sub_pd(im, yi));
                store4(
                    value,
                    _mm256_blend_pd::<0b1100>(_mm256_add_pd(re, yr), dr),
                    _mm256_blend_pd::<0b1100>(_mm256_add_pd(im, yi), di),
                );
            }
        }
        for (q, value) in a.iter_mut().enumerate() {
            let (re, im) = load4(value);
            let yr = _mm256_permute_pd::<0b0101>(re);
            let yi = _mm256_permute_pd::<0b0101>(im);
            let w0 = cache[q << 1];
            let w1 = cache[q << 1 | 1];
            let wr = _mm256_setr_pd(w0.re, w0.re, w1.re, w1.re);
            let wi = _mm256_setr_pd(w0.im, w0.im, w1.im, w1.im);
            let (yr, yi) = mul4(yr, yi, wr, wi);
            let dr = _mm256_permute_pd::<0b0101>(_mm256_sub_pd(re, yr));
            let di = _mm256_permute_pd::<0b0101>(_mm256_sub_pd(im, yi));
            store4(
                value,
                _mm256_blend_pd::<0b1010>(_mm256_add_pd(re, yr), dr),
                _mm256_blend_pd::<0b1010>(_mm256_add_pd(im, yi), di),
            );
        }
    });
}

#[target_feature(enable = "avx2,fma")]
unsafe fn ifft_soa(a: &mut [Complex4]) {
    let n = a.len() * 4;
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        for (q, value) in a.iter_mut().enumerate() {
            let (re, im) = load4(value);
            let yr = _mm256_permute_pd::<0b0101>(re);
            let yi = _mm256_permute_pd::<0b0101>(im);
            let w0 = cache[q << 1].conjugate();
            let w1 = cache[q << 1 | 1].conjugate();
            let wr = _mm256_setr_pd(w0.re, w0.re, w1.re, w1.re);
            let wi = _mm256_setr_pd(w0.im, w0.im, w1.im, w1.im);
            let (dr, di) = mul4(_mm256_sub_pd(re, yr), _mm256_sub_pd(im, yi), wr, wi);
            store4(
                value,
                _mm256_blend_pd::<0b1010>(_mm256_add_pd(re, yr), _mm256_permute_pd::<0b0101>(dr)),
                _mm256_blend_pd::<0b1010>(_mm256_add_pd(im, yi), _mm256_permute_pd::<0b0101>(di)),
            );
        }
        for (value, w) in a.iter_mut().zip(cache) {
            let (re, im) = load4(value);
            let yr = _mm256_permute2f128_pd::<0x01>(re, re);
            let yi = _mm256_permute2f128_pd::<0x01>(im, im);
            let w = w.conjugate();
            let (dr, di) = mul4(
                _mm256_sub_pd(re, yr),
                _mm256_sub_pd(im, yi),
                _mm256_set1_pd(w.re),
                _mm256_set1_pd(w.im),
            );
            store4(
                value,
                _mm256_blend_pd::<0b1100>(
                    _mm256_add_pd(re, yr),
                    _mm256_permute2f128_pd::<0x01>(dr, dr),
                ),
                _mm256_blend_pd::<0b1100>(
                    _mm256_add_pd(im, yi),
                    _mm256_permute2f128_pd::<0x01>(di, di),
                ),
            );
        }
        let mut v = 4;
        while v << 1 < n {
            let block_len = v / 4;
            for (q, block) in a.chunks_exact_mut(block_len * 4).enumerate() {
                let (a, rest) = block.split_at_mut(block_len);
                let (b, rest) = rest.split_at_mut(block_len);
                let (c, d) = rest.split_at_mut(block_len);
                let w0 = cache[q].conjugate();
                let w1 = cache[q << 1].conjugate();
                let w2 = cache[q << 1 | 1].conjugate();
                let w0r = _mm256_set1_pd(w0.re);
                let w0i = _mm256_set1_pd(w0.im);
                let w1r = _mm256_set1_pd(w1.re);
                let w1i = _mm256_set1_pd(w1.im);
                let w2r = _mm256_set1_pd(w2.re);
                let w2i = _mm256_set1_pd(w2.im);
                for i in 0..block_len {
                    let (ar, ai) = load4(&a[i]);
                    let (br, bi) = load4(&b[i]);
                    let (cr, ci) = load4(&c[i]);
                    let (dr, di) = load4(&d[i]);
                    let ab0r = _mm256_add_pd(ar, br);
                    let ab0i = _mm256_add_pd(ai, bi);
                    let (ab1r, ab1i) = mul4(_mm256_sub_pd(ar, br), _mm256_sub_pd(ai, bi), w1r, w1i);
                    let cd0r = _mm256_add_pd(cr, dr);
                    let cd0i = _mm256_add_pd(ci, di);
                    let (cd1r, cd1i) = mul4(_mm256_sub_pd(cr, dr), _mm256_sub_pd(ci, di), w2r, w2i);
                    store4(
                        &mut a[i],
                        _mm256_add_pd(ab0r, cd0r),
                        _mm256_add_pd(ab0i, cd0i),
                    );
                    store4(
                        &mut b[i],
                        _mm256_add_pd(ab1r, cd1r),
                        _mm256_add_pd(ab1i, cd1i),
                    );
                    let (cr, ci) = mul4(
                        _mm256_sub_pd(ab0r, cd0r),
                        _mm256_sub_pd(ab0i, cd0i),
                        w0r,
                        w0i,
                    );
                    store4(&mut c[i], cr, ci);
                    let (dr, di) = mul4(
                        _mm256_sub_pd(ab1r, cd1r),
                        _mm256_sub_pd(ab1i, cd1i),
                        w0r,
                        w0i,
                    );
                    store4(&mut d[i], dr, di);
                }
            }
            v <<= 2;
        }
        if v < n {
            let block_len = v / 4;
            for (block, w) in a.chunks_exact_mut(block_len * 2).zip(cache) {
                let (l, r) = block.split_at_mut(block_len);
                let w = w.conjugate();
                let wr = _mm256_set1_pd(w.re);
                let wi = _mm256_set1_pd(w.im);
                for i in 0..block_len {
                    let (lr, li) = load4(&l[i]);
                    let (rr, ri) = load4(&r[i]);
                    store4(&mut l[i], _mm256_add_pd(lr, rr), _mm256_add_pd(li, ri));
                    let (rr, ri) = mul4(_mm256_sub_pd(lr, rr), _mm256_sub_pd(li, ri), wr, wi);
                    store4(&mut r[i], rr, ri);
                }
            }
        }
    });
}

#[target_feature(enable = "avx2,fma")]
unsafe fn transpose4(a: __m256d, b: __m256d, c: __m256d, d: __m256d) -> [__m256d; 4] {
    let ab0 = _mm256_unpacklo_pd(a, b);
    let ab1 = _mm256_unpackhi_pd(a, b);
    let cd0 = _mm256_unpacklo_pd(c, d);
    let cd1 = _mm256_unpackhi_pd(c, d);
    [
        _mm256_permute2f128_pd::<0x20>(ab0, cd0),
        _mm256_permute2f128_pd::<0x20>(ab1, cd1),
        _mm256_permute2f128_pd::<0x31>(ab0, cd0),
        _mm256_permute2f128_pd::<0x31>(ab1, cd1),
    ]
}

#[target_feature(enable = "avx2,fma")]
unsafe fn bit_reverse_soa(a: &mut Vec<Complex4>) {
    let quarter = a.len() / 4;
    let shift = usize::BITS - quarter.trailing_zeros();
    let mut result = vec![Complex4::default(); a.len()];
    for i in 0..quarter {
        let j = i.reverse_bits() >> shift;
        let (r0, i0) = load4(&a[j]);
        let (r1, i1) = load4(&a[j + quarter * 2]);
        let (r2, i2) = load4(&a[j + quarter]);
        let (r3, i3) = load4(&a[j + quarter * 3]);
        let re = transpose4(r0, r1, r2, r3);
        let im = transpose4(i0, i1, i2, i3);
        store4(&mut result[i], re[0], im[0]);
        store4(&mut result[i + quarter], re[2], im[2]);
        store4(&mut result[i + quarter * 2], re[1], im[1]);
        store4(&mut result[i + quarter * 3], re[3], im[3]);
    }
    *a = result;
}

#[target_feature(enable = "avx2,fma")]
unsafe fn real_fft_soa(a: &mut Vec<Complex4>) {
    let n = a.len() * 8;
    fft_soa(a);
    bit_reverse_soa(a);
    let z = a[0].get(0);
    a[0].set(0, Complex::new(z.re + z.im, z.re - z.im));
    let middle = n / 4;
    let z = a[middle >> 2].get(middle & 3).conjugate();
    a[middle >> 2].set(middle & 3, z);
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let shift = usize::BITS - (n / 2).trailing_zeros();
        for k in 1..n / 4 {
            let j = n / 2 - k;
            let x = a[k >> 2].get(k & 3);
            let y = a[j >> 2].get(j & 3);
            let w = cache[k.reverse_bits() >> shift];
            let c = w.conjugate().transpose() + 1.0;
            let d = c * (x - y.conjugate()) * 0.5;
            a[k >> 2].set(k & 3, x - d);
            a[j >> 2].set(j & 3, y + d.conjugate());
        }
    });
}

#[target_feature(enable = "avx2,fma")]
unsafe fn real_ifft_soa(a: &mut Vec<Complex4>) {
    let n = a.len() * 8;
    let z = a[0].get(0);
    a[0].set(0, Complex::new((z.re + z.im) * 0.5, (z.re - z.im) * 0.5));
    let middle = n / 4;
    let z = a[middle >> 2].get(middle & 3).conjugate();
    a[middle >> 2].set(middle & 3, z);
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let shift = usize::BITS - (n / 2).trailing_zeros();
        for k in 1..n / 4 {
            let j = n / 2 - k;
            let x = a[k >> 2].get(k & 3);
            let y = a[j >> 2].get(j & 3);
            let w = cache[k.reverse_bits() >> shift].conjugate();
            let c = w.transpose().conjugate() + 1.0;
            let d = c * (x - y.conjugate()) * 0.5;
            a[k >> 2].set(k & 3, x - d);
            a[j >> 2].set(j & 3, y + d.conjugate());
        }
    });
    bit_reverse_soa(a);
    ifft_soa(a);
}

fn split_coefficients<M>(
    values: Vec<MInt<M>>,
    blocks: usize,
    modulus: i64,
    split: i64,
) -> (Vec<Complex4>, Vec<Complex4>)
where
    M: MIntConvert + MIntConvert<u32>,
{
    let mut low = vec![Complex4::default(); blocks];
    let mut high = low.clone();
    for (i, x) in values.into_iter().enumerate() {
        let mut x = <M as MIntConvert<u32>>::into(x.inner()) as i64;
        if x * 2 > modulus {
            x -= modulus;
        }
        let upper = (x as f64 / split as f64).round() as i64;
        let lane = (i >> 1) & 3;
        let (low, high) = if i & 1 == 0 {
            (&mut low[i >> 3].re, &mut high[i >> 3].re)
        } else {
            (&mut low[i >> 3].im, &mut high[i >> 3].im)
        };
        low[lane] = (x - upper * split) as f64;
        high[lane] = upper as f64;
    }
    (low, high)
}

// Fixed lane indexes keep coefficient order and compile to straight-line recovery.
#[target_feature(enable = "avx2,fma")]
#[allow(clippy::needless_range_loop)]
pub(super) unsafe fn convolve_mint_avx2<M>(a: Vec<MInt<M>>, b: Vec<MInt<M>>) -> Vec<MInt<M>>
where
    M: MIntConvert + MIntConvert<u32>,
{
    let len = a.len() + b.len() - 1;
    let n = len.next_power_of_two();
    let modulus = <M as MIntConvert<u32>>::mod_into() as i64;
    let split = (modulus as f64).sqrt() as i64 + 1;
    let (mut a0, mut a1) = split_coefficients(a, n / 8, modulus, split);
    let (mut b0, mut b1) = split_coefficients(b, n / 8, modulus, split);
    real_fft_soa(&mut a0);
    real_fft_soa(&mut a1);
    real_fft_soa(&mut b0);
    real_fft_soa(&mut b1);

    let a00 = a0[0].get(0);
    let a10 = a1[0].get(0);
    let b00 = b0[0].get(0);
    let b10 = b1[0].get(0);
    for i in 0..a0.len() {
        let (a0r, a0i) = load4(&a0[i]);
        let (a1r, a1i) = load4(&a1[i]);
        let (b0r, b0i) = load4(&b0[i]);
        let (b1r, b1i) = load4(&b1[i]);
        let (c0r, c0i) = mul4(a0r, a0i, b0r, b0i);
        let (c01r, c01i) = mul4(a0r, a0i, b1r, b1i);
        let (c10r, c10i) = mul4(a1r, a1i, b0r, b0i);
        let (c2r, c2i) = mul4(a1r, a1i, b1r, b1i);
        store4(&mut a0[i], c0r, c0i);
        store4(
            &mut a1[i],
            _mm256_add_pd(c01r, c10r),
            _mm256_add_pd(c01i, c10i),
        );
        store4(&mut b0[i], c2r, c2i);
    }
    a0[0].set(0, Complex::new(a00.re * b00.re, a00.im * b00.im));
    a1[0].set(
        0,
        Complex::new(
            a00.re * b10.re + a10.re * b00.re,
            a00.im * b10.im + a10.im * b00.im,
        ),
    );
    b0[0].set(0, Complex::new(a10.re * b10.re, a10.im * b10.im));
    real_ifft_soa(&mut a0);
    real_ifft_soa(&mut a1);
    real_ifft_soa(&mut b0);
    let inv_n = 2.0 / n as f64;
    let modulus = modulus as u64;
    let split = split as u64;
    let split2 = split * split % modulus;
    let scale = _mm256_set1_pd(inv_n);
    let mut result = Vec::with_capacity(len);
    for block in 0..a0.len() {
        let values = [
            [round4(&a0[block].re, scale), round4(&a0[block].im, scale)],
            [round4(&a1[block].re, scale), round4(&a1[block].im, scale)],
            [round4(&b0[block].re, scale), round4(&b0[block].im, scale)],
        ];
        for lane in 0..4 {
            for part in 0..2 {
                if result.len() == len {
                    break;
                }
                let value = (values[0][part][lane].rem_euclid(modulus as i64) as u64
                    + values[1][part][lane].rem_euclid(modulus as i64) as u64 * split
                    + values[2][part][lane].rem_euclid(modulus as i64) as u64 * split2)
                    % modulus;
                result.push(MInt::<M>::from(value as u32));
            }
        }
    }
    result
}
