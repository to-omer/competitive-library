#![allow(clippy::suspicious_arithmetic_impl, clippy::suspicious_op_assign_impl)]

use super::*;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};

impl<T, C> AddAssign<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn add_assign(&mut self, rhs: &T) {
        if self.length() == 0 {
            self.data.push(T::zero());
        }
        self.data[0].add_assign(rhs);
    }
}
impl<T, C> SubAssign<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn sub_assign(&mut self, rhs: &T) {
        if self.length() == 0 {
            self.data.push(T::zero());
        }
        self.data[0].sub_assign(rhs);
    }
}
impl<T, C> MulAssign<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn mul_assign(&mut self, rhs: &T) {
        for x in self.data.iter_mut() {
            x.mul_assign(rhs);
        }
    }
}
impl<T, C> DivAssign<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn div_assign(&mut self, rhs: &T) {
        let rinv = T::one() / rhs;
        for x in self.data.iter_mut() {
            x.mul_assign(&rinv);
        }
    }
}
impl<T, C> Add<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn add(mut self, rhs: &T) -> Self::Output {
        self.add_assign(rhs);
        self
    }
}
impl<T, C> Sub<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn sub(mut self, rhs: &T) -> Self::Output {
        self.sub_assign(rhs);
        self
    }
}
impl<T, C> Mul<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn mul(mut self, rhs: &T) -> Self::Output {
        self.mul_assign(rhs);
        self
    }
}
impl<T, C> Div<&T> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn div(mut self, rhs: &T) -> Self::Output {
        self.div_assign(rhs);
        self
    }
}

impl<T, C> AddAssign<&Self> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn add_assign(&mut self, rhs: &Self) {
        if self.length() < rhs.length() {
            self.data.resize_with(rhs.length(), Zero::zero);
        }
        for (x, y) in self.data.iter_mut().zip(rhs.data.iter()) {
            x.add_assign(y);
        }
    }
}
impl<T, C> SubAssign<&Self> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn sub_assign(&mut self, rhs: &Self) {
        if self.length() < rhs.length() {
            self.data.resize_with(rhs.length(), Zero::zero);
        }
        for (x, y) in self.data.iter_mut().zip(rhs.data.iter()) {
            x.sub_assign(y);
        }
    }
}
impl<T, C> MulAssign<&Self> for FormalPowerSeries<T, C>
where
    T: Clone,
    C: ConvolveSteps<T = Vec<T>>,
{
    fn mul_assign(&mut self, rhs: &Self) {
        *self = Mul::mul(&*self, rhs);
    }
}
impl<T, C> DivAssign<&Self> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    fn div_assign(&mut self, rhs: &Self) {
        *self = Div::div(&*self, rhs);
    }
}
impl<T, C> RemAssign<&Self> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    fn rem_assign(&mut self, rhs: &Self) {
        self.sub_assign(&(&(&*self / rhs) * rhs));
    }
}

impl<T, C> Add for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.add_assign(&rhs);
        self
    }
}
impl<T, C> Sub for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self.sub_assign(&rhs);
        self
    }
}
impl<T, C> Mul for FormalPowerSeries<T, C>
where
    C: ConvolveSteps<T = Vec<T>>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_vec(C::convolve(self.data, rhs.data))
    }
}
impl<T, C> Div for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    type Output = Self;
    fn div(mut self, mut rhs: Self) -> Self::Output {
        while self.data.last().map_or(false, |x| x.is_zero()) {
            self.data.pop();
        }
        while rhs.data.last().map_or(false, |x| x.is_zero()) {
            rhs.data.pop();
        }
        if self.length() < rhs.length() {
            return Self::zero();
        }
        self.data.reverse();
        rhs.data.reverse();
        let n = self.length() - rhs.length() + 1;
        let mut res = (self * rhs.inv(n)).prefix_inplace(n);
        res.data.reverse();
        res
    }
}
impl<T, C> Rem for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        let mut res = self.clone() - self / rhs.clone() * rhs;
        while res.data.last().map_or(false, |x| x.is_zero()) {
            res.data.pop();
        }
        res
    }
}
impl<T, C> Add<&FormalPowerSeries<T, C>> for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, C>;
    fn add(self, rhs: &FormalPowerSeries<T, C>) -> Self::Output {
        Add::add(self.clone(), rhs.clone())
    }
}
impl<T, C> Sub<&FormalPowerSeries<T, C>> for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, C>;
    fn sub(self, rhs: &FormalPowerSeries<T, C>) -> Self::Output {
        Sub::sub(self.clone(), rhs.clone())
    }
}
impl<T, C> Mul<&FormalPowerSeries<T, C>> for &FormalPowerSeries<T, C>
where
    T: Clone,
    C: ConvolveSteps<T = Vec<T>>,
{
    type Output = FormalPowerSeries<T, C>;
    fn mul(self, rhs: &FormalPowerSeries<T, C>) -> Self::Output {
        Mul::mul(self.clone(), rhs.clone())
    }
}
impl<T, C> Div<&FormalPowerSeries<T, C>> for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    type Output = FormalPowerSeries<T, C>;
    fn div(self, rhs: &FormalPowerSeries<T, C>) -> Self::Output {
        Div::div(self.clone(), rhs.clone())
    }
}
impl<T, C> Rem<&FormalPowerSeries<T, C>> for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
    C: ConvolveSteps<T = Vec<T>>,
{
    type Output = FormalPowerSeries<T, C>;
    fn rem(self, rhs: &FormalPowerSeries<T, C>) -> Self::Output {
        Rem::rem(self.clone(), rhs.clone())
    }
}

impl<T, C> Neg for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        for x in self.data.iter_mut() {
            *x = -x.clone();
        }
        self
    }
}
impl<T, C> Neg for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, C>;
    fn neg(self) -> Self::Output {
        self.clone().neg()
    }
}

impl<T, C> ShrAssign<usize> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn shr_assign(&mut self, rhs: usize) {
        if self.length() <= rhs {
            *self = Self::zero();
        } else {
            for i in rhs..self.length() {
                self[i - rhs] = self[i].clone();
            }
            self.truncate(self.length() - rhs);
        }
    }
}
impl<T, C> ShlAssign<usize> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    fn shl_assign(&mut self, rhs: usize) {
        let n = self.length();
        self.resize(n + rhs);
        for i in (0..n).rev() {
            self[i + rhs] = self[i].clone();
        }
        for i in 0..rhs {
            self[i] = T::zero();
        }
    }
}

impl<T, C> Shr<usize> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn shr(mut self, rhs: usize) -> Self::Output {
        self.shr_assign(rhs);
        self
    }
}
impl<T, C> Shl<usize> for FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn shl(mut self, rhs: usize) -> Self::Output {
        self.shl_assign(rhs);
        self
    }
}
impl<T, C> Shr<usize> for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, C>;
    fn shr(self, rhs: usize) -> Self::Output {
        if self.length() <= rhs {
            Self::Output::zero()
        } else {
            let mut f = Self::Output::zeros(self.length() - rhs);
            for i in rhs..self.length() {
                f[i - rhs] = self[i].clone();
            }
            f
        }
    }
}
impl<T, C> Shl<usize> for &FormalPowerSeries<T, C>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, C>;
    fn shl(self, rhs: usize) -> Self::Output {
        let mut f = Self::Output::zeros(self.length() + rhs);
        for (i, x) in self.data.iter().cloned().enumerate().rev() {
            f[i + rhs] = x;
        }
        f
    }
}
