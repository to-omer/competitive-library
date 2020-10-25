use crate::num::Complex;

#[codesnip::entry("fast_fourier_transform", include("Complex"))]
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
        let mut w = Complex::new(1., 0.);
        for j in (0..n).step_by(i) {
            for k in 0..i {
                g[j + k] = f[((j * 2) & mask) + k] + w * f[((j * 2 + i) & mask) + k];
            }
            w = w * t;
        }
        i /= 2;
        std::mem::swap(&mut f, &mut g);
    }
    if inv {
        for a in f.iter_mut() {
            *a = *a / n as f64;
        }
    }
    f
}

#[codesnip::entry("fast_fourier_transform")]
pub fn convolve_i64(mut a: Vec<i64>, mut b: Vec<i64>) -> Vec<i64> {
    let m = a.len() + b.len() - 1;
    let n = m.next_power_of_two();
    a.resize_with(n, Default::default);
    b.resize_with(n, Default::default);
    let a: Vec<_> = a.into_iter().map(|x| Complex::new(x as f64, 0.)).collect();
    let b: Vec<_> = b.into_iter().map(|x| Complex::new(x as f64, 0.)).collect();
    let mut a = fast_fourier_transform(a, false);
    for (a, b) in a.iter_mut().zip(fast_fourier_transform(b, false)) {
        *a = *a * b;
    }
    let c = fast_fourier_transform(a, true);
    c.into_iter().take(m).map(|x| x.re.round() as i64).collect()
}

#[test]
fn test_fast_fourier_transform() {
    use crate::tools::Xorshift;
    const N: usize = 3_000;
    const M: i64 = 100_000;
    let mut rand = Xorshift::time();
    let a: Vec<_> = (0..N).map(|_| rand.rand(M as u64 * 2) as i64 - M).collect();
    let b: Vec<_> = (0..N).map(|_| rand.rand(M as u64 * 2) as i64 - M).collect();
    let mut c = vec![0; N * 2 - 1];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += a[i] * b[j];
        }
    }
    let d = convolve_i64(a, b);
    assert_eq!(c, d);
}
