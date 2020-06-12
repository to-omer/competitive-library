use crate::num::complex::Complex;

pub fn fft_rec(f: &mut [Complex<f64>], inv: bool) {
    let n = f.len();
    debug_assert!(n.count_ones() == 1);
    if n == 1 {
        return;
    }
    let mut g1: Vec<_> = (0..n).step_by(2).map(|i| f[i]).collect();
    let mut g2: Vec<_> = (1..n).step_by(2).map(|i| f[i]).collect();
    fft_rec(&mut g1, inv);
    fft_rec(&mut g2, inv);
    const TAU: f64 = 2. * std::f64::consts::PI;
    let omega = if inv { -TAU / n as f64 } else { TAU / n as f64 };
    for i in 0..n {
        f[i] = g1[i % (n / 2)] + Complex::polar(1., omega * i as f64) * g2[i % (n / 2)];
    }
}

#[test]
fn test_fft_rec() {
    let f = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let mut g: Vec<_> = f.iter().map(|&x| Complex::new(x as f64, 0.)).collect();
    fft_rec(&mut g, false);
    fft_rec(&mut g, true);
    let g: Vec<_> = g
        .into_iter()
        .map(|x| (x.re / f.len() as f64).round() as i32)
        .collect();
    assert_eq!(f, g);
}

pub fn fft(mut f: Vec<Complex<f64>>, inv: bool) -> Vec<Complex<f64>> {
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
                g[j + k] = f[(j * 2 & mask) + k] + w * f[(j * 2 + i & mask) + k];
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

#[test]
fn test_fft() {
    let f = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let g: Vec<_> = f.iter().map(|&x| Complex::new(x as f64, 0.)).collect();
    let g = fft(g, false);
    let g = fft(g, true);
    let g: Vec<_> = g.into_iter().map(|x| x.re.round() as i32).collect();
    assert_eq!(f, g);
}
