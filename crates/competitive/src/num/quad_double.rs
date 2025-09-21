use super::{Bounded, Decimal, IterScan, One, Zero};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    num::ParseFloatError,
    ops::{Add, Div, Index, Mul, Neg, Sub},
    str::FromStr,
};

/// ref: <https://na-inet.jp/na/qd_ja.pdf>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuadDouble(f64, f64, f64, f64);

impl QuadDouble {
    fn renormalize(a0: f64, a1: f64, a2: f64, a3: f64, a4: f64) -> Self {
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

fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let e = b - (s - a);
    (s, e)
}

fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    (s, e)
}

fn split(a: f64) -> (f64, f64) {
    let t = 134_217_729. * a; // 134217729 = 2 ** 27 + 1
    let ahi = t - (t - a);
    let alo = a - ahi;
    (ahi, alo)
}

fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (ahi, alo) = split(a);
    let (bhi, blo) = split(b);
    let e = ((ahi * bhi - p) + ahi * blo + alo * bhi) + alo * blo;
    (p, e)
}

fn three_three_sum(a: f64, b: f64, c: f64) -> (f64, f64, f64) {
    let (u, v) = two_sum(a, b);
    let (r0, w) = two_sum(u, c);
    let (r1, r2) = two_sum(v, w);
    (r0, r1, r2)
}

fn three_two_sum(a: f64, b: f64, c: f64) -> (f64, f64) {
    let (u, v) = two_sum(a, b);
    let (r0, w) = two_sum(u, c);
    let r1 = v + w;
    (r0, r1)
}

fn multiple_three_sum(xs: &[f64]) -> (f64, f64, f64) {
    let (mut r0, mut r1, mut r2) = (*xs.first().unwrap_or(&0.), 0., 0.);
    for &x in xs.iter() {
        let (s, e) = two_sum(r0, x);
        r0 = s;
        let (s, e) = two_sum(r1, e);
        r1 = s;
        r2 += e;
    }
    (r0, r1, r2)
}

fn multiple_two_sum(xs: &[f64]) -> (f64, f64) {
    let (mut r0, mut r1) = (*xs.first().unwrap_or(&0.), 0.);
    for &x in xs.iter() {
        let (s, e) = two_sum(r0, x);
        r0 = s;
        r1 += e;
    }
    (r0, r1)
}

impl Add<f64> for QuadDouble {
    type Output = Self;
    fn add(self, rhs: f64) -> Self::Output {
        let (t0, e) = two_sum(self.0, rhs);
        let (t1, e) = two_sum(self.1, e);
        let (t2, e) = two_sum(self.2, e);
        let (t3, t4) = two_sum(self.3, e);
        Self::renormalize(t0, t1, t2, t3, t4)
    }
}

fn double_accumulate(u: f64, v: f64, x: f64) -> (f64, f64, f64) {
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

impl Add<QuadDouble> for QuadDouble {
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

impl Sub for QuadDouble {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Neg for QuadDouble {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2, -self.3)
    }
}

impl Mul<f64> for QuadDouble {
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

impl Mul<QuadDouble> for QuadDouble {
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

impl Div<QuadDouble> for QuadDouble {
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

impl Index<usize> for QuadDouble {
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

impl From<QuadDouble> for f64 {
    fn from(x: QuadDouble) -> f64 {
        x.3 + x.2 + x.1 + x.0
    }
}

impl From<QuadDouble> for i64 {
    fn from(mut x: QuadDouble) -> i64 {
        let is_neg = x.0.is_sign_negative();
        if is_neg {
            x = -x;
        }
        let mut i = 0i64;
        for k in (1..64).rev() {
            let t = (k as f64).exp2();
            if x.0 >= t {
                x = x + -t;
                i += 1 << k;
            }
        }
        i += x.0.round() as i64;
        if is_neg {
            i = -i;
        }
        i
    }
}

impl From<f64> for QuadDouble {
    fn from(x: f64) -> Self {
        Self(x, 0., 0., 0.)
    }
}

impl Display for QuadDouble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            Decimal::from(self.0)
                + Decimal::from(self.1)
                + Decimal::from(self.2)
                + Decimal::from(self.3)
        )
    }
}

#[derive(Debug, Clone)]
pub enum ParseDoubleDoubleError {
    ParseFloatError(ParseFloatError),
    ParseDecimalError(super::decimal::convert::ParseDecimalError),
}

impl From<ParseFloatError> for ParseDoubleDoubleError {
    fn from(e: ParseFloatError) -> Self {
        Self::ParseFloatError(e)
    }
}

impl From<super::decimal::convert::ParseDecimalError> for ParseDoubleDoubleError {
    fn from(e: super::decimal::convert::ParseDecimalError) -> Self {
        Self::ParseDecimalError(e)
    }
}

impl FromStr for QuadDouble {
    type Err = ParseDoubleDoubleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f0: f64 = s.parse()?;
        let d1 = Decimal::from_str(s)? - Decimal::from(f0);
        let f1: f64 = d1.to_string().parse()?;
        let d2 = d1 - Decimal::from(f1);
        let f2: f64 = d2.to_string().parse()?;
        let d3 = d2 - Decimal::from(f2);
        let f3: f64 = d3.to_string().parse()?;
        Ok(Self::renormalize(f0, f1, f2, f3, 0.))
    }
}

impl Eq for QuadDouble {}
impl PartialOrd for QuadDouble {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QuadDouble {
    fn cmp(&self, other: &Self) -> Ordering {
        fn total_cmp(x: f64, y: f64) -> Ordering {
            let mut left = x.to_bits() as i64;
            let mut right = y.to_bits() as i64;
            left ^= (((left >> 63) as u64) >> 1) as i64;
            right ^= (((right >> 63) as u64) >> 1) as i64;
            left.cmp(&right)
        }
        total_cmp(self.0, other.0).then_with(|| total_cmp(self.1, other.1))
    }
}
impl Bounded for QuadDouble {
    fn maximum() -> Self {
        Self::from(<f64 as Bounded>::maximum())
    }
    fn minimum() -> Self {
        Self::from(<f64 as Bounded>::minimum())
    }
}

impl Zero for QuadDouble {
    fn zero() -> Self {
        Self::from(0.)
    }
    fn is_zero(&self) -> bool
    where
        Self: PartialEq,
    {
        self.0 == 0.
    }
}

impl One for QuadDouble {
    fn one() -> Self {
        Self::from(1.)
    }
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        self.0 == 1.
    }
}

impl IterScan for QuadDouble {
    type Output = Self;
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        iter.next().and_then(|s| s.parse().ok())
    }
}

impl QuadDouble {
    pub fn is_zero(&self) -> bool {
        self.0 == 0.
    }
    pub fn sqrt(self) -> Self {
        if self.is_zero() {
            return Self::from(0.);
        }
        let x = Self::from(1. / self.0.sqrt());
        let x = x + x * (Self::from(1.) - self * x * x).div2(2.);
        let x = x + x * (Self::from(1.) - self * x * x).div2(2.);
        let x = x + x * (Self::from(1.) - self * x * x).div2(2.);
        x * self
    }
    pub fn abs(self) -> Self {
        if self.0.is_sign_negative() {
            -self
        } else {
            self
        }
    }
    fn div2(self, rhs: f64) -> Self {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs, self.3 / rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let x = QuadDouble::from(1.234);
        assert_eq!(x.to_string(), "1.234");
        let x = QuadDouble::from(1.234e-10);
        assert_eq!(x.to_string(), "0.0000000001234");
        let x = QuadDouble::from(1.234e10);
        assert_eq!(x.to_string(), "12340000000");
        let x = QuadDouble::from(1.234e-10) + QuadDouble::from(1.234e10);
        assert_eq!(x.to_string(), "12340000000.0000000001234");
    }

    #[test]
    fn test_from_str() {
        let x = QuadDouble::from_str("1.234").unwrap();
        assert_eq!(x, QuadDouble::from(1.234));
        let x = QuadDouble::from_str("0.0000000001234").unwrap();
        assert_eq!(x, QuadDouble::from(1.234e-10));
        let x = QuadDouble::from_str("12340000000").unwrap();
        assert_eq!(x, QuadDouble::from(1.234e10));
        let x = QuadDouble::from_str("12340000000.0000000001234").unwrap();
        assert_eq!(x, QuadDouble::from(1.234e10) + QuadDouble::from(1.234e-10));
    }
}
