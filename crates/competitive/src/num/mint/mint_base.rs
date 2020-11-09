use super::*;

use std::{
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    iter::{Product, Sum},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

#[macro_export]
macro_rules! define_basic_mintbase {
    ($name:ident, $m:expr, $basety:ty, $upperty:ty, [$($unsigned:ty),*], [$($signed:ty),*]) => {
        pub struct $name;
        impl MIntBase for $name {
            type Inner = $basety;
            #[inline]
            fn get_mod() -> Self::Inner {
                $m
            }
            #[inline]
            fn mod_zero() -> Self::Inner {
                0
            }
            #[inline]
            fn mod_one() -> Self::Inner {
                1
            }
            #[inline]
            fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                let z = x + y;
                let m = Self::get_mod();
                if z >= m {
                    z - m
                } else {
                    z
                }
            }
            #[inline]
            fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                if x < y {
                    x + Self::get_mod() - y
                } else {
                    x - y
                }
            }
            #[inline]
            fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                (x as $upperty * y as $upperty % Self::get_mod() as $upperty) as $basety
            }
            #[inline]
            fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                Self::mod_mul(x, Self::mod_inv(y))
            }
            #[inline]
            fn mod_neg(x: Self::Inner) -> Self::Inner {
                if x == 0 {
                    0
                } else {
                    Self::get_mod() - x
                }
            }
            fn mod_inv(x: Self::Inner) -> Self::Inner {
                let mut a = x;
                let (mut b, mut u, mut s) = (Self::get_mod(), 1, 0);
                let k = a.trailing_zeros();
                a >>= k;
                for _ in 0..k {
                    if u & 1 == 1 {
                        u += Self::get_mod();
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
                        s += Self::get_mod();
                    }
                    s -= u;
                    let k = b.trailing_zeros();
                    b >>= k;
                    for _ in 0..k {
                        if s & 1 == 1 {
                            s += Self::get_mod();
                        }
                        s /= 2;
                    }
                }
                s
            }
        }
        $(impl MIntConvert<$unsigned> for $name {
            #[inline]
            fn from(x: $unsigned) -> Self::Inner {
                (x % <Self as MIntBase>::get_mod() as $unsigned) as $basety
            }
            #[inline]
            fn into(x: Self::Inner) -> $unsigned {
                x as $unsigned
            }
            #[inline]
            fn mod_into() -> $unsigned {
                <Self as MIntBase>::get_mod() as $unsigned
            }
        })*
        $(impl MIntConvert<$signed> for $name {
            #[inline]
            fn from(x: $signed) -> Self::Inner {
                let x = x % <Self as MIntBase>::get_mod() as $signed;
                if x < 0 {
                    (x + <Self as MIntBase>::get_mod() as $signed) as $basety
                } else {
                    x as $basety
                }
            }
            #[inline]
            fn into(x: Self::Inner) -> $signed {
                x as $signed
            }
            #[inline]
            fn mod_into() -> $signed {
                <Self as MIntBase>::get_mod() as $signed
            }
        })*
    };
}
#[macro_export]
macro_rules! define_basic_mint32 {
    ($([$name:ident, $m:expr, $mint_name:ident]),*) => {
        $(crate::define_basic_mintbase!(
            $name,
            $m,
            u32,
            u64,
            [u32, u64, u128, usize],
            [i32, i64, i128, isize]
        );
        pub type $mint_name = MInt<$name>;)*
    };
}
define_basic_mint32!(
    [Modulo998244353, 998_244_353, MInt998244353],
    [Modulo1000000007, 1_000_000_007, MInt1000000007],
    [Modulo1000000009, 1_000_000_009, MInt1000000009],
    [DynModuloU32, unsafe { DYN_MODULUS_U32 }, DynMIntU32]
);

static mut DYN_MODULUS_U32: u32 = 1_000_000_007;
impl DynModuloU32 {
    pub fn set_mod(m: u32) {
        unsafe {
            DYN_MODULUS_U32 = m;
        }
    }
}
static mut DYN_MODULUS_U64: u64 = 1_000_000_007;
define_basic_mintbase!(
    DynModuloU64,
    unsafe { DYN_MODULUS_U64 },
    u64,
    u128,
    [u64, u128, usize],
    [i64, i128, isize]
);
impl DynModuloU64 {
    pub fn set_mod(m: u64) {
        unsafe {
            DYN_MODULUS_U64 = m;
        }
    }
}
pub type DynMIntU64 = MInt<DynModuloU64>;

pub struct Modulo2;
impl MIntBase for Modulo2 {
    type Inner = u32;
    #[inline]
    fn get_mod() -> Self::Inner {
        2
    }
    #[inline]
    fn mod_zero() -> Self::Inner {
        0
    }
    #[inline]
    fn mod_one() -> Self::Inner {
        1
    }
    #[inline]
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        x ^ y
    }
    #[inline]
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        x ^ y
    }
    #[inline]
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        x | y
    }
    #[inline]
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        assert_ne!(y, 0);
        x
    }
    #[inline]
    fn mod_neg(x: Self::Inner) -> Self::Inner {
        x
    }
    #[inline]
    fn mod_inv(x: Self::Inner) -> Self::Inner {
        assert_ne!(x, 0);
        x
    }
    #[inline]
    fn mod_pow(x: Self::Inner, y: usize) -> Self::Inner {
        if y == 0 {
            1
        } else {
            x
        }
    }
}
macro_rules! impl_to_mint_base_for_modulo2 {
    ($name:ident, $basety:ty, [$($t:ty),*]) => {
        $(impl MIntConvert<$t> for $name {
            #[inline]
            fn from(x: $t) -> Self::Inner {
                (x & 1) as $basety
            }
            #[inline]
            fn into(x: Self::Inner) -> $t {
                x as $t
            }
            #[inline]
            fn mod_into() -> $t {
                1
            }
        })*
    };
}
impl_to_mint_base_for_modulo2!(
    Modulo2,
    u32,
    [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
);
pub type MInt2 = MInt<mint_base::Modulo2>;

impl<M> MInt<M>
where
    M: MIntConvert,
{
    #[inline]
    pub fn new(x: M::Inner) -> Self {
        Self::new_unchecked(<M as MIntConvert<M::Inner>>::from(x))
    }
    #[inline]
    pub fn inner(self) -> M::Inner {
        <M as MIntConvert<M::Inner>>::into(self.x)
    }
}
impl<M> MInt<M>
where
    M: MIntBase,
{
    #[inline]
    pub fn new_unchecked(x: M::Inner) -> Self {
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
}

impl<M> Clone for MInt<M>
where
    M: MIntBase,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            x: Clone::clone(&self.x),
            _marker: PhantomData,
        }
    }
}
impl<M> Copy for MInt<M> where M: MIntBase {}
impl<M> Debug for MInt<M>
where
    M: MIntBase,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.x, f)
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
impl_mint_from!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
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
    M: MIntConvert,
    M::Inner: Display,
{
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
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
