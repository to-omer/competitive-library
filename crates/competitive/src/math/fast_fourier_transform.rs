use crate::{
    impl_assoc_value,
    num::{Complex, One, Zero},
    tools::AssociatedValue,
};

#[codesnip::entry("fast_fourier_transform")]
pub use fast_fourier_transform_impls::convolve_fft;
#[codesnip::entry("fast_fourier_transform", include("Complex", "AssociatedValue"))]
pub mod fast_fourier_transform_impls {
    use super::*;
    struct RotateCache;
    impl RotateCache {
        fn ensure(n: usize) {
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
                    let p = Complex::polar(1., -std::f64::consts::PI / (m * 2) as f64);
                    for i in 0..m {
                        cache.push(cache[i] * p);
                    }
                    m <<= 1;
                }
                assert_eq!(cache.len(), n);
            });
        }
    }
    impl_assoc_value!(RotateCache, Vec<Complex<f64>>, vec![Complex::one()]);
    pub fn convolve_fft<IA, T, IB, U>(a: IA, b: IB) -> Vec<i64>
    where
        T: Into<f64>,
        U: Into<f64>,
        IA: IntoIterator<Item = T>,
        IA::IntoIter: ExactSizeIterator,
        IB: IntoIterator<Item = U>,
        IB::IntoIter: ExactSizeIterator,
    {
        let a = a.into_iter();
        let b = b.into_iter();
        let alen = a.len();
        let blen = b.len();
        assert_ne!(alen, 0, "empty sequence on first argument");
        assert_ne!(blen, 0, "empty sequence on second argument");
        let m = alen + blen - 1;
        let n = (std::cmp::max(m, 2)).next_power_of_two();
        let mut c = vec![Complex::zero(); n];
        for (c, a) in c.iter_mut().zip(a) {
            c.re = a.into();
        }
        for (c, b) in c.iter_mut().zip(b) {
            c.im = b.into();
        }

        RotateCache::ensure(n / 2);
        RotateCache::with(|cache| {
            fft(&mut c, &cache);

            c[0] = Complex::new(0., c[0].re * c[0].im);
            c[1] = Complex::new(0., c[1].re * c[1].im);
            for i in (2..n).step_by(2) {
                let j = {
                    let y = 1 << (63 - i.leading_zeros());
                    (!i & (y - 1)) ^ y
                };
                c[i] = (c[i] + c[j].conjugate()) * (c[i] - c[j].conjugate()) / 4.;
                c[j] = -c[i].conjugate();
            }

            for i in 0..n / 2 {
                let mut wi = cache[i] * Complex::i();
                wi.re += 1.;
                c[i] = c[i * 2] - (c[i * 2] - c[i * 2 + 1]) * wi / 2.;
            }

            ifft(&mut c[..n / 2], &cache);
        });

        (0..m)
            .map(|i| {
                (if i & 1 == 0 {
                    c[i / 2].im
                } else {
                    c[i / 2 + 1].re
                } / ((n / 2) as f64))
                    .round() as _
            })
            .collect()
    }
    #[allow(clippy::needless_range_loop)]
    fn fft(a: &mut [Complex<f64>], cache: &[Complex<f64>]) {
        let n = a.len();
        let mut u = 1;
        let mut v = n / 2;
        for i in (1..=n.trailing_zeros()).rev() {
            for jh in 0..u {
                let wj = cache[jh];
                for j in jh << i..(jh << i) + v {
                    let ajv = wj * a[j + v];
                    a[j + v] = a[j] - ajv;
                    a[j] += ajv;
                }
            }
            u <<= 1;
            v >>= 1;
        }
    }
    #[allow(clippy::needless_range_loop)]
    fn ifft(a: &mut [Complex<f64>], cache: &[Complex<f64>]) {
        let n = a.len();
        let mut u = n / 2;
        let mut v = 1;
        for i in 1..=n.trailing_zeros() {
            for jh in 0..u {
                let wj = cache[jh].conjugate();
                for j in jh << i..(jh << i) + v {
                    let ajv = a[j] - a[j + v];
                    a[j] += a[j + v];
                    a[j + v] = wj * ajv;
                }
            }
            u >>= 1;
            v <<= 1;
        }
    }
    pub fn fast_fourier_transform(mut f: Vec<Complex<f64>>, inv: bool) -> Vec<Complex<f64>> {
        let n = f.len();
        debug_assert!(n.count_ones() == 1);
        let mask = n - 1;
        const TAU: f64 = 2. * std::f64::consts::PI;
        let omega = if inv { -TAU / n as f64 } else { TAU / n as f64 };

        let mut g = vec![Complex::<f64>::default(); n];
        let mut i = n / 2;
        while i >= 1 {
            let t = Complex::polar(1., omega * i as f64);
            let mut w = Complex::<f64>::one();
            for j in (0..n).step_by(i) {
                for k in 0..i {
                    g[j + k] = f[((j * 2) & mask) + k] + w * f[((j * 2 + i) & mask) + k];
                }
                w *= t;
            }
            i /= 2;
            std::mem::swap(&mut f, &mut g);
        }
        if inv {
            for a in f.iter_mut() {
                *a /= n as f64;
            }
        }
        f
    }
}

#[test]
fn test_convolve_fft() {
    use crate::{rand, tools::Xorshift};
    for n in 0..=10 {
        let n = 2usize.pow(n);
        const A: i32 = 100_000;
        let mut rng = Xorshift::default();
        rand!(rng, a: [-A..=A; n], b: [-A..=A; n]);
        let mut c = vec![0; n * 2 - 1];
        for i in 0..n {
            for j in 0..n {
                c[i + j] += a[i] as i64 * b[j] as i64;
            }
        }
        let d = convolve_fft(a, b);
        assert_eq!(c, d);
    }
}
