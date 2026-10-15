use super::{One, Signed, Zero};
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Neg, Sub},
};

/// A reduced `numerator / 2^exponent`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dyadic<T> {
    numerator: T,
    exponent: u32,
}

impl<T> Dyadic<T> {
    pub fn numerator(self) -> T {
        self.numerator
    }

    pub fn exponent(self) -> u32 {
        self.exponent
    }
}

impl<T> Dyadic<T>
where
    T: Signed,
{
    pub fn new(mut numerator: T, mut exponent: u32) -> Self {
        let two = T::one() + T::one();
        if numerator.is_zero() {
            exponent = 0;
        }
        while exponent > 0 && (numerator % two).is_zero() {
            numerator /= two;
            exponent -= 1;
        }
        Self {
            numerator,
            exponent,
        }
    }

    /// The simplest number strictly between the bounds; absent bounds are unbounded.
    /// Returns `None` for an empty interval.
    pub fn simplest_between(left: Option<Self>, right: Option<Self>) -> Option<Self> {
        if left.zip(right).is_some_and(|(l, r)| l >= r) {
            return None;
        }
        if left.is_none_or(|l| l.numerator < T::zero())
            && right.is_none_or(|r| r.numerator > T::zero())
        {
            return Some(Self::zero());
        }
        let negative = right.is_some_and(|r| r.numerator <= T::zero());
        let (lower, upper) = if negative {
            (-right.unwrap(), left.map(Neg::neg))
        } else {
            (left.unwrap(), right)
        };
        let two = T::one() + T::one();
        for exponent in 0.. {
            let floor = if exponent < lower.exponent {
                lower.numerator / two.pow(lower.exponent - exponent)
            } else {
                lower.numerator * two.pow(exponent - lower.exponent)
            };
            let value = Self::new(floor + T::one(), exponent);
            if upper.is_none_or(|u| value < u) {
                return Some(if negative { -value } else { value });
            }
        }
        unreachable!()
    }
}

impl<T> From<T> for Dyadic<T>
where
    T: Signed,
{
    fn from(numerator: T) -> Self {
        Self::new(numerator, 0)
    }
}
impl<T> Zero for Dyadic<T>
where
    T: Signed,
{
    fn zero() -> Self {
        T::zero().into()
    }
}
impl<T> One for Dyadic<T>
where
    T: Signed,
{
    fn one() -> Self {
        T::one().into()
    }
}
impl<T> Default for Dyadic<T>
where
    T: Signed,
{
    fn default() -> Self {
        Self::zero()
    }
}
impl<T> Neg for Dyadic<T>
where
    T: Signed,
{
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            numerator: -self.numerator,
            exponent: self.exponent,
        }
    }
}
impl<T> Add for Dyadic<T>
where
    T: Signed,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let exponent = self.exponent.max(rhs.exponent);
        let two = T::one() + T::one();
        let left = self.numerator * two.pow(exponent - self.exponent);
        let right = rhs.numerator * two.pow(exponent - rhs.exponent);
        Self::new(left + right, exponent)
    }
}
impl<T> Sub for Dyadic<T>
where
    T: Signed,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + -rhs
    }
}
impl<T> Ord for Dyadic<T>
where
    T: Signed,
{
    fn cmp(&self, rhs: &Self) -> Ordering {
        let exponent = self.exponent.max(rhs.exponent);
        let two = T::one() + T::one();
        let left = self.numerator * two.pow(exponent - self.exponent);
        let right = rhs.numerator * two.pow(exponent - rhs.exponent);
        left.cmp(&right)
    }
}
impl<T> PartialOrd for Dyadic<T>
where
    T: Signed,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl<T> fmt::Display for Dyadic<T>
where
    T: Signed,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.exponent == 0 {
            write!(f, "{}", self.numerator)
        } else {
            let two = T::one() + T::one();
            write!(f, "{}/{}", self.numerator, two.pow(self.exponent))
        }
    }
}
impl<T> fmt::Debug for Dyadic<T>
where
    T: Signed,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::integer_pairs};

    #[test]
    fn arithmetic_and_order() {
        let mut rng = Xorshift::default();
        const MAX_EXPONENT: u32 = i32::BITS / 2;
        let bound = i32::MAX >> (MAX_EXPONENT + 1);
        for exponent in 0..=MAX_EXPONENT {
            for (left, right) in integer_pairs(-bound..=bound).chain(
                rng.random_iter((-bound..=bound, -bound..=bound))
                    .take(60_000),
            ) {
                for other_exponent in [0, exponent, MAX_EXPONENT] {
                    let x = Dyadic::new(left, exponent);
                    let y = Dyadic::new(right, other_exponent);
                    let scaled = |v: Dyadic<i32>| v.numerator() << (MAX_EXPONENT - v.exponent());
                    let a = left << (MAX_EXPONENT - exponent);
                    let b = right << (MAX_EXPONENT - other_exponent);
                    assert_eq!((scaled(x), scaled(y)), (a, b));
                    assert_eq!(x.cmp(&y), a.cmp(&b));
                    assert_eq!(scaled(x + y), a + b);
                    assert_eq!(scaled(x - y), a - b);
                    assert_eq!(scaled(-x), -a);
                    for v in [x, y, x + y, x - y, -x] {
                        assert!(v.exponent() == 0 || v.numerator() % 2 != 0);
                    }
                    let wide = Dyadic::new(i64::from(left), exponent)
                        + Dyadic::new(i64::from(right), other_exponent);
                    assert_eq!(
                        (i64::from((x + y).numerator()), (x + y).exponent()),
                        (wide.numerator(), wide.exponent())
                    );
                }
            }
        }
    }

    #[test]
    fn simplest_between() {
        // Bounds on a 1/8 grid. Enumerate by denominator, then distance from zero.
        let bounds: Vec<_> = std::iter::once(None).chain((-32..=32).map(Some)).collect();
        for &left in &bounds {
            for &right in &bounds {
                let actual = Dyadic::simplest_between(
                    left.map(|n| Dyadic::new(n, 3)),
                    right.map(|n| Dyadic::new(n, 3)),
                );
                let expected = (0..=4).find_map(|e| {
                    (-80i32..=80)
                        .filter(|&n| {
                            left.is_none_or(|l| l * (1 << e) < n * 8)
                                && right.is_none_or(|r| n * 8 < r * (1 << e))
                        })
                        .min_by_key(|n| n.abs())
                        .map(|n| Dyadic::new(n, e))
                });
                assert_eq!(actual, expected, "{left:?} .. {right:?}");
            }
        }
    }
}
