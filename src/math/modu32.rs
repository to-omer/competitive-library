#[cargo_snippet::snippet("Modu32")]
pub trait Modulo: Copy {
    const MODULO: u32;
    #[inline]
    fn modulo(x: u32) -> u32 {
        x % Self::MODULO
    }
}
#[cargo_snippet::snippet("Modu32")]
pub mod modulos {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Modulo1000000007 {}
    impl Modulo for Modulo1000000007 {
        const MODULO: u32 = 1_000_000_007;
    }
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Modulo1000000009 {}
    impl Modulo for Modulo1000000009 {
        const MODULO: u32 = 1_000_000_009;
    }
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Modulo998244353 {}
    impl Modulo for Modulo998244353 {
        const MODULO: u32 = 998_244_353;
    }
}
#[cargo_snippet::snippet("Modu32")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modu32<M = modulos::Modulo1000000007>
where
    M: Modulo,
{
    x: u32,
    phantom: std::marker::PhantomData<M>,
}
#[cargo_snippet::snippet("Modu32")]
impl<M: Modulo> Modu32<M> {
    #[inline]
    pub fn new(x: u32) -> Self {
        Self {
            x: M::modulo(x),
            phantom: std::marker::PhantomData,
        }
    }
    #[inline]
    pub fn new_unchecked(x: u32) -> Self {
        Self {
            x,
            phantom: std::marker::PhantomData,
        }
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
    pub fn get_mod() -> u32 {
        M::MODULO
    }
    #[inline]
    pub fn pow(mut self, mut y: usize) -> Self {
        let mut x = Self::one();
        while y > 0 {
            if y & 1 == 1 {
                x *= self;
            }
            self *= self;
            y >>= 1;
        }
        x
    }
    #[inline]
    pub fn inv(self) -> Self {
        let mut a = self.x;
        let (mut b, mut u, mut s) = (M::MODULO, 1, 0);
        let k = a.trailing_zeros();
        a >>= k;
        for _ in 0..k {
            if u & 1 == 1 {
                u += M::MODULO;
            }
            u /= 2;
        }
        while a != b {
            if b < a {
                std::mem::swap(&mut a, &mut b);
                std::mem::swap(&mut u, &mut s);
            }
            b -= a;
            if s < u {
                s += M::MODULO;
            }
            s -= u;
            let k = b.trailing_zeros();
            b >>= k;
            for _ in 0..k {
                if s & 1 == 1 {
                    s += M::MODULO;
                }
                s /= 2;
            }
        }
        Self::new_unchecked(s)
    }
}
#[cargo_snippet::snippet("Modu32")]
pub mod modu32_impl {
    use super::*;
    use std::{
        fmt,
        iter::{Product, Sum},
        num::ParseIntError,
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
        str::FromStr,
    };
    impl<M: Modulo> From<u32> for Modu32<M> {
        #[inline]
        fn from(x: u32) -> Self {
            Self::new(x)
        }
    }
    impl<M: Modulo> From<u64> for Modu32<M> {
        #[inline]
        fn from(x: u64) -> Self {
            Self::new_unchecked((x % M::MODULO as u64) as u32)
        }
    }
    impl<M: Modulo> Add for Modu32<M> {
        type Output = Self;
        #[inline]
        fn add(self, rhs: Self) -> Self::Output {
            let mut x = self.x + rhs.x;
            if x >= M::MODULO {
                x -= M::MODULO;
            }
            Self::new_unchecked(x)
        }
    }
    impl<M: Modulo> Sub for Modu32<M> {
        type Output = Self;
        #[inline]
        fn sub(self, rhs: Self) -> Self::Output {
            if self.x < rhs.x {
                Self::new_unchecked(self.x + M::MODULO - rhs.x)
            } else {
                Self::new_unchecked(self.x - rhs.x)
            }
        }
    }
    impl<M: Modulo> Mul for Modu32<M> {
        type Output = Self;
        #[inline]
        fn mul(self, rhs: Self) -> Self::Output {
            Self::new_unchecked((self.x as u64 * rhs.x as u64 % M::MODULO as u64) as u32)
        }
    }
    impl<M: Modulo> Div for Modu32<M> {
        type Output = Self;
        #[inline]
        fn div(self, rhs: Self) -> Self::Output {
            self * rhs.inv()
        }
    }
    impl<M: Modulo> Neg for Modu32<M> {
        type Output = Self;
        #[inline]
        fn neg(self) -> Self::Output {
            if self.x == 0 {
                Self::zero()
            } else {
                Self::new_unchecked(M::MODULO - self.x)
            }
        }
    }
    impl<M: Modulo> Sum for Modu32<M> {
        #[inline]
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Self::zero(), Add::add)
        }
    }
    impl<M: Modulo> Product for Modu32<M> {
        #[inline]
        fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Self::one(), Mul::mul)
        }
    }
    impl<'a, M: Modulo + 'a> Sum<&'a Modu32<M>> for Modu32<M> {
        #[inline]
        fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
            iter.fold(Self::zero(), Add::add)
        }
    }
    impl<'a, M: Modulo + 'a> Product<&'a Modu32<M>> for Modu32<M> {
        #[inline]
        fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
            iter.fold(Self::one(), Mul::mul)
        }
    }
    impl<M: Modulo> fmt::Display for Modu32<M> {
        fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
            write!(f, "{}", self.x)
        }
    }
    impl<M: Modulo> FromStr for Modu32<M> {
        type Err = ParseIntError;
        #[inline]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u32>().map(Self::new_unchecked)
        }
    }
    macro_rules! modu32_ref_binop {
        ($imp:ident, $method:ident, $t:ty) => {
            impl<'a, M: Modulo> $imp<$t> for &'a $t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: $t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(*self, other)
                }
            }
            impl<M: Modulo> $imp<&$t> for $t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(self, *other)
                }
            }
            impl<M: Modulo> $imp<&$t> for &$t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(*self, *other)
                }
            }
        };
    }
    modu32_ref_binop!(Add, add, Modu32<M>);
    modu32_ref_binop!(Sub, sub, Modu32<M>);
    modu32_ref_binop!(Mul, mul, Modu32<M>);
    modu32_ref_binop!(Div, div, Modu32<M>);
    macro_rules! modu32_ref_unop {
        ($imp:ident, $method:ident, $t:ty) => {
            impl<M: Modulo> $imp for &$t {
                type Output = <$t as $imp>::Output;
                #[inline]
                fn $method(self) -> <$t as $imp>::Output {
                    $imp::$method(*self)
                }
            }
        };
    }
    modu32_ref_unop!(Neg, neg, Modu32<M>);
    macro_rules! modu32_ref_op_assign {
        ($imp:ident, $method:ident, $t:ty, $fromimp:ident, $frommethod:ident) => {
            impl<M: Modulo> $imp<$t> for $t {
                #[inline]
                fn $method(&mut self, rhs: $t) {
                    *self = $fromimp::$frommethod(*self, rhs);
                }
            }
            impl<M: Modulo> $imp<&$t> for $t {
                #[inline]
                fn $method(&mut self, other: &$t) {
                    $imp::$method(self, *other);
                }
            }
        };
    }
    modu32_ref_op_assign!(AddAssign, add_assign, Modu32<M>, Add, add);
    modu32_ref_op_assign!(SubAssign, sub_assign, Modu32<M>, Sub, sub);
    modu32_ref_op_assign!(MulAssign, mul_assign, Modu32<M>, Mul, mul);
    modu32_ref_op_assign!(DivAssign, div_assign, Modu32<M>, Div, div);
}

#[test]
fn test_modu32() {
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::default();
    const Q: usize = 10_000;
    type M = Modu32;
    for _ in 0..Q {
        let a = M::new(rand.rand(M::get_mod() as u64 - 1) as u32 + 1);
        let x = a.inv();
        assert!(x.x < M::get_mod());
        assert_eq!(a * x, M::one());
        assert_eq!(x, a.pow(M::get_mod() as usize - 2));
    }
}

use crate::algebra::operations::{AdditiveIdentity, MultiplicativeIdentity};
impl_additive_identity!([M: Modulo + PartialEq], Modu32<M>, Self::zero());
impl_multiplicative_identity!([M: Modulo + PartialEq], Modu32<M>, Self::one());

use crate::tools::scanner::IterScan;
impl<M: Modulo> IterScan for Modu32<M> {
    #[inline]
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self> {
        iter.next()?.parse::<Modu32<M>>().ok()
    }
}
