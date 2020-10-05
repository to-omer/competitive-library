#![allow(clippy::suspicious_arithmetic_impl)]

#[snippet::entry("QuadDouble")]
/// ref: https://na-inet.jp/na/qd_ja.pdf
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct QuadDouble(pub f64, f64, f64, f64);
#[snippet::entry("QuadDouble")]
pub mod quad_double_impl {
    use super::*;
    impl QuadDouble {
        pub fn new(a: f64) -> Self {
            Self(a, 0., 0., 0.)
        }
        pub fn renormalize(a0: f64, a1: f64, a2: f64, a3: f64, a4: f64) -> Self {
            let (s, t4) = quick_two_sum(a3, a4);
            let (s, t3) = quick_two_sum(a2, s);
            let (s, t2) = quick_two_sum(a1, s);
            let (mut s, t1) = quick_two_sum(a0, s);
            let mut k = 0;
            let mut b = [s, t1, t2, t3];
            for &t in [t1, t2, t3, t4].iter() {
                let (s_, e) = quick_two_sum(s, t);
                s = s_;
                if e != 0. {
                    b[k] = s;
                    s = e;
                    k += 1;
                }
            }
            Self(b[0], b[1], b[2], b[3])
        }
    }
    #[inline]
    pub fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
        let s = a + b;
        let e = b - (s - a);
        (s, e)
    }
    #[inline]
    pub fn two_sum(a: f64, b: f64) -> (f64, f64) {
        let s = a + b;
        let v = s - a;
        let e = (a - (s - v)) + (b - v);
        (s, e)
    }
    #[inline]
    pub fn split(a: f64) -> (f64, f64) {
        let t = 134_217_729. * a; // 134217729 = 2 ** 27 + 1
        let ahi = t - (t - a);
        let alo = a - ahi;
        (ahi, alo)
    }
    #[inline]
    pub fn two_prod(a: f64, b: f64) -> (f64, f64) {
        let p = a * b;
        let (ahi, alo) = split(a);
        let (bhi, blo) = split(b);
        let e = ((ahi * bhi - p) + ahi * blo + alo * bhi) + alo * blo;
        (p, e)
    }
    #[inline]
    pub fn three_three_sum(a: f64, b: f64, c: f64) -> (f64, f64, f64) {
        let (u, v) = two_sum(a, b);
        let (r0, w) = two_sum(u, c);
        let (r1, r2) = two_sum(v, w);
        (r0, r1, r2)
    }
    #[inline]
    pub fn three_two_sum(a: f64, b: f64, c: f64) -> (f64, f64) {
        let (u, v) = two_sum(a, b);
        let (r0, w) = two_sum(u, c);
        let r1 = v + w;
        (r0, r1)
    }
    #[inline]
    pub fn multiple_three_sum(xs: &[f64]) -> (f64, f64, f64) {
        let (mut r0, mut r1, mut r2) = (*xs.get(0).unwrap_or(&0.), 0., 0.);
        for &x in xs.iter() {
            let (s, e) = two_sum(r0, x);
            r0 = s;
            let (s, e) = two_sum(r1, e);
            r1 = s;
            r2 += e;
        }
        (r0, r1, r2)
    }
    #[inline]
    pub fn multiple_two_sum(xs: &[f64]) -> (f64, f64) {
        let (mut r0, mut r1) = (*xs.get(0).unwrap_or(&0.), 0.);
        for &x in xs.iter() {
            let (s, e) = two_sum(r0, x);
            r0 = s;
            r1 += e;
        }
        (r0, r1)
    }
    impl std::ops::Add<f64> for QuadDouble {
        type Output = Self;
        fn add(self, rhs: f64) -> Self::Output {
            let (t0, e) = two_sum(self.0, rhs);
            let (t1, e) = two_sum(self.1, e);
            let (t2, e) = two_sum(self.2, e);
            let (t3, t4) = two_sum(self.3, e);
            Self::renormalize(t0, t1, t2, t3, t4)
        }
    }
    #[inline]
    pub fn double_accumulate(u: f64, v: f64, x: f64) -> (f64, f64, f64) {
        let (s, mut v) = two_sum(v, x);
        let (mut s, mut u) = two_sum(u, s);
        if u == 0. {
            u = s;
            s = 0.;
        }
        if v == 0. {
            v = u;
            u = s;
            s = 0.
        }
        (s, u, v)
    }
    impl std::ops::Add<QuadDouble> for QuadDouble {
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            let mut x = [0.; 8];
            let (mut i, mut j, mut k) = (0, 0, 0);
            while k < 8 {
                if j >= 4 || i < 4 && self[i].abs() > rhs[j].abs() {
                    x[k] = self[i];
                    i += 1;
                } else {
                    x[k] = rhs[j];
                    j += 1;
                }
                k += 1;
            }

            let (mut u, mut v) = (0., 0.);
            let (mut k, mut i) = (0, 0);
            let mut c = [0.; 4];
            while k < 4 && i < 8 {
                let tpl = double_accumulate(u, v, x[i]);
                let s = tpl.0;
                u = tpl.1;
                v = tpl.2;
                if s != 0. {
                    c[k] = s;
                    k += 1;
                }
                i += 1;
            }
            if k < 2 {
                c[k + 1] = v;
            }
            if k < 3 {
                c[k] = u;
            }
            Self::renormalize(c[0], c[1], c[2], c[3], 0.)
        }
    }
    impl std::ops::Sub for QuadDouble {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self::Output {
            self + -rhs
        }
    }
    impl std::ops::Neg for QuadDouble {
        type Output = Self;
        fn neg(self) -> Self::Output {
            Self(-self.0, -self.1, -self.2, -self.3)
        }
    }
    impl std::ops::Mul<f64> for QuadDouble {
        type Output = Self;
        fn mul(self, rhs: f64) -> Self::Output {
            let (t0, e0) = two_prod(self.0, rhs);
            let (p1, e1) = two_prod(self.1, rhs);
            let (p2, e2) = two_prod(self.2, rhs);
            let p3 = self.3 * rhs;

            let (t1, e4) = two_sum(p1, e0);
            let (t2, e5, e6) = three_three_sum(p2, e1, e4);
            let (t3, e7) = three_two_sum(p3, e2, e5);
            let t4 = e7 + e6;
            Self::renormalize(t0, t1, t2, t3, t4)
        }
    }
    impl std::ops::Mul<QuadDouble> for QuadDouble {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self::Output {
            let (t0, q00) = two_prod(self.0, rhs.0);

            let (p01, q01) = two_prod(self.0, rhs.1);
            let (p10, q10) = two_prod(self.1, rhs.0);

            let (p02, q02) = two_prod(self.0, rhs.2);
            let (p11, q11) = two_prod(self.1, rhs.1);
            let (p20, q20) = two_prod(self.2, rhs.0);

            let (p03, q03) = two_prod(self.0, rhs.3);
            let (p12, q12) = two_prod(self.1, rhs.2);
            let (p21, q21) = two_prod(self.2, rhs.1);
            let (p30, q30) = two_prod(self.3, rhs.0);

            let p13 = self.1 * rhs.3;
            let p22 = self.2 * rhs.2;
            let p31 = self.3 * rhs.1;

            let (t1, e1, e2) = three_three_sum(q00, p01, p10);
            let (t2, e3, e4) = multiple_three_sum(&[e1, q01, q10, p02, p11, p20]);
            let (t3, e5) = multiple_two_sum(&[e2, e3, q02, q11, q20, p03, p12, p21, p30]);
            let t4 = e4 + e5 + q03 + q12 + q21 + q30 + p13 + p22 + p31;
            Self::renormalize(t0, t1, t2, t3, t4)
        }
    }
    impl std::ops::Div<QuadDouble> for QuadDouble {
        type Output = Self;
        fn div(self, rhs: Self) -> Self::Output {
            let q0 = self.0 / rhs.0;
            let r = self - rhs * q0;
            let q1 = r.0 / rhs.0;
            let r = r - rhs * q1;
            let q2 = r.0 / rhs.0;
            let r = r - rhs * q2;
            let q3 = r.0 / rhs.0;
            let r = r - rhs * q3;
            let q4 = r.0 / rhs.0;
            Self::renormalize(q0, q1, q2, q3, q4)
        }
    }
    impl std::ops::Index<usize> for QuadDouble {
        type Output = f64;
        fn index(&self, index: usize) -> &Self::Output {
            match index {
                0 => &self.0,
                1 => &self.1,
                2 => &self.2,
                3 => &self.3,
                _ => panic!(),
            }
        }
    }
    impl Into<f64> for QuadDouble {
        fn into(self) -> f64 {
            self.3 + self.2 + self.1 + self.0
        }
    }
    impl From<f64> for QuadDouble {
        fn from(x: f64) -> Self {
            Self(x, 0., 0., 0.)
        }
    }
    impl std::fmt::Display for QuadDouble {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> Result<(), std::fmt::Error> {
            write!(f, "{}", self.3 + self.2 + self.1 + self.0)
        }
    }
    impl std::str::FromStr for QuadDouble {
        type Err = std::num::ParseFloatError;
        #[inline]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<f64>().map(Self::new)
        }
    }
    impl QuadDouble {
        #[inline]
        pub fn is_zero(&self) -> bool {
            self.0 == 0.
        }
        #[inline]
        pub fn is_sign_negative(&self) -> bool {
            self.0.is_sign_negative()
        }
        #[inline]
        pub fn sqrt(self) -> Self {
            if self.is_zero() {
                return Self::new(0.);
            }
            let x = Self::new(1. / self.0.sqrt());
            let x = x + x * (Self::new(1.) - self * x * x).div2(2.);
            let x = x + x * (Self::new(1.) - self * x * x).div2(2.);
            let x = x + x * (Self::new(1.) - self * x * x).div2(2.);
            x * self
        }
        #[inline]
        pub fn abs(self) -> Self {
            if self.0.is_sign_negative() {
                -self
            } else {
                self
            }
        }
        #[inline]
        pub fn div2(self, rhs: f64) -> Self {
            Self(self.0 / rhs, self.1 / rhs, self.2 / rhs, self.3 / rhs)
        }
    }
}
