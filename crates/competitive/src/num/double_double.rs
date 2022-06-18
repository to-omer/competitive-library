#![allow(clippy::suspicious_arithmetic_impl)]

use super::Bounded;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    num::ParseFloatError,
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DoubleDouble(f64, f64);

impl Eq for DoubleDouble {}
impl PartialOrd for DoubleDouble {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        fn total_cmp(x: &f64, y: &f64) -> Ordering {
            let mut left = x.to_bits() as i64;
            let mut right = y.to_bits() as i64;
            left ^= (((left >> 63) as u64) >> 1) as i64;
            right ^= (((right >> 63) as u64) >> 1) as i64;
            left.cmp(&right)
        }
        Some(total_cmp(&self.0, &other.0).then_with(|| total_cmp(&self.1, &other.1)))
    }
}
impl Ord for DoubleDouble {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl Bounded for DoubleDouble {
    fn maximum() -> Self {
        DoubleDouble::from(<f64 as Bounded>::maximum())
    }
    fn minimum() -> Self {
        DoubleDouble::from(<f64 as Bounded>::minimum())
    }
}

impl DoubleDouble {
    fn renormalize(a0: f64, a1: f64, a2: f64) -> Self {
        let (s, t2) = quick_two_sum(a1, a2);
        let (mut s, t1) = quick_two_sum(a0, s);
        let mut b = (s, t1);
        if t1 != 0. {
            b.0 = s;
            s = t1;
        }
        let (s, e) = quick_two_sum(s, t2);
        if e != 0. {
            if t1 != 0. {
                b.1 = s;
            } else {
                b.0 = s;
            }
        }
        Self(b.0, b.1)
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

fn three_two_sum(a: f64, b: f64, c: f64) -> (f64, f64) {
    let (u, v) = two_sum(a, b);
    let (r0, w) = two_sum(u, c);
    let r1 = v + w;
    (r0, r1)
}

impl Add<f64> for DoubleDouble {
    type Output = Self;
    fn add(self, rhs: f64) -> Self::Output {
        let (t0, e) = two_sum(self.0, rhs);
        let (t1, t2) = two_sum(self.1, e);
        Self::renormalize(t0, t1, t2)
    }
}

impl Add<DoubleDouble> for DoubleDouble {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (t0, e) = two_sum(self.0, rhs.0);
        let (t1, t2) = three_two_sum(self.1, rhs.1, e);
        Self::renormalize(t0, t1, t2)
    }
}

impl Sub for DoubleDouble {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Neg for DoubleDouble {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

impl Mul<f64> for DoubleDouble {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let (t0, e0) = two_prod(self.0, rhs);
        let p1 = self.1 * rhs;
        let (t1, t2) = two_sum(p1, e0);
        Self::renormalize(t0, t1, t2)
    }
}

impl Mul<DoubleDouble> for DoubleDouble {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let (t0, q00) = two_prod(self.0, rhs.0);
        let (p01, q01) = two_prod(self.0, rhs.1);
        let (p10, q10) = two_prod(self.1, rhs.0);
        let p11 = self.1 * rhs.1;
        let (t1, e1) = three_two_sum(q00, p01, p10);
        let t2 = e1 + q01 + q10 + p11;
        Self::renormalize(t0, t1, t2)
    }
}

impl Div<DoubleDouble> for DoubleDouble {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let q0 = self.0 / rhs.0;
        let r = self - rhs * q0;
        let q1 = r.0 / rhs.0;
        let r = r - rhs * q1;
        let q2 = r.0 / rhs.0;
        Self::renormalize(q0, q1, q2)
    }
}

impl From<DoubleDouble> for f64 {
    fn from(x: DoubleDouble) -> f64 {
        x.1 + x.0
    }
}

impl From<DoubleDouble> for i64 {
    fn from(mut x: DoubleDouble) -> i64 {
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

impl From<f64> for DoubleDouble {
    fn from(x: f64) -> Self {
        Self(x, 0.)
    }
}

impl Display for DoubleDouble {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "{}", self.1 + self.0)
    }
}

impl FromStr for DoubleDouble {
    type Err = ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>().map(Self::from)
    }
}

impl DoubleDouble {
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
        Self(self.0 / rhs, self.1 / rhs)
    }
}
