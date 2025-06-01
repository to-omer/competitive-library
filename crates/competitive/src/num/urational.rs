use super::{Bounded, One, Unsigned, Zero};
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Clone, Copy)]
pub struct URational<T>
where
    T: Unsigned,
{
    pub num: T,
    pub den: T,
}

impl<T> PartialEq for URational<T>
where
    T: Unsigned,
{
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.den == other.den
    }
}

impl<T> Eq for URational<T> where T: Unsigned {}

impl<T> PartialOrd for URational<T>
where
    T: Unsigned,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for URational<T>
where
    T: Unsigned,
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

impl<T> Debug for URational<T>
where
    T: Unsigned + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{:?}", self.num, self.den)
    }
}

impl<T> URational<T>
where
    T: Unsigned,
{
    pub fn new(num: T, den: T) -> Self {
        let g = num.gcd(den);
        Self::new_unchecked(num / g, den / g)
    }
    pub fn new_unchecked(num: T, den: T) -> Self {
        Self { num, den }
    }
    pub fn eval(self) -> T {
        self.num / self.den
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> URational<U>
    where
        U: Unsigned,
    {
        URational::new(f(self.num), f(self.den))
    }
    pub fn map_unchecked<U>(self, mut f: impl FnMut(T) -> U) -> URational<U>
    where
        U: Unsigned,
    {
        URational::new_unchecked(f(self.num), f(self.den))
    }
    pub fn map_eval<U>(self, mut f: impl FnMut(T) -> U) -> <U as Div>::Output
    where
        U: Div,
    {
        f(self.num) / f(self.den)
    }
}

impl<T> Bounded for URational<T>
where
    T: Unsigned,
{
    fn maximum() -> Self {
        Self::new_unchecked(T::one(), T::zero())
    }
    fn minimum() -> Self {
        Self::zero()
    }
}

impl<T> Zero for URational<T>
where
    T: Unsigned,
{
    fn zero() -> Self {
        Self::new_unchecked(T::zero(), T::one())
    }
}
impl<T> One for URational<T>
where
    T: Unsigned,
{
    fn one() -> Self {
        Self::new_unchecked(T::one(), T::one())
    }
}

impl<T> Add for URational<T>
where
    T: Unsigned,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den + self.den * rhs.num, self.den * rhs.den)
    }
}
impl<T> Sub for URational<T>
where
    T: Unsigned,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den - self.den * rhs.num, self.den * rhs.den)
    }
}
impl<T> Mul for URational<T>
where
    T: Unsigned,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.num, self.den * rhs.den)
    }
}
impl<T> Div for URational<T>
where
    T: Unsigned,
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.num * rhs.den, self.den * rhs.num)
    }
}
impl<T> AddAssign for URational<T>
where
    T: Unsigned,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T> SubAssign for URational<T>
where
    T: Unsigned,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl<T> MulAssign for URational<T>
where
    T: Unsigned,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl<T> DivAssign for URational<T>
where
    T: Unsigned,
{
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}
