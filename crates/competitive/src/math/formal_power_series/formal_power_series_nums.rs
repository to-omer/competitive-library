use super::*;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};

impl<T, Multiplier> AddAssign<&T> for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> SubAssign<&T> for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> MulAssign<&T> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    fn mul_assign(&mut self, rhs: &T) {
        for x in self.data.iter_mut() {
            x.mul_assign(rhs);
        }
    }
}
impl<T, Multiplier> DivAssign<&T> for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> Add<&T> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn add(mut self, rhs: &T) -> Self::Output {
        self.add_assign(rhs);
        self
    }
}
impl<T, Multiplier> Sub<&T> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn sub(mut self, rhs: &T) -> Self::Output {
        self.sub_assign(rhs);
        self
    }
}
impl<T, Multiplier> Mul<&T> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn mul(mut self, rhs: &T) -> Self::Output {
        self.mul_assign(rhs);
        self
    }
}
impl<T, Multiplier> Div<&T> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn div(mut self, rhs: &T) -> Self::Output {
        self.div_assign(rhs);
        self
    }
}

impl<T, Multiplier> AddAssign<&Self> for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> SubAssign<&Self> for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> MulAssign<&Self> for FormalPowerSeries<T, Multiplier>
where
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    fn mul_assign(&mut self, rhs: &Self) {
        *self = Mul::mul(&*self, rhs);
    }
}
impl<T, Multiplier> DivAssign<&Self> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    fn div_assign(&mut self, rhs: &Self) {
        if self.length() < rhs.length() {
            self.data.clear();
        } else {
            let n = self.length() - rhs.length() + 1;
            *self = Mul::mul(&*self, &rhs.inv(n));
            todo!()
        }
    }
}
impl<T, Multiplier> RemAssign<&Self> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    fn rem_assign(&mut self, rhs: &Self) {
        self.sub_assign(&(&(&*self / rhs) * rhs));
    }
}

impl<T, Multiplier> Add for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Add::add(&self, &rhs)
    }
}
impl<T, Multiplier> Sub for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Sub::sub(&self, &rhs)
    }
}
impl<T, Multiplier> Mul for FormalPowerSeries<T, Multiplier>
where
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Mul::mul(&self, &rhs)
    }
}
impl<T, Multiplier> Div for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
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
        let mut res = (&self * &rhs.inv(n)).prefix(n);
        res.data.reverse();
        res
    }
}
impl<T, Multiplier> Rem for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Rem::rem(&self, &rhs)
    }
}
impl<T, Multiplier> Add<&FormalPowerSeries<T, Multiplier>> for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn add(self, rhs: &FormalPowerSeries<T, Multiplier>) -> Self::Output {
        let mut self_ = self.clone();
        self_.add_assign(rhs);
        self_
    }
}
impl<T, Multiplier> Sub<&FormalPowerSeries<T, Multiplier>> for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn sub(self, rhs: &FormalPowerSeries<T, Multiplier>) -> Self::Output {
        let mut self_ = self.clone();
        self_.sub_assign(rhs);
        self_
    }
}
impl<T, Multiplier> Mul<&FormalPowerSeries<T, Multiplier>> for &FormalPowerSeries<T, Multiplier>
where
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn mul(self, rhs: &FormalPowerSeries<T, Multiplier>) -> Self::Output {
        Multiplier::convolve(self, rhs)
    }
}
impl<T, Multiplier> Div<&FormalPowerSeries<T, Multiplier>> for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn div(self, rhs: &FormalPowerSeries<T, Multiplier>) -> Self::Output {
        Div::div(self.clone(), rhs.clone())
    }
}
impl<T, Multiplier> Rem<&FormalPowerSeries<T, Multiplier>> for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
    Multiplier: FormalPowerSeriesMultiplier<T = T>,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn rem(self, rhs: &FormalPowerSeries<T, Multiplier>) -> Self::Output {
        Rem::rem(self.clone(), rhs.clone())
    }
}

impl<T, Multiplier> Neg for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> Neg for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn neg(self) -> Self::Output {
        self.clone().neg()
    }
}

impl<T, Multiplier> ShrAssign<usize> for FormalPowerSeries<T, Multiplier>
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
impl<T, Multiplier> ShlAssign<usize> for FormalPowerSeries<T, Multiplier>
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

impl<T, Multiplier> Shr<usize> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn shr(mut self, rhs: usize) -> Self::Output {
        self.shr_assign(rhs);
        self
    }
}
impl<T, Multiplier> Shl<usize> for FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = Self;
    fn shl(mut self, rhs: usize) -> Self::Output {
        self.shl_assign(rhs);
        self
    }
}
impl<T, Multiplier> Shr<usize> for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, Multiplier>;
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
impl<T, Multiplier> Shl<usize> for &FormalPowerSeries<T, Multiplier>
where
    T: FormalPowerSeriesCoefficient,
{
    type Output = FormalPowerSeries<T, Multiplier>;
    fn shl(self, rhs: usize) -> Self::Output {
        let mut f = Self::Output::zeros(self.length() + rhs);
        for (i, x) in self.data.iter().cloned().enumerate().rev() {
            f[i + rhs] = x;
        }
        f
    }
}
