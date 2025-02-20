use super::*;

use std::{
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    iter::{Product, Sum},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

#[repr(transparent)]
pub struct MInt<M>
where
    M: MIntBase,
{
    x: M::Inner,
    _marker: PhantomData<fn() -> M>,
}

pub trait MIntConvert<T = <Self as MIntBase>::Inner>: MIntBase {
    fn from(x: T) -> <Self as MIntBase>::Inner;
    fn into(x: <Self as MIntBase>::Inner) -> T;
    fn mod_into() -> T;
}

pub trait MIntBase {
    type Inner: Sized + Copy + Eq + Debug + Hash;
    fn get_mod() -> Self::Inner;
    fn mod_zero() -> Self::Inner;
    fn mod_one() -> Self::Inner;
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner;
    fn mod_neg(x: Self::Inner) -> Self::Inner;
    fn mod_inv(x: Self::Inner) -> Self::Inner;
    fn mod_pow(x: Self::Inner, y: usize) -> Self::Inner {
        let (mut x, mut y, mut z) = (x, y, Self::mod_one());
        while y > 0 {
            if y & 1 == 1 {
                z = Self::mod_mul(z, x);
            }
            x = Self::mod_mul(x, x);
            y >>= 1;
        }
        z
    }
    fn mod_inner(x: Self::Inner) -> Self::Inner {
        x
    }
}

impl<M> MInt<M>
where
    M: MIntConvert,
{
    #[inline]
    pub fn new(x: M::Inner) -> Self {
        Self::new_unchecked(<M as MIntConvert<M::Inner>>::from(x))
    }
}
impl<M> MInt<M>
where
    M: MIntBase,
{
    #[inline]
    pub const fn new_unchecked(x: M::Inner) -> Self {
        Self {
            x,
            _marker: PhantomData,
        }
    }
    #[inline]
    pub fn get_mod() -> M::Inner {
        M::get_mod()
    }
    #[inline]
    pub fn pow(self, y: usize) -> Self {
        Self::new_unchecked(M::mod_pow(self.x, y))
    }
    #[inline]
    pub fn inv(self) -> Self {
        Self::new_unchecked(M::mod_inv(self.x))
    }
    #[inline]
    pub fn inner(self) -> M::Inner {
        M::mod_inner(self.x)
    }
}

impl<M> Clone for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<M> Copy for MInt<M> where M: MIntBase {}
impl<M> Debug for MInt<M>
where
    M: MIntBase,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.inner(), f)
    }
}
impl<M> Default for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn default() -> Self {
        <Self as Zero>::zero()
    }
}
impl<M> PartialEq for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.x, &other.x)
    }
}
impl<M> Eq for MInt<M> where M: MIntBase {}
impl<M> Hash for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.x, state)
    }
}
macro_rules! impl_mint_from {
    ($($t:ty),*) => {
        $(impl<M> From<$t> for MInt<M>
        where
            M: MIntConvert<$t>,
        {
            #[inline]
            fn from(x: $t) -> Self {
                Self::new_unchecked(<M as MIntConvert<$t>>::from(x))
            }
        }
        impl<M> From<MInt<M>> for $t
        where
            M: MIntConvert<$t>,
        {
            #[inline]
            fn from(x: MInt<M>) -> $t {
                <M as MIntConvert<$t>>::into(x.x)
            }
        })*
    };
}
impl_mint_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
impl<M> Zero for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn zero() -> Self {
        Self::new_unchecked(M::mod_zero())
    }
}
impl<M> One for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn one() -> Self {
        Self::new_unchecked(M::mod_one())
    }
}

impl<M> Add for MInt<M>
where
    M: MIntBase,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(M::mod_add(self.x, rhs.x))
    }
}
impl<M> Sub for MInt<M>
where
    M: MIntBase,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(M::mod_sub(self.x, rhs.x))
    }
}
impl<M> Mul for MInt<M>
where
    M: MIntBase,
{
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(M::mod_mul(self.x, rhs.x))
    }
}
impl<M> Div for MInt<M>
where
    M: MIntBase,
{
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(M::mod_div(self.x, rhs.x))
    }
}
impl<M> Neg for MInt<M>
where
    M: MIntBase,
{
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self::new_unchecked(M::mod_neg(self.x))
    }
}
impl<M> Sum for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(<Self as Zero>::zero(), Add::add)
    }
}
impl<M> Product for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(<Self as One>::one(), Mul::mul)
    }
}
impl<'a, M: 'a> Sum<&'a MInt<M>> for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(<Self as Zero>::zero(), Add::add)
    }
}
impl<'a, M: 'a> Product<&'a MInt<M>> for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(<Self as One>::one(), Mul::mul)
    }
}
impl<M> Display for MInt<M>
where
    M: MIntBase,
    M::Inner: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.inner())
    }
}
impl<M> FromStr for MInt<M>
where
    M: MIntConvert,
    M::Inner: FromStr,
{
    type Err = <M::Inner as FromStr>::Err;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<M::Inner>().map(Self::new)
    }
}
impl<M> IterScan for MInt<M>
where
    M: MIntConvert,
    M::Inner: FromStr,
{
    type Output = Self;
    #[inline]
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        iter.next()?.parse::<MInt<M>>().ok()
    }
}
macro_rules! impl_mint_ref_binop {
    ($imp:ident, $method:ident, $t:ty) => {
        impl<M> $imp<$t> for &$t
        where
            M: MIntBase,
        {
            type Output = <$t as $imp<$t>>::Output;
            #[inline]
            fn $method(self, other: $t) -> <$t as $imp<$t>>::Output {
                $imp::$method(*self, other)
            }
        }
        impl<M> $imp<&$t> for $t
        where
            M: MIntBase,
        {
            type Output = <$t as $imp<$t>>::Output;
            #[inline]
            fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                $imp::$method(self, *other)
            }
        }
        impl<M> $imp<&$t> for &$t
        where
            M: MIntBase,
        {
            type Output = <$t as $imp<$t>>::Output;
            #[inline]
            fn $method(self, other: &$t) -> <$t as $imp<$t>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}
impl_mint_ref_binop!(Add, add, MInt<M>);
impl_mint_ref_binop!(Sub, sub, MInt<M>);
impl_mint_ref_binop!(Mul, mul, MInt<M>);
impl_mint_ref_binop!(Div, div, MInt<M>);
macro_rules! impl_mint_ref_unop {
    ($imp:ident, $method:ident, $t:ty) => {
        impl<M> $imp for &$t
        where
            M: MIntBase,
        {
            type Output = <$t as $imp>::Output;
            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    };
}
impl_mint_ref_unop!(Neg, neg, MInt<M>);
macro_rules! impl_mint_ref_op_assign {
    ($imp:ident, $method:ident, $t:ty, $fromimp:ident, $frommethod:ident) => {
        impl<M> $imp<$t> for $t
        where
            M: MIntBase,
        {
            #[inline]
            fn $method(&mut self, rhs: $t) {
                *self = $fromimp::$frommethod(*self, rhs);
            }
        }
        impl<M> $imp<&$t> for $t
        where
            M: MIntBase,
        {
            #[inline]
            fn $method(&mut self, other: &$t) {
                $imp::$method(self, *other);
            }
        }
    };
}
impl_mint_ref_op_assign!(AddAssign, add_assign, MInt<M>, Add, add);
impl_mint_ref_op_assign!(SubAssign, sub_assign, MInt<M>, Sub, sub);
impl_mint_ref_op_assign!(MulAssign, mul_assign, MInt<M>, Mul, mul);
impl_mint_ref_op_assign!(DivAssign, div_assign, MInt<M>, Div, div);
