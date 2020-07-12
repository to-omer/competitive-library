//! modint

#[cargo_snippet::snippet("MInt")]
pub trait Modulus: Copy {
    fn get_modulus() -> u32;
    #[inline]
    fn modulo(x: u32) -> u32 {
        x % Self::get_modulus()
    }
}
#[cargo_snippet::snippet("MInt")]
#[allow(unused_macros)]
macro_rules! make_modulus {
    ($t:ident, $e:expr) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $t {}
        impl Modulus for $t {
            #[inline]
            fn get_modulus() -> u32 {
                const MODULUS: u32 = $e;
                MODULUS
            }
        }
    };
}
#[cargo_snippet::snippet("MInt")]
#[allow(unused_macros)]
macro_rules! make_dynamic_modulus {
    ($t:ident, $m:ident, $e:expr) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $t {}
        static mut $m: u32 = $e;
        impl Modulus for $t {
            #[inline]
            fn get_modulus() -> u32 {
                unsafe { $m }
            }
        }
    };
}
#[cargo_snippet::snippet("MInt")]
pub mod modulus {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Modulo1000000007 {}
    impl Modulus for Modulo1000000007 {
        #[inline]
        fn get_modulus() -> u32 {
            const MODULUS: u32 = 1_000_000_007;
            MODULUS
        }
    }
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Modulo1000000009 {}
    impl Modulus for Modulo1000000009 {
        #[inline]
        fn get_modulus() -> u32 {
            const MODULUS: u32 = 1_000_000_009;
            MODULUS
        }
    }
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Modulo998244353 {}
    impl Modulus for Modulo998244353 {
        #[inline]
        fn get_modulus() -> u32 {
            const MODULUS: u32 = 998_244_353;
            MODULUS
        }
    }
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct DynModulo {}
    static mut DYN_MODULUS: u32 = 1_000_000_007;
    impl Modulus for DynModulo {
        #[inline]
        fn get_modulus() -> u32 {
            unsafe { DYN_MODULUS }
        }
    }
    pub fn set_dyn_modulus(m: u32) {
        unsafe {
            DYN_MODULUS = m;
        }
    }
}
#[cargo_snippet::snippet("MInt")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MInt<M>
where
    M: Modulus,
{
    x: u32,
    phantom: std::marker::PhantomData<fn() -> M>,
}
#[cargo_snippet::snippet("MInt")]
impl<M: Modulus> MInt<M> {
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
        M::get_modulus()
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
        let (mut b, mut u, mut s) = (M::get_modulus(), 1, 0);
        let k = a.trailing_zeros();
        a >>= k;
        for _ in 0..k {
            if u & 1 == 1 {
                u += M::get_modulus();
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
                s += M::get_modulus();
            }
            s -= u;
            let k = b.trailing_zeros();
            b >>= k;
            for _ in 0..k {
                if s & 1 == 1 {
                    s += M::get_modulus();
                }
                s /= 2;
            }
        }
        Self::new_unchecked(s)
    }
}
#[cargo_snippet::snippet("MInt")]
pub mod modu32_impls {
    use super::*;
    use std::{
        fmt,
        iter::{Product, Sum},
        num::ParseIntError,
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
        str::FromStr,
    };
    impl<M: Modulus> From<u32> for MInt<M> {
        #[inline]
        fn from(x: u32) -> Self {
            Self::new(x)
        }
    }
    impl<M: Modulus> From<u64> for MInt<M> {
        #[inline]
        fn from(x: u64) -> Self {
            Self::new_unchecked((x % M::get_modulus() as u64) as u32)
        }
    }
    impl<M: Modulus> From<i32> for MInt<M> {
        #[inline]
        fn from(x: i32) -> Self {
            let x = x % M::get_modulus() as i32;
            if x < 0 {
                Self::new_unchecked((x + M::get_modulus() as i32) as u32)
            } else {
                Self::new_unchecked(x as u32)
            }
        }
    }
    impl<M: Modulus> From<i64> for MInt<M> {
        #[inline]
        fn from(x: i64) -> Self {
            let x = x % M::get_modulus() as i64;
            if x < 0 {
                Self::new_unchecked((x + M::get_modulus() as i64) as u32)
            } else {
                Self::new_unchecked(x as u32)
            }
        }
    }
    impl<M: Modulus> Add for MInt<M> {
        type Output = Self;
        #[inline]
        fn add(self, rhs: Self) -> Self::Output {
            let mut x = self.x + rhs.x;
            if x >= M::get_modulus() {
                x -= M::get_modulus();
            }
            Self::new_unchecked(x)
        }
    }
    impl<M: Modulus> Sub for MInt<M> {
        type Output = Self;
        #[inline]
        fn sub(self, rhs: Self) -> Self::Output {
            if self.x < rhs.x {
                Self::new_unchecked(self.x + M::get_modulus() - rhs.x)
            } else {
                Self::new_unchecked(self.x - rhs.x)
            }
        }
    }
    impl<M: Modulus> Mul for MInt<M> {
        type Output = Self;
        #[inline]
        fn mul(self, rhs: Self) -> Self::Output {
            Self::new_unchecked((self.x as u64 * rhs.x as u64 % M::get_modulus() as u64) as u32)
        }
    }
    impl<M: Modulus> Div for MInt<M> {
        type Output = Self;
        #[inline]
        fn div(self, rhs: Self) -> Self::Output {
            self * rhs.inv()
        }
    }
    impl<M: Modulus> Neg for MInt<M> {
        type Output = Self;
        #[inline]
        fn neg(self) -> Self::Output {
            if self.x == 0 {
                Self::zero()
            } else {
                Self::new_unchecked(M::get_modulus() - self.x)
            }
        }
    }
    impl<M: Modulus> Sum for MInt<M> {
        #[inline]
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Self::zero(), Add::add)
        }
    }
    impl<M: Modulus> Product for MInt<M> {
        #[inline]
        fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Self::one(), Mul::mul)
        }
    }
    impl<'a, M: Modulus + 'a> Sum<&'a MInt<M>> for MInt<M> {
        #[inline]
        fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
            iter.fold(Self::zero(), Add::add)
        }
    }
    impl<'a, M: Modulus + 'a> Product<&'a MInt<M>> for MInt<M> {
        #[inline]
        fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
            iter.fold(Self::one(), Mul::mul)
        }
    }
    impl<M: Modulus> fmt::Display for MInt<M> {
        fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
            write!(f, "{}", self.x)
        }
    }
    impl<M: Modulus> FromStr for MInt<M> {
        type Err = ParseIntError;
        #[inline]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u32>().map(Self::new)
        }
    }
    macro_rules! modu32_ref_binop {
        ($imp:ident, $method:ident, $t:ty) => {
            impl<M: Modulus> $imp<$t> for &$t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: $t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(*self, other)
                }
            }
            impl<M: Modulus> $imp<&$t> for $t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(self, *other)
                }
            }
            impl<M: Modulus> $imp<&$t> for &$t {
                type Output = <$t as $imp<$t>>::Output;
                #[inline]
                fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                    $imp::$method(*self, *other)
                }
            }
        };
    }
    modu32_ref_binop!(Add, add, MInt<M>);
    modu32_ref_binop!(Sub, sub, MInt<M>);
    modu32_ref_binop!(Mul, mul, MInt<M>);
    modu32_ref_binop!(Div, div, MInt<M>);
    macro_rules! modu32_ref_unop {
        ($imp:ident, $method:ident, $t:ty) => {
            impl<M: Modulus> $imp for &$t {
                type Output = <$t as $imp>::Output;
                #[inline]
                fn $method(self) -> <$t as $imp>::Output {
                    $imp::$method(*self)
                }
            }
        };
    }
    modu32_ref_unop!(Neg, neg, MInt<M>);
    macro_rules! modu32_ref_op_assign {
        ($imp:ident, $method:ident, $t:ty, $fromimp:ident, $frommethod:ident) => {
            impl<M: Modulus> $imp<$t> for $t {
                #[inline]
                fn $method(&mut self, rhs: $t) {
                    *self = $fromimp::$frommethod(*self, rhs);
                }
            }
            impl<M: Modulus> $imp<&$t> for $t {
                #[inline]
                fn $method(&mut self, other: &$t) {
                    $imp::$method(self, *other);
                }
            }
        };
    }
    modu32_ref_op_assign!(AddAssign, add_assign, MInt<M>, Add, add);
    modu32_ref_op_assign!(SubAssign, sub_assign, MInt<M>, Sub, sub);
    modu32_ref_op_assign!(MulAssign, mul_assign, MInt<M>, Mul, mul);
    modu32_ref_op_assign!(DivAssign, div_assign, MInt<M>, Div, div);
}

#[test]
fn test_modu32() {
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::default();
    const Q: usize = 10_000;
    type M = MInt<modulus::Modulo1000000007>;
    for _ in 0..Q {
        let a = M::new(rand.rand(M::get_mod() as u64 - 1) as u32 + 1);
        let x = a.inv();
        assert!(x.x < M::get_mod());
        assert_eq!(a * x, M::one());
        assert_eq!(x, a.pow(M::get_mod() as usize - 2));
    }
}

use crate::algebra::operations::{AdditiveIdentity, MultiplicativeIdentity};
impl_additive_identity!([M: Modulus + PartialEq], MInt<M>, Self::zero());
impl_multiplicative_identity!([M: Modulus + PartialEq], MInt<M>, Self::one());

use crate::tools::scanner::IterScan;

#[cargo_snippet::snippet("MInt")]
impl<M: Modulus> IterScan for MInt<M> {
    type Output = Self;
    #[inline]
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        iter.next()?.parse::<MInt<M>>().ok()
    }
}
