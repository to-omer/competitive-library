//! Partial transforms modulo X^(blocks * lanes) - i, leaving lane-sized products.
//! Buffers contain a nonzero power-of-two number of blocks. SIMD functions require
//! their declared CPU features. Real inputs pack their two halves into re and im.

use super::{Complex, One};

// Correctly rounded exp(-i pi / 2^(k+1)); matching uses only these levels.
const ROOTS: [Complex<f64>; 20] = [
    Complex {
        re: f64::from_bits(0x0000000000000000),
        im: f64::from_bits(0xbff0000000000000),
    },
    Complex {
        re: f64::from_bits(0x3fe6a09e667f3bcd),
        im: f64::from_bits(0xbfe6a09e667f3bcd),
    },
    Complex {
        re: f64::from_bits(0x3fed906bcf328d46),
        im: f64::from_bits(0xbfd87de2a6aea963),
    },
    Complex {
        re: f64::from_bits(0x3fef6297cff75cb0),
        im: f64::from_bits(0xbfc8f8b83c69a60b),
    },
    Complex {
        re: f64::from_bits(0x3fefd88da3d12526),
        im: f64::from_bits(0xbfb917a6bc29b42c),
    },
    Complex {
        re: f64::from_bits(0x3feff621e3796d7e),
        im: f64::from_bits(0xbfa91f65f10dd814),
    },
    Complex {
        re: f64::from_bits(0x3feffd886084cd0d),
        im: f64::from_bits(0xbf992155f7a3667e),
    },
    Complex {
        re: f64::from_bits(0x3fefff62169b92db),
        im: f64::from_bits(0xbf8921d1fcdec784),
    },
    Complex {
        re: f64::from_bits(0x3fefffd8858e8a92),
        im: f64::from_bits(0xbf7921f0fe670071),
    },
    Complex {
        re: f64::from_bits(0x3feffff621621d02),
        im: f64::from_bits(0xbf6921f8becca4ba),
    },
    Complex {
        re: f64::from_bits(0x3feffffd88586ee6),
        im: f64::from_bits(0xbf5921faaee6472e),
    },
    Complex {
        re: f64::from_bits(0x3fefffff62161a34),
        im: f64::from_bits(0xbf4921fb2aecb360),
    },
    Complex {
        re: f64::from_bits(0x3fefffffd8858675),
        im: f64::from_bits(0xbf3921fb49ee4ea6),
    },
    Complex {
        re: f64::from_bits(0x3feffffff621619c),
        im: f64::from_bits(0xbf2921fb51aeb57c),
    },
    Complex {
        re: f64::from_bits(0x3feffffffd885867),
        im: f64::from_bits(0xbf1921fb539ecf31),
    },
    Complex {
        re: f64::from_bits(0x3fefffffff62161a),
        im: f64::from_bits(0xbf0921fb541ad59e),
    },
    Complex {
        re: f64::from_bits(0x3fefffffffd88586),
        im: f64::from_bits(0xbef921fb5439d73a),
    },
    Complex {
        re: f64::from_bits(0x3feffffffff62162),
        im: f64::from_bits(0xbee921fb544197a1),
    },
    Complex {
        re: f64::from_bits(0x3feffffffffd8858),
        im: f64::from_bits(0xbed921fb544387ba),
    },
    Complex {
        re: f64::from_bits(0x3fefffffffff6216),
        im: f64::from_bits(0xbec921fb544403c1),
    },
];
thread_local! {
    static ROTATE_CACHE: std::cell::RefCell<Vec<Complex<f64>>> =
        std::cell::RefCell::new(vec![Complex::one()]);
}

/// Borrows at least `n` cached roots, where `n` is a nonzero power of two.
/// The callback must not reenter this cache.
pub fn with_roots<T>(n: usize, f: impl FnOnce(&[Complex<f64>]) -> T) -> T {
    assert!(n.is_power_of_two());
    ROTATE_CACHE.with_borrow_mut(|cache| {
        let mut m = cache.len();
        if m < n {
            cache.reserve_exact(n - m);
            while m < n {
                let level = m.trailing_zeros() as usize;
                let p = ROOTS
                    .get(level)
                    .copied()
                    .unwrap_or_else(|| Complex::primitive_nth_root_of_unity(-((m * 4) as f64)));
                for i in 0..m {
                    let value = cache[i] * p;
                    cache.push(value);
                }
                m <<= 1;
            }
        }
        f(cache)
    })
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
macro_rules! partial_fft {
    ([$($qualifier:tt)*] $(#[$attr:meta])*; $block:ident, $lanes:literal) => {
        $(#[$attr])*
        pub $($qualifier)* fn fft_soa(a: &mut [$block]) {
            let n = a.len() * $lanes;
            with_roots(n / 2, |cache| {
                let parity = (n.trailing_zeros() - ($lanes as u32).trailing_zeros()) & 1;
                for leaf in (0..n).step_by(4 * $lanes) {
                    let mut level = (n + leaf).trailing_zeros();
                    level -= u32::from(level & 1 != n.trailing_zeros() & 1);
                    while level >= ($lanes as u32).trailing_zeros() + 2 {
                        let len = 1usize << level;
                        let q = leaf >> level;
                        let width = len / (4 * $lanes);
                        let start = q * width * 4;
                        let (a, rest) = a[start..start + width * 4].split_at_mut(width);
                        let (b, rest) = rest.split_at_mut(width);
                        let (c, d) = rest.split_at_mut(width);
                        let w1 = eval_twiddle(cache, 4, n >> level, q);
                        let w2 = w1 * w1;
                        let w3 = w1 * w2;
                        let (w1r, w1i) = (splat(w1.re), splat(w1.im));
                        let (w2r, w2i) = (splat(w2.re), splat(w2.im));
                        let (w3r, w3i) = (splat(w3.re), splat(w3.im));
                        for i in 0..width {
                            let (ar, ai) = load(&a[i]);
                            let (br, bi) = load(&b[i]);
                            let (cr, ci) = load(&c[i]);
                            let (dr, di) = load(&d[i]);
                            let (br, bi) = complex_mul(br, bi, w1r, w1i);
                            let (cr, ci) = complex_mul(cr, ci, w2r, w2i);
                            let (dr, di) = complex_mul(dr, di, w3r, w3i);
                            let acr = add(ar, cr);
                            let aci = add(ai, ci);
                            let bdr = add(br, dr);
                            let bdi = add(bi, di);
                            let acd_r = sub(ar, cr);
                            let acd_i = sub(ai, ci);
                            let bdd_r = sub(br, dr);
                            let bdd_i = sub(bi, di);
                            store(&mut a[i], add(acr, bdr), add(aci, bdi));
                            store(&mut b[i], sub(acr, bdr), sub(aci, bdi));
                            store(&mut c[i], sub(acd_r, bdd_i), add(acd_i, bdd_r));
                            store(&mut d[i], add(acd_r, bdd_i), sub(acd_i, bdd_r));
                        }
                        level -= 2;
                    }
                }
                if parity != 0 {
                    let blocks = n / (2 * $lanes);
                    for k in 0..blocks {
                        let w = eval_twiddle(cache, 2, blocks, k);
                        let wr = splat(w.re);
                        let wi = splat(w.im);
                        let (ar, ai) = load(&a[k * 2]);
                        let (br, bi) = load(&a[k * 2 + 1]);
                        let (br, bi) = complex_mul(br, bi, wr, wi);
                        store(&mut a[k * 2], add(ar, br), add(ai, bi));
                        store(&mut a[k * 2 + 1], sub(ar, br), sub(ai, bi));
                    }
                }
            });
        }

        $(#[$attr])*
        pub $($qualifier)* fn ifft_soa(a: &mut [$block]) {
            let n = a.len() * $lanes;
            with_roots(n / 2, |cache| {
                let parity = (n.trailing_zeros() - ($lanes as u32).trailing_zeros()) & 1;
                if parity != 0 {
                    let blocks = n / (2 * $lanes);
                    for k in 0..blocks {
                        let w = eval_twiddle(cache, 2, blocks, k).conjugate();
                        let wr = splat(w.re);
                        let wi = splat(w.im);
                        let (ar, ai) = load(&a[k * 2]);
                        let (br, bi) = load(&a[k * 2 + 1]);
                        store(&mut a[k * 2], add(ar, br), add(ai, bi));
                        let (br, bi) = complex_mul(sub(ar, br), sub(ai, bi), wr, wi);
                        store(&mut a[k * 2 + 1], br, bi);
                    }
                }
                for leaf in (3 * $lanes..n).step_by(4 * $lanes) {
                    let max_level = (leaf + $lanes - 1).trailing_ones();
                    let mut level = ($lanes as u32).trailing_zeros() + 2 + parity;
                    while level <= max_level {
                        let len = 1usize << level;
                        let q = leaf >> level;
                        let width = len / (4 * $lanes);
                        let start = q * width * 4;
                        let (a, rest) = a[start..start + width * 4].split_at_mut(width);
                        let (b, rest) = rest.split_at_mut(width);
                        let (c, d) = rest.split_at_mut(width);
                        let w1 = eval_twiddle(cache, 4, n >> level, q).conjugate();
                        let w2 = w1 * w1;
                        let w3 = w1 * w2;
                        let (w1r, w1i) = (splat(w1.re), splat(w1.im));
                        let (w2r, w2i) = (splat(w2.re), splat(w2.im));
                        let (w3r, w3i) = (splat(w3.re), splat(w3.im));
                        for i in 0..width {
                            let (ar, ai) = load(&a[i]);
                            let (br, bi) = load(&b[i]);
                            let (cr, ci) = load(&c[i]);
                            let (dr, di) = load(&d[i]);
                            let abr = add(ar, br);
                            let abi = add(ai, bi);
                            let cdr = add(cr, dr);
                            let cdi = add(ci, di);
                            let abd_r = sub(ar, br);
                            let abd_i = sub(ai, bi);
                            let cdd_r = sub(cr, dr);
                            let cdd_i = sub(ci, di);
                            store(&mut a[i], add(abr, cdr), add(abi, cdi));
                            let (br, bi) = complex_mul(add(abd_r, cdd_i), sub(abd_i, cdd_r), w1r, w1i);
                            store(&mut b[i], br, bi);
                            let (cr, ci) = complex_mul(sub(abr, cdr), sub(abi, cdi), w2r, w2i);
                            store(&mut c[i], cr, ci);
                            let (dr, di) = complex_mul(sub(abd_r, cdd_i), add(abd_i, cdd_r), w3r, w3i);
                            store(&mut d[i], dr, di);
                        }
                        level += 2;
                    }
                }
                let scale = splat($lanes as f64 / n as f64);
                for value in a {
                    let (re, im) = load(value);
                    store(value, mul(re, scale), mul(im, scale));
                }
            });
        }

    };
}

pub mod scalar {
    use super::{eval_twiddle, with_roots};
    #[derive(Clone, Copy, Default)]
    pub struct Complex4 {
        pub re: [f64; 4],
        pub im: [f64; 4],
    }
    #[inline]
    pub fn load(v: &Complex4) -> ([f64; 4], [f64; 4]) {
        (v.re, v.im)
    }
    #[inline]
    pub fn store(v: &mut Complex4, re: [f64; 4], im: [f64; 4]) {
        v.re = re;
        v.im = im;
    }
    #[inline]
    pub fn splat(x: f64) -> [f64; 4] {
        [x; 4]
    }
    #[inline]
    pub fn first(a: f64, b: f64) -> [f64; 4] {
        [a, b, b, b]
    }
    #[inline]
    pub fn rotate(v: [f64; 4]) -> [f64; 4] {
        [v[3], v[0], v[1], v[2]]
    }
    #[inline]
    pub fn add(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
        std::array::from_fn(|i| a[i] + b[i])
    }
    #[inline]
    pub fn sub(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
        std::array::from_fn(|i| a[i] - b[i])
    }
    #[inline]
    pub fn mul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
        std::array::from_fn(|i| a[i] * b[i])
    }
    #[inline]
    pub fn complex_mul(
        ar: [f64; 4],
        ai: [f64; 4],
        br: [f64; 4],
        bi: [f64; 4],
    ) -> ([f64; 4], [f64; 4]) {
        (sub(mul(ar, br), mul(ai, bi)), add(mul(ai, br), mul(ar, bi)))
    }
    #[inline]
    pub fn multiply_accumulate(
        rr: &mut [f64; 4],
        ri: &mut [f64; 4],
        ar: [f64; 4],
        ai: [f64; 4],
        br: [f64; 4],
        bi: [f64; 4],
    ) {
        *rr = add(*rr, mul(ar, br));
        *rr = sub(*rr, mul(ai, bi));
        *ri = add(*ri, mul(ai, br));
        *ri = add(*ri, mul(ar, bi));
    }
    partial_fft!([]; Complex4, 4);
}

#[cfg(target_arch = "x86_64")]
pub mod avx2 {
    // Callers check CPU features before entering these kernels.
    #![allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
    use super::{eval_twiddle, with_roots};
    use std::arch::x86_64::*;
    pub use std::arch::x86_64::{
        _mm256_add_pd as add, _mm256_mul_pd as mul, _mm256_set1_pd as splat, _mm256_sub_pd as sub,
    };
    #[derive(Clone, Copy, Default)]
    #[repr(C, align(32))]
    pub struct Complex4 {
        pub re: [f64; 4],
        pub im: [f64; 4],
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn load(value: &Complex4) -> (__m256d, __m256d) {
        (
            _mm256_load_pd(value.re.as_ptr()),
            _mm256_load_pd(value.im.as_ptr()),
        )
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn store(value: &mut Complex4, re: __m256d, im: __m256d) {
        _mm256_store_pd(value.re.as_mut_ptr(), re);
        _mm256_store_pd(value.im.as_mut_ptr(), im);
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn complex_mul(
        ar: __m256d,
        ai: __m256d,
        br: __m256d,
        bi: __m256d,
    ) -> (__m256d, __m256d) {
        (
            _mm256_fmsub_pd(ar, br, _mm256_mul_pd(ai, bi)),
            _mm256_fmadd_pd(ai, br, _mm256_mul_pd(ar, bi)),
        )
    }

    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn multiply_accumulate(
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
    pub unsafe fn first(a: f64, b: f64) -> __m256d {
        _mm256_setr_pd(a, b, b, b)
    }
    #[target_feature(enable = "avx2,fma")]
    #[inline]
    pub unsafe fn rotate(v: __m256d) -> __m256d {
        _mm256_permute4x64_pd::<0x93>(v)
    }
    partial_fft!([unsafe] #[target_feature(enable = "avx2,fma")]; Complex4, 4);
}

#[cfg(target_arch = "x86_64")]
pub mod avx512 {
    // Callers check CPU features before entering these kernels.
    #![allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
    use super::{eval_twiddle, with_roots};
    use std::arch::x86_64::*;
    pub use std::arch::x86_64::{
        _mm512_add_pd as add, _mm512_mul_pd as mul, _mm512_set1_pd as splat, _mm512_sub_pd as sub,
    };
    #[derive(Clone, Copy, Default)]
    #[repr(C, align(64))]
    pub struct Complex8 {
        pub re: [f64; 8],
        pub im: [f64; 8],
    }

    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    pub unsafe fn load(value: &Complex8) -> (__m512d, __m512d) {
        (
            _mm512_load_pd(value.re.as_ptr()),
            _mm512_load_pd(value.im.as_ptr()),
        )
    }

    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    pub unsafe fn store(value: &mut Complex8, re: __m512d, im: __m512d) {
        _mm512_store_pd(value.re.as_mut_ptr(), re);
        _mm512_store_pd(value.im.as_mut_ptr(), im);
    }

    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    pub unsafe fn complex_mul(
        ar: __m512d,
        ai: __m512d,
        br: __m512d,
        bi: __m512d,
    ) -> (__m512d, __m512d) {
        (
            _mm512_fmsub_pd(ar, br, _mm512_mul_pd(ai, bi)),
            _mm512_fmadd_pd(ai, br, _mm512_mul_pd(ar, bi)),
        )
    }

    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    pub unsafe fn multiply_accumulate(
        rr: &mut __m512d,
        ri: &mut __m512d,
        ar: __m512d,
        ai: __m512d,
        br: __m512d,
        bi: __m512d,
    ) {
        *rr = _mm512_fmadd_pd(ar, br, *rr);
        *rr = _mm512_fnmadd_pd(ai, bi, *rr);
        *ri = _mm512_fmadd_pd(ai, br, *ri);
        *ri = _mm512_fmadd_pd(ar, bi, *ri);
    }

    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    pub unsafe fn first(a: f64, b: f64) -> __m512d {
        _mm512_setr_pd(a, b, b, b, b, b, b, b)
    }
    #[target_feature(enable = "avx2,avx512f,fma")]
    #[inline]
    pub unsafe fn rotate(v: __m512d) -> __m512d {
        _mm512_permutexvar_pd(_mm512_setr_epi64(7, 0, 1, 2, 3, 4, 5, 6), v)
    }
    partial_fft!([unsafe] #[target_feature(enable = "avx2,avx512f,fma")]; Complex8, 8);
}
