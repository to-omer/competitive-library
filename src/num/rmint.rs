//! runtime modint

#[cargo_snippet::snippet("RMInt")]
static mut RMODULO: u32 = 1_000_000_007;
#[cargo_snippet::snippet("RMInt")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RMInt {
    x: u32,
}
#[cargo_snippet::snippet("RMInt")]
impl RMInt {
    #[inline]
    pub fn new(x: u32) -> Self {
        Self {
            x: x % Self::get_modulus(),
        }
    }
    #[inline]
    pub fn new_unchecked(x: u32) -> Self {
        Self { x }
    }
    #[inline]
    pub fn one() -> Self {
        Self::new_unchecked(1)
    }
    #[inline]
    pub fn zero() -> Self {
        Self::new_unchecked(0)
    }
    #[inline]
    pub fn get_modulus() -> u32 {
        unsafe { RMODULO }
    }
    #[inline]
    pub fn set_modulus(m: u32) {
        unsafe {
            RMODULO = m;
        }
    }
    #[inline]
    pub fn pow(mut self, mut y: usize) -> Self {
        let mut x = Self::one();
        while y > 0 {
            if y & 1 == 1 {
                x = x * self;
            }
            self = self * self;
            y >>= 1;
        }
        x
    }
    #[inline]
    pub fn inv(self) -> Self {
        let (mut x, mut s, mut t, mut u) = (1i64, self.x as i64, Self::get_modulus() as i64, 0i64);
        while t != 0 {
            let k = s / t;
            s -= k * t;
            std::mem::swap(&mut s, &mut t);
            x -= k * u;
            std::mem::swap(&mut x, &mut u);
        }
        x %= Self::get_modulus() as i64;
        if x < 0 {
            x += Self::get_modulus() as i64;
        }
        Self::new(x as u32)
    }
}
#[cargo_snippet::snippet("RMInt")]
pub mod modu32_impls {
    use super::*;
    use std::{
        fmt,
        iter::{Product, Sum},
        num::ParseIntError,
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
        str::FromStr,
    };
    impl From<u32> for RMInt {
        #[inline]
        fn from(x: u32) -> Self {
            Self::new(x)
        }
    }
    impl From<u64> for RMInt {
        #[inline]
        fn from(x: u64) -> Self {
            Self::new_unchecked((x % Self::get_modulus() as u64) as u32)
        }
    }
    impl Add for RMInt {
        type Output = Self;
        #[inline]
        fn add(self, rhs: Self) -> Self::Output {
            let mut x = self.x + rhs.x;
            if x >= Self::get_modulus() {
                x -= Self::get_modulus();
            }
            Self::new_unchecked(x)
        }
    }
    impl Sub for RMInt {
        type Output = Self;
        #[inline]
        fn sub(self, rhs: Self) -> Self::Output {
            if self.x < rhs.x {
                Self::new_unchecked(self.x + Self::get_modulus() - rhs.x)
            } else {
                Self::new_unchecked(self.x - rhs.x)
            }
        }
    }
    impl Mul for RMInt {
        type Output = Self;
        #[inline]
        fn mul(self, rhs: Self) -> Self::Output {
            Self::new_unchecked((self.x as u64 * rhs.x as u64 % Self::get_modulus() as u64) as u32)
        }
    }
    impl Div for RMInt {
        type Output = Self;
        #[inline]
        fn div(self, rhs: Self) -> Self::Output {
            self * rhs.inv()
        }
    }
    impl Neg for RMInt {
        type Output = Self;
        #[inline]
        fn neg(self) -> Self::Output {
            if self.x == 0 {
                Self::zero()
            } else {
                Self::new_unchecked(Self::get_modulus() - self.x)
            }
        }
    }
    impl Sum for RMInt {
        #[inline]
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Self::zero(), Add::add)
        }
    }
    impl Product for RMInt {
        #[inline]
        fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Self::one(), Mul::mul)
        }
    }
    impl<'a> Sum<&'a RMInt> for RMInt {
        #[inline]
        fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
            iter.fold(Self::zero(), Add::add)
        }
    }
    impl<'a> Product<&'a RMInt> for RMInt {
        #[inline]
        fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
            iter.fold(Self::one(), Mul::mul)
        }
    }
    impl fmt::Display for RMInt {
        fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
            write!(f, "{}", self.x)
        }
    }
    impl FromStr for RMInt {
        type Err = ParseIntError;
        #[inline]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u32>().map(Self::new)
        }
    }
    macro_rules! modu32_ref_binop {
        ($imp:ident, $method:ident, $t:ty) => {
            impl<'a> $imp<$t> for &'a $t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: $t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(*self, other)
                }
            }
            impl $imp<&$t> for $t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(self, *other)
                }
            }
            impl $imp<&$t> for &$t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(*self, *other)
                }
            }
        };
    }
    modu32_ref_binop!(Add, add, RMInt);
    modu32_ref_binop!(Sub, sub, RMInt);
    modu32_ref_binop!(Mul, mul, RMInt);
    modu32_ref_binop!(Div, div, RMInt);
    macro_rules! modu32_ref_unop {
        ($imp:ident, $method:ident, $t:ty) => {
            impl $imp for &$t {
                type Output = <$t as $imp>::Output;
                #[inline]
                fn $method(self) -> <$t as $imp>::Output {
                    $imp::$method(*self)
                }
            }
        };
    }
    modu32_ref_unop!(Neg, neg, RMInt);
    macro_rules! modu32_ref_op_assign {
        ($imp:ident, $method:ident, $t:ty, $fromimp:ident, $frommethod:ident) => {
            impl $imp<$t> for $t {
                #[inline]
                fn $method(&mut self, rhs: $t) {
                    *self = $fromimp::$frommethod(*self, rhs);
                }
            }
            impl $imp<&$t> for $t {
                #[inline]
                fn $method(&mut self, other: &$t) {
                    $imp::$method(self, *other);
                }
            }
        };
    }
    modu32_ref_op_assign!(AddAssign, add_assign, RMInt, Add, add);
    modu32_ref_op_assign!(SubAssign, sub_assign, RMInt, Sub, sub);
    modu32_ref_op_assign!(MulAssign, mul_assign, RMInt, Mul, mul);
    modu32_ref_op_assign!(DivAssign, div_assign, RMInt, Div, div);
}

use crate::algebra::operations::{AdditiveIdentity, MultiplicativeIdentity};
impl_additive_identity!(RMInt, Self::zero());
impl_multiplicative_identity!(RMInt, Self::one());
