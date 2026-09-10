use super::{AssociatedValue, Complex, ConvolveSteps, One, Zero};
#[cfg(target_arch = "x86_64")]
use super::{SimdBackend, simd_backend};

pub enum ConvolveRealFft {}

pub enum RotateCache {}
impl RotateCache {
    pub fn ensure(n: usize) {
        assert_eq!(n.count_ones(), 1, "call with power of two but {}", n);
        Self::modify(|cache| {
            let mut m = cache.len();
            assert!(
                m.count_ones() <= 1,
                "length might be power of two but {}",
                m
            );
            if m >= n {
                return;
            }
            cache.reserve_exact(n - m);
            if cache.is_empty() {
                cache.push(Complex::one());
                m += 1;
            }
            while m < n {
                let p = Complex::primitive_nth_root_of_unity(-((m * 4) as f64));
                for i in 0..m {
                    cache.push(cache[i] * p);
                }
                m <<= 1;
            }
            assert_eq!(cache.len(), n);
        });
    }
}
crate::impl_assoc_value!(RotateCache, Vec<Complex<f64>>, vec![Complex::one()]);

#[cfg(target_arch = "x86_64")]
pub mod simd {
    // These primitives are called only after AVX2 and FMA have been detected.
    #![allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]

    use super::{AssociatedValue, Complex, RotateCache};
    use std::arch::x86_64::*;

    #[derive(Clone, Copy, Default)]
    #[repr(C, align(32))]
    pub struct Complex4 {
        pub re: [f64; 4],
        pub im: [f64; 4],
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn load4(value: &Complex4) -> (__m256d, __m256d) {
        (
            _mm256_load_pd(value.re.as_ptr()),
            _mm256_load_pd(value.im.as_ptr()),
        )
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn store4(value: &mut Complex4, re: __m256d, im: __m256d) {
        _mm256_store_pd(value.re.as_mut_ptr(), re);
        _mm256_store_pd(value.im.as_mut_ptr(), im);
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn mul4(ar: __m256d, ai: __m256d, br: __m256d, bi: __m256d) -> (__m256d, __m256d) {
        (
            _mm256_fmsub_pd(ar, br, _mm256_mul_pd(ai, bi)),
            _mm256_fmadd_pd(ai, br, _mm256_mul_pd(ar, bi)),
        )
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn multiply_accumulate4(
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

    #[inline]
    pub fn eval_twiddle(cache: &[Complex<f64>], step: usize, n: usize, k: usize) -> Complex<f64> {
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
    pub unsafe fn fft_soa(a: &mut [Complex4]) {
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
    pub unsafe fn ifft_soa(a: &mut [Complex4]) {
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
                        let (cr, ci) =
                            mul4(_mm256_sub_pd(abr, cdr), _mm256_sub_pd(abi, cdi), w2r, w2i);
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
    pub unsafe fn convolve_f64_avx2(
        a: impl ExactSizeIterator<Item = f64>,
        b: impl ExactSizeIterator<Item = f64>,
        range: std::ops::Range<usize>,
    ) -> Vec<f64> {
        let n = (range.end.next_power_of_two() / 2).max(4);
        let mut fa = pack_f64(a, n);
        let mut fb = pack_f64(b, n);
        fft_soa(&mut fa);
        fft_soa(&mut fb);
        dot_one_soa(&mut fa, &fb);
        drop(fb);
        ifft_soa(&mut fa);
        range
            .map(|i| {
                if i < n {
                    fa[i >> 2].re[i & 3]
                } else {
                    fa[(i - n) >> 2].im[i & 3]
                }
            })
            .collect()
    }
    #[target_feature(enable = "avx2")]
    pub unsafe fn convolve_i64_avx2(a: Vec<i64>, b: Vec<i64>, len: usize) -> Vec<i64> {
        super::convolve_i64_naive(a, b, len)
    }
    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn convolve_i64_avx512(a: Vec<i64>, b: Vec<i64>, len: usize) -> Vec<i64> {
        super::convolve_i64_naive(a, b, len)
    }
}

fn bit_reverse<T>(f: &mut [T]) {
    let mut ip = vec![0u32];
    let mut k = f.len();
    let mut m = 1;
    while 2 * m < k {
        k /= 2;
        for j in 0..m {
            ip.push(ip[j] + k as u32);
        }
        m *= 2;
    }
    if m == k {
        for i in 1..m {
            for j in 0..i {
                let ji = j + ip[i] as usize;
                let ij = i + ip[j] as usize;
                f.swap(ji, ij);
            }
        }
    } else {
        for i in 1..m {
            for j in 0..i {
                let ji = j + ip[i] as usize;
                let ij = i + ip[j] as usize;
                f.swap(ji, ij);
                f.swap(ji + m, ij + m);
            }
        }
    }
}

fn real_twiddles(n: usize, inverse: bool, mut f: impl FnMut(usize, Complex<f64>)) {
    const BLOCK: usize = 256;
    let sign = if inverse { 1.0 } else { -1.0 };
    let step = Complex::primitive_nth_root_of_unity(sign * n as f64);
    for start in (1..n / 4).step_by(BLOCK) {
        let mut w = Complex::polar(1.0, sign * std::f64::consts::TAU * start as f64 / n as f64);
        for k in start..(start + BLOCK).min(n / 4) {
            f(k, w);
            w *= step;
        }
    }
}

pub fn transform_real(t: impl IntoIterator<Item = f64>, len: usize) -> Vec<Complex<f64>> {
    let n = len.max(4).next_power_of_two();
    let mut f = vec![Complex::zero(); n / 2];
    for (i, t) in t.into_iter().enumerate() {
        if i & 1 == 0 {
            f[i / 2].re = t;
        } else {
            f[i / 2].im = t;
        }
    }
    fft(&mut f);
    bit_reverse(&mut f);
    f[0] = Complex::new(f[0].re + f[0].im, f[0].re - f[0].im);
    f[n / 4] = f[n / 4].conjugate();
    real_twiddles(n, false, |k, wk| {
        let c = wk.conjugate().transpose() + 1.;
        let d = c * (f[k] - f[n / 2 - k].conjugate()) * 0.5;
        f[k] -= d;
        f[n / 2 - k] += d.conjugate();
    });
    f
}

pub fn inverse_transform_real(mut f: Vec<Complex<f64>>, len: usize) -> Vec<f64> {
    let n = len.max(4).next_power_of_two();
    assert_eq!(f.len(), n / 2);
    f[0] = Complex::new((f[0].re + f[0].im) * 0.5, (f[0].re - f[0].im) * 0.5);
    f[n / 4] = f[n / 4].conjugate();
    real_twiddles(n, true, |k, wk| {
        let c = wk.transpose().conjugate() + 1.;
        let d = c * (f[k] - f[n / 2 - k].conjugate()) * 0.5;
        f[k] -= d;
        f[n / 2 - k] += d.conjugate();
    });
    bit_reverse(&mut f);
    ifft(&mut f);
    let inv = 1. / (n / 2) as f64;
    (0..len)
        .map(|i| inv * if i & 1 == 0 { f[i / 2].re } else { f[i / 2].im })
        .collect()
}

#[inline(always)]
fn convolve_i64_naive(a: Vec<i64>, b: Vec<i64>, len: usize) -> Vec<i64> {
    let (a, b) = if a.len() < b.len() { (b, a) } else { (a, b) };
    if b.len() == 1 {
        return a.into_iter().map(|a| a * b[0]).collect();
    }
    let mut c = vec![0; len];
    for (i, a) in a.chunks(1024).enumerate() {
        for (j, b) in b.iter().enumerate() {
            let start = i * 1024 + j;
            for (c, a) in c[start..start + a.len()].iter_mut().zip(a) {
                *c += *a * *b;
            }
        }
    }
    c
}

impl ConvolveSteps for ConvolveRealFft {
    type T = Vec<i64>;
    type F = Vec<Complex<f64>>;
    fn length(t: &Self::T) -> usize {
        t.len()
    }
    fn transform(t: Self::T, len: usize) -> Self::F {
        transform_real(t.into_iter().map(|t| t as f64), len)
    }
    fn inverse_transform(f: Self::F, len: usize) -> Self::T {
        inverse_transform_real(f, len)
            .into_iter()
            .map(|value| value.round() as i64)
            .collect()
    }
    fn convolve(a: Self::T, b: Self::T) -> Self::T {
        let len = (a.len() + b.len()).saturating_sub(1);
        // Keep accumulation overflow-free and exact in the FFT's f64 representation.
        if (a.len().min(b.len()) <= 32 || {
            let size = len.next_power_of_two();
            let log = size.ilog2();
            let limit = crate::avx_helper!(@dispatch simd_backend, SimdBackend;
                3 * log + 16, 2 * log + 16, 4 * log + 16
            );
            2 * a.len() as u128 * b.len() as u128 <= limit as u128 * size as u128
        }) && a.iter().map(|x| x.unsigned_abs()).max().unwrap_or(0) as u128
            * b.iter().map(|x| x.unsigned_abs()).max().unwrap_or(0) as u128
            <= (1u128 << 53) / a.len().min(b.len()).max(1) as u128
        {
            return crate::avx_helper!(@dispatch simd_backend, SimdBackend;
                unsafe {
                    if a.len().max(b.len()) < 32 {
                        simd::convolve_i64_avx2(a, b, len)
                    } else {
                        simd::convolve_i64_avx512(a, b, len)
                    }
                },
                unsafe { simd::convolve_i64_avx2(a, b, len) },
                convolve_i64_naive(a, b, len)
            );
        }
        if !a.is_empty() && !b.is_empty() {
            crate::avx_helper!(@dispatch_avx2_fma return unsafe {
                simd::convolve_f64_avx2(
                    a.into_iter().map(|x| x as f64),
                    b.into_iter().map(|x| x as f64),
                    0..len,
                )
                .into_iter()
                .map(|x| x.round() as i64)
                .collect()
            }, ());
        }
        let mut a = Self::transform(a, len);
        let b = Self::transform(b, len);
        Self::multiply(&mut a, &b);
        Self::inverse_transform(a, len)
    }
    fn multiply(f: &mut Self::F, g: &Self::F) {
        assert_eq!(f.len(), g.len());
        f[0].re *= g[0].re;
        f[0].im *= g[0].im;
        for (f, g) in f.iter_mut().zip(g.iter()).skip(1) {
            *f *= *g;
        }
    }
}

fn middle_product_f64_scalar(
    a: impl ExactSizeIterator<Item = f64>,
    b: impl ExactSizeIterator<Item = f64>,
) -> Vec<f64> {
    let a_len = a.len();
    let b_len = b.len();
    let len = a_len + b_len - 1;
    let mut a = transform_real(a, len);
    let b = transform_real(b, len);
    ConvolveRealFft::multiply(&mut a, &b);
    inverse_transform_real(a, len)[b_len - 1..a_len].to_vec()
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
            let range = b.len() - 1..a.len();
            simd::convolve_f64_avx2(a, b, range)
        }, ());
        middle_product_f64_scalar(a, b)
    }
}

macro_rules! fft_kernel {
    ($a:expr, $cache:expr, $inverse:expr) => {{
        let a = $a;
        let cache = $cache;
        let n = a.len();
        if $inverse {
            let mut v = 1;
            if n.trailing_zeros() & 1 == 1 {
                for (a, w) in a.as_chunks_mut::<2>().0.iter_mut().zip(cache) {
                    let y = (a[0] - a[1]) * w.conjugate();
                    a[0] += a[1];
                    a[1] = y;
                }
                v = 2;
            }
            while v < n {
                for (q, block) in a.chunks_exact_mut(v * 4).enumerate() {
                    let (a, rest) = block.split_at_mut(v);
                    let (b, rest) = rest.split_at_mut(v);
                    let (c, d) = rest.split_at_mut(v);
                    let w0 = cache[q].conjugate();
                    let w1 = cache[q << 1].conjugate();
                    let w3 = w0 * w1;
                    for (((a, b), c), d) in a.iter_mut().zip(b).zip(c).zip(d) {
                        let ac0 = *a + *b;
                        let ac1 = *c + *d;
                        let bd0 = *a - *b;
                        let bd1 = *c - *d;
                        let bd1 = Complex::new(-bd1.im, bd1.re);
                        *a = ac0 + ac1;
                        *b = (bd0 + bd1) * w1;
                        *c = (ac0 - ac1) * w0;
                        *d = (bd0 - bd1) * w3;
                    }
                }
                v <<= 2;
            }
        } else {
            let mut v = n / 2;
            while v >= 2 {
                let l = v / 2;
                for (q, block) in a.chunks_exact_mut(l * 4).enumerate() {
                    let (a, rest) = block.split_at_mut(l);
                    let (b, rest) = rest.split_at_mut(l);
                    let (c, d) = rest.split_at_mut(l);
                    let w0 = cache[q];
                    let w1 = cache[q << 1];
                    let w3 = w0 * w1;
                    for (((a, b), c), d) in a.iter_mut().zip(b).zip(c).zip(d) {
                        let bv = *b * w1;
                        let cv = *c * w0;
                        let dv = *d * w3;
                        let ac0 = *a + cv;
                        let ac1 = *a - cv;
                        let bd0 = bv + dv;
                        let bd1 = bv - dv;
                        let bd1 = Complex::new(bd1.im, -bd1.re);
                        *a = ac0 + bd0;
                        *b = ac0 - bd0;
                        *c = ac1 + bd1;
                        *d = ac1 - bd1;
                    }
                }
                v >>= 2;
            }
            if v == 1 {
                for (a, w) in a.as_chunks_mut::<2>().0.iter_mut().zip(cache) {
                    let y = a[1] * *w;
                    a[1] = a[0] - y;
                    a[0] += y;
                }
            }
        }
    }};
}

pub fn fft(a: &mut [Complex<f64>]) {
    fft_dispatch::<false>(a);
}

pub fn ifft(a: &mut [Complex<f64>]) {
    fft_dispatch::<true>(a);
}

fn fft_dispatch<const INVERSE: bool>(a: &mut [Complex<f64>]) {
    RotateCache::ensure(a.len() / 2);
    RotateCache::with(|cache| {
        #[cfg(target_arch = "x86_64")]
        if a.len() >= 16 {
            match simd_backend() {
                SimdBackend::Avx512 => {
                    return unsafe { fft_avx512::<INVERSE>(a, cache) };
                }
                SimdBackend::Avx2 => return unsafe { fft_avx2::<INVERSE>(a, cache) },
                SimdBackend::Scalar => {}
            }
        }
        fft_kernel!(a, cache, INVERSE);
    });
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fft_avx2<const INVERSE: bool>(a: &mut [Complex<f64>], cache: &[Complex<f64>]) {
    fft_kernel!(a, cache, INVERSE);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
unsafe fn fft_avx512<const INVERSE: bool>(a: &mut [Complex<f64>], cache: &[Complex<f64>]) {
    fft_kernel!(a, cache, INVERSE);
}

#[test]
fn test_convolve_fft() {
    use crate::{rand, tools::Xorshift};
    let mut rng = Xorshift::default();
    for log_n in 1..=16 {
        let n = 1 << log_n;
        let input: Vec<_> = (0..n)
            .map(|_| {
                Complex::new(
                    (rng.randf() - 0.5) * 200_000.,
                    (rng.randf() - 0.5) * 200_000.,
                )
            })
            .collect();
        let mut actual = input.clone();
        fft(&mut actual);
        if n <= 64 {
            for k in 0..n {
                let mut expected: Complex<f64> = Complex::zero();
                for (j, value) in input.iter().enumerate() {
                    let angle = -std::f64::consts::TAU * (j * k) as f64 / n as f64;
                    expected += *value * Complex::new(angle.cos(), angle.sin());
                }
                let actual = actual[k.reverse_bits() >> (usize::BITS - log_n)];
                assert!((actual.re - expected.re).abs() < 1e-6);
                assert!((actual.im - expected.im).abs() < 1e-6);
            }
        }
        ifft(&mut actual);
        for (actual, expected) in actual.into_iter().zip(input) {
            assert!((actual.re / n as f64 - expected.re).abs() < 1e-7);
            assert!((actual.im / n as f64 - expected.im).abs() < 1e-7);
        }
    }
    for log_n in 10..=16 {
        let size = 1 << log_n;
        for n in size - 1..=size + 1 {
            let m = rng.random(1..=size + 1);
            let a: Vec<i64> = rng.random_iter(-128..=128).take(n).collect();
            let factor = rng.random(1i64..=256) * if rng.gen_bool(0.5) { 1 } else { -1 };
            let b: Vec<_> = (0..m)
                .map(|i| if i & 1 == 0 { factor } else { -factor })
                .collect();
            let mut prefix = vec![0; n + 1];
            for (i, value) in a.iter().enumerate() {
                prefix[i + 1] = prefix[i] + if i & 1 == 0 { *value } else { -*value };
            }
            let expected: Vec<_> = (0..n + m - 1)
                .map(|i| {
                    (prefix[(i + 1).min(n)] - prefix[(i + 1).saturating_sub(m)])
                        * if i & 1 == 0 { factor } else { -factor }
                })
                .collect();
            assert_eq!(ConvolveRealFft::convolve(a.clone(), b.clone()), expected);
            assert_eq!(ConvolveRealFft::convolve(b, a), expected);
        }
    }
    for m in 1..=32 {
        for n in [
            rng.random(1..=64),
            1023,
            1024,
            1025,
            rng.random(1026..=3073),
        ] {
            let factor = rng.random(1i64..=16);
            let limit = (1i64 << 53) / m as i64 / factor;
            let a: Vec<_> = (0..n)
                .map(|_| (limit - rng.random(0i64..=128)) * if rng.gen_bool(0.5) { 1 } else { -1 })
                .collect();
            let b: Vec<i64> = rng.random_iter(-factor..=factor).take(m).collect();
            let mut expected = vec![0; n + m - 1];
            for (i, a) in a.iter().enumerate() {
                for (j, b) in b.iter().enumerate() {
                    expected[i + j] += a * b;
                }
            }
            assert_eq!(ConvolveRealFft::convolve(a.clone(), b.clone()), expected);
            assert_eq!(ConvolveRealFft::convolve(b, a), expected);
        }
    }
    for n in 0..10 {
        for m in 0..10 {
            for rn in 0..2 {
                for rm in 0..2 {
                    let n = 2usize.pow(n);
                    let m = 2usize.pow(m);
                    let n = n - rng.random(0..n) * rn;
                    let m = m - rng.random(0..m) * rm;
                    const A: i64 = 100_000;
                    rand!(rng, a: [-A..=A; n], b: [-A..=A; m]);
                    let mut c = vec![0; n + m - 1];
                    for i in 0..n {
                        for j in 0..m {
                            c[i + j] += a[i] * b[j];
                        }
                    }
                    let d = ConvolveRealFft::convolve(a, b);
                    assert_eq!(c, d);
                }
            }
        }
    }
}
