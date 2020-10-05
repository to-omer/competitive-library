#[snippet::entry("Complex")]
use std::ops::{Add, Div, Mul, Neg, Sub};
#[snippet::entry("Complex")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}
#[snippet::entry("Complex")]
impl<T> Complex<T> {
    #[inline]
    pub fn new(re: T, im: T) -> Self {
        Complex { re, im }
    }
    #[inline]
    pub fn transpose(self) -> Self {
        Complex {
            re: self.im,
            im: self.re,
        }
    }
}
#[snippet::entry("Complex")]
impl<T: Neg<Output = T>> Complex<T> {
    #[inline]
    pub fn conjugate(self) -> Self {
        Self::new(self.re, -self.im)
    }
}
#[snippet::entry("Complex")]
impl<T: Add<Output = T> + Mul<Output = T>> Complex<T> {
    #[inline]
    pub fn dot(self, rhs: Self) -> T {
        self.re * rhs.re + self.im * rhs.im
    }
}
#[snippet::entry("Complex")]
impl<T: Sub<Output = T> + Mul<Output = T>> Complex<T> {
    #[inline]
    pub fn cross(self, rhs: Self) -> T {
        self.re * rhs.im - self.im * rhs.re
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Add<Output = T> + Mul<Output = T>> Complex<T> {
    #[inline]
    pub fn norm(self) -> T {
        self.re * self.re + self.im * self.im
    }
}
#[snippet::entry("Complex")]
impl Complex<f64> {
    #[inline]
    pub fn polar(r: f64, theta: f64) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }
    #[inline]
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    #[inline]
    pub fn unit(self) -> Self {
        self / self.abs()
    }
    #[inline]
    pub fn angle(self) -> f64 {
        self.im.atan2(self.re)
    }
}
#[snippet::entry("Complex")]
impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Add<Output = T>> Add<T> for Complex<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self::new(self.re + rhs, self.im + rhs)
    }
}
#[snippet::entry("Complex")]
impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Sub<Output = T>> Sub<T> for Complex<T> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self::new(self.re - rhs, self.im - rhs)
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T>> Mul for Complex<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Mul<Output = T>> Mul<T> for Complex<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.re * rhs, self.im * rhs)
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>> Div
    for Complex<T>
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let d = rhs.re * rhs.re + rhs.im * rhs.im;
        Self::new(
            (self.re * rhs.re + self.im * rhs.im) / d,
            (self.im * rhs.re - self.re * rhs.im) / d,
        )
    }
}
#[snippet::entry("Complex")]
impl<T: Copy + Div<Output = T>> Div<T> for Complex<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.re / rhs, self.im / rhs)
    }
}
#[snippet::entry("Complex")]
impl<T: Neg<Output = T>> Neg for Complex<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}
