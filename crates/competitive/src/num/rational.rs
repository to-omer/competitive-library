use super::{Bounded, One, Signed, Unsigned, Zero};
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy)]
pub struct Rational<T>
where
    T: Signed,
{
    pub num: T,
    pub den: T,
}

impl<T> PartialEq for Rational<T>
where
    T: Signed,
{
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.den == other.den
    }
}

impl<T> Eq for Rational<T> where T: Signed {}

impl<T> PartialOrd for Rational<T>
where
    T: Signed,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Rational<T>
where
    T: Signed,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.den.is_zero(), other.den.is_zero()) {
            (true, true) => self.num.cmp(&other.num),
            (true, false) => self.num.cmp(&T::zero()),
            (false, true) => T::zero().cmp(&other.num),
            (false, false) => (self.num * other.den).cmp(&(self.den * other.num)),
        }
    }
}

impl<T> Debug for Rational<T>
where
    T: Signed + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{:?}", self.num, self.den)
    }
}

impl<T> Rational<T>
where
    T: Signed,
{
    pub fn new(num: T, den: T) -> Self {
        let g = num.abs().unsigned().gcd(den.abs().unsigned()).signed();
        let g = if den.is_negative() { -g } else { g };
        Self::new_unchecked(num / g, den / g)
    }
    pub fn new_unchecked(num: T, den: T) -> Self {
        Self { num, den }
    }
    pub fn abs(self) -> Self {
        Self::new_unchecked(self.num.abs(), self.den)
    }
    pub fn inner(self) -> T {
        self.num / self.den
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Rational<U>
    where
        U: Signed,
    {
        Rational::new(f(self.num), f(self.den))
    }
    pub fn map_unchecked<U>(self, mut f: impl FnMut(T) -> U) -> Rational<U>
    where
        U: Signed,
    {
        Rational::new_unchecked(f(self.num), f(self.den))
    }
}

impl<T> Bounded for Rational<T>
where
    T: Signed,
{
    fn maximum() -> Self {
        Self::new_unchecked(T::one(), T::zero())
    }
    fn minimum() -> Self {
        Self::new_unchecked(-T::one(), T::zero())
    }
}

impl<T> Zero for Rational<T>
where
    T: Signed,
{
    fn zero() -> Self {
        Self::new_unchecked(T::zero(), T::one())
    }
}
impl<T> One for Rational<T>
where
    T: Signed,
{
    fn one() -> Self {
        Self::new_unchecked(T::one(), T::one())
    }
}

impl<T> Add for Rational<T>
where
    T: Signed,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den + self.den * rhs.num, self.den * rhs.den)
    }
}
impl<T> Sub for Rational<T>
where
    T: Signed,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den - self.den * rhs.num, self.den * rhs.den)
    }
}
impl<T> Mul for Rational<T>
where
    T: Signed,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.num, self.den * rhs.den)
    }
}
impl<T> Div for Rational<T>
where
    T: Signed,
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den, self.den * rhs.num)
    }
}
impl<T> Neg for Rational<T>
where
    T: Signed,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new_unchecked(-self.num, self.den)
    }
}
impl<T> AddAssign for Rational<T>
where
    T: Signed,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T> SubAssign for Rational<T>
where
    T: Signed,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl<T> MulAssign for Rational<T>
where
    T: Signed,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl<T> DivAssign for Rational<T>
where
    T: Signed,
{
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}
