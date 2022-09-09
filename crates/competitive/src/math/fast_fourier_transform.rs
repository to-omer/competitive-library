use super::{AssociatedValue, Complex, ConvolveSteps, One, Zero};

pub enum ConvolveRealFft {}

enum RotateCache {}
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

impl ConvolveSteps for ConvolveRealFft {
    type T = Vec<i64>;
    type F = Vec<Complex<f64>>;
    fn length(t: &Self::T) -> usize {
        t.len()
    }
    fn transform(t: Self::T, len: usize) -> Self::F {
        let n = len.max(4).next_power_of_two();
        let mut f = vec![Complex::zero(); n / 2];
        for (i, t) in t.into_iter().enumerate() {
            if i & 1 == 0 {
                f[i / 2].re = t as f64;
            } else {
                f[i / 2].im = t as f64;
            }
        }
        fft(&mut f);
        bit_reverse(&mut f);
        f[0] = Complex::new(f[0].re + f[0].im, f[0].re - f[0].im);
        f[n / 4] = f[n / 4].conjugate();
        let w = Complex::primitive_nth_root_of_unity(-(n as f64));
        let mut wk = Complex::<f64>::one();
        for k in 1..n / 4 {
            wk *= w;
            let c = wk.conjugate().transpose() + 1.;
            let d = c * (f[k] - f[n / 2 - k].conjugate()) * 0.5;
            f[k] -= d;
            f[n / 2 - k] += d.conjugate();
        }
        f
    }
    fn inverse_transform(mut f: Self::F, len: usize) -> Self::T {
        let n = len.max(4).next_power_of_two();
        assert_eq!(f.len(), n / 2);
        f[0] = Complex::new((f[0].re + f[0].im) * 0.5, (f[0].re - f[0].im) * 0.5);
        f[n / 4] = f[n / 4].conjugate();
        let w = Complex::primitive_nth_root_of_unity(n as f64);
        let mut wk = Complex::<f64>::one();
        for k in 1..n / 4 {
            wk *= w;
            let c = wk.transpose().conjugate() + 1.;
            let d = c * (f[k] - f[n / 2 - k].conjugate()) * 0.5;
            f[k] -= d;
            f[n / 2 - k] += d.conjugate();
        }
        bit_reverse(&mut f);
        ifft(&mut f);
        let inv = 1. / (n / 2) as f64;
        (0..len)
            .map(|i| (inv * if i & 1 == 0 { f[i / 2].re } else { f[i / 2].im }).round() as i64)
            .collect()
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

pub fn fft(a: &mut [Complex<f64>]) {
    let n = a.len();
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let mut v = n / 2;
        while v > 0 {
            for (a, wj) in a.chunks_exact_mut(v << 1).zip(cache) {
                let (l, r) = a.split_at_mut(v);
                for (x, y) in l.iter_mut().zip(r) {
                    let ajv = wj * *y;
                    *y = *x - ajv;
                    *x += ajv;
                }
            }
            v >>= 1;
        }
    });
}

pub fn ifft(a: &mut [Complex<f64>]) {
    let n = a.len();
    RotateCache::ensure(n / 2);
    RotateCache::with(|cache| {
        let mut v = 1;
        while v < n {
            for (a, wj) in a
                .chunks_exact_mut(v << 1)
                .zip(cache.iter().map(|wj| wj.conjugate()))
            {
                let (l, r) = a.split_at_mut(v);
                for (x, y) in l.iter_mut().zip(r) {
                    let ajv = *x - *y;
                    *x += *y;
                    *y = wj * ajv;
                }
            }
            v <<= 1;
        }
    });
}

#[test]
fn test_convolve_fft() {
    use crate::{rand, tools::Xorshift};
    let mut rng = Xorshift::default();
    for n in 0..10 {
        for m in 0..10 {
            for rn in 0..2 {
                for rm in 0..2 {
                    let n = 2usize.pow(n);
                    let m = 2usize.pow(m);
                    let n = n - rng.gen(0..n) * rn;
                    let m = m - rng.gen(0..m) * rm;
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
