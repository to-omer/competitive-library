use super::{Bounded, IterScan, One, Zero};
use std::{
    convert::TryFrom,
    fmt::{self, Display},
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
    str::FromStr,
};

// primitive integer = arithmetic operations + binary represented operation
// arithmetic operations = integer basic operations + (unsigned operations | signed operations)

/// Trait for basic primitive integer operations.
pub trait IntBase:
    Copy
    + Bounded
    + Zero
    + One
    + Eq
    + Ord
    + Default
    + FromStr
    + Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + RemAssign
    + Sum
    + Product
{
    type Error;
    fn div_euclid(self, rhs: Self) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;
    fn pow(self, exp: u32) -> Self;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error>;
    fn ilog(self, base: Self) -> u32;
    fn ilog2(self) -> u32;
    fn ilog10(self) -> u32;
}
macro_rules! impl_int_base {
    ($($t:ty)*) => {
        $(
            impl IntBase for $t {
                type Error = std::num::ParseIntError;
                fn div_euclid(self, rhs: Self) -> Self { self.div_euclid(rhs) }
                fn rem_euclid(self, rhs: Self) -> Self { self.rem_euclid(rhs) }
                fn pow(self, exp: u32) -> Self { self.pow(exp) }
                fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error> { Self::from_str_radix(src, radix) }
                fn ilog(self, base: Self) -> u32 { self.ilog(base) }
                fn ilog2(self) -> u32 { self.ilog2() }
                fn ilog10(self) -> u32 { self.ilog10() }
            }
        )*
    };
}
impl_int_base!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

/// extended_gcd(a,b): ax + by = g = gcd(a,b)
pub struct ExtendedGcd<T: Signed> {
    /// gcd
    pub g: T::Unsigned,
    pub x: T,
    pub y: T,
}

/// Trait for unsigned integer operations.
pub trait Unsigned: IntBase {
    type Signed: Signed<Unsigned = Self>;
    fn signed(self) -> Self::Signed;
    fn abs_diff(self, other: Self) -> Self;
    fn next_power_of_two(self) -> Self;
    fn gcd(self, other: Self) -> Self;
    fn lcm(self, other: Self) -> Self {
        if self.is_zero() && other.is_zero() {
            Self::zero()
        } else {
            self / self.gcd(other) * other
        }
    }
    fn modinv(self, modulo: Self) -> Self {
        assert!(
            !self.is_zero(),
            "attempt to inverse zero with modulo {}",
            modulo
        );
        let extgcd = self.signed().extgcd(modulo.signed());
        assert!(
            extgcd.g.is_one(),
            "there is no inverse {} modulo {}",
            self,
            modulo
        );
        extgcd.x.rem_euclid(modulo.signed()).unsigned()
    }
}
/// Trait for signed integer operations.
pub trait Signed: IntBase + Neg<Output = Self> {
    type Unsigned: Unsigned<Signed = Self>;
    fn unsigned(self) -> Self::Unsigned;
    fn abs(self) -> Self;
    fn abs_diff(self, other: Self) -> Self::Unsigned;
    fn is_negative(self) -> bool;
    fn is_positive(self) -> bool;
    fn signum(self) -> Self;
    fn extgcd(self, other: Self) -> ExtendedGcd<Self> {
        let (mut a, mut b) = (self, other);
        let (mut u, mut v, mut x, mut y) = (Self::one(), Self::zero(), Self::zero(), Self::one());
        while !a.is_zero() {
            let k = b / a;
            x -= k * u;
            y -= k * v;
            b -= k * a;
            std::mem::swap(&mut x, &mut u);
            std::mem::swap(&mut y, &mut v);
            std::mem::swap(&mut b, &mut a);
        }
        if b.is_negative() {
            b = -b;
            x = -x;
            y = -y;
        }
        ExtendedGcd {
            g: b.unsigned(),
            x,
            y,
        }
    }
}

macro_rules! impl_unsigned_signed {
    ($($unsigned:ident $signed:ident)*) => {
        $(
            impl Unsigned for $unsigned {
                type Signed = $signed;
                fn signed(self) -> Self::Signed { self as Self::Signed }
                fn abs_diff(self, other: Self) -> Self { self.abs_diff(other) }
                fn next_power_of_two(self) -> Self { self.next_power_of_two() }
                fn gcd(self, other: Self) -> Self {
                    let (mut a, mut b) = (self, other);
                    if a.is_zero() || b.is_zero() {
                        return a | b;
                    }
                    let u = a.trailing_zeros();
                    let v = b.trailing_zeros();
                    a >>= u;
                    b >>= v;
                    let k = u.min(v);
                    while a != b {
                        if a < b {
                            std::mem::swap(&mut a, &mut b);
                        }
                        a -= b;
                        a >>= a.trailing_zeros();
                    }
                    a << k
                }
            }
            impl Signed for $signed {
                type Unsigned = $unsigned;
                fn unsigned(self) -> Self::Unsigned { self as Self::Unsigned }
                fn abs_diff(self, other: Self) -> Self::Unsigned { self.abs_diff(other) }
                fn abs(self) -> Self { self.abs() }
                fn is_negative(self) -> bool { self.is_negative() }
                fn is_positive(self) -> bool { self.is_positive() }
                fn signum(self) -> Self { self.signum() }
            }
        )*
    };
}
impl_unsigned_signed!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

/// Trait for operations of integer in binary representation.
pub trait BinaryRepr<Size = u32>:
    Sized
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<Size, Output = Self>
    + Shr<Size, Output = Self>
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
    + ShlAssign<Size>
    + ShrAssign<Size>
{
    fn count_ones(self) -> Size;
    fn count_zeros(self) -> Size;
    fn leading_ones(self) -> Size;
    fn leading_zeros(self) -> Size;
    fn reverse_bits(self) -> Self;
    fn rotate_left(self, n: Size) -> Self;
    fn rotate_right(self, n: Size) -> Self;
    fn swap_bytes(self) -> Self;
    fn trailing_ones(self) -> Size;
    fn trailing_zeros(self) -> Size;
}

macro_rules! impl_binary_repr {
    ($($t:ty)*) => {
        $(
            impl BinaryRepr for $t {
                fn count_ones(self) -> u32 { self.count_ones() }
                fn count_zeros(self) -> u32 { self.count_zeros() }
                fn leading_ones(self) -> u32 { self.leading_ones() }
                fn leading_zeros(self) -> u32 { self.leading_zeros() }
                fn reverse_bits(self) -> Self { self.reverse_bits() }
                fn rotate_left(self, n: u32) -> Self { self.rotate_left(n) }
                fn rotate_right(self, n: u32) -> Self { self.rotate_right(n) }
                fn swap_bytes(self) -> Self { self.swap_bytes() }
                fn trailing_ones(self) -> u32 { self.trailing_ones() }
                fn trailing_zeros(self) -> u32 { self.trailing_zeros() }
            }
        )*
    };
}
impl_binary_repr!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

macro_rules! impl_binop {
    (impl<$T:ident> $Trait:ident $impl:ident for $t:ty) => {
        impl<$T> $Trait for $t
        where
            $T: $Trait<Output = $T>,
        {
            type Output = Self;
            fn $impl(self, rhs: Self) -> Self::Output {
                Self($Trait::$impl(self.0, rhs.0))
            }
        }
        impl<$T> $Trait<$T> for $t
        where
            $T: $Trait<Output = $T>,
        {
            type Output = Self;
            fn $impl(self, rhs: $T) -> Self::Output {
                Self($Trait::$impl(self.0, rhs))
            }
        }
    };
}
macro_rules! impl_opassign {
    (impl<$T:ident> $Trait:ident $impl:ident for $t:ty) => {
        impl<$T> $Trait for $t
        where
            $T: $Trait,
        {
            fn $impl(&mut self, rhs: Self) {
                $Trait::$impl(&mut self.0, rhs.0)
            }
        }
        impl<$T> $Trait<$T> for $t
        where
            $T: $Trait,
        {
            fn $impl(&mut self, rhs: $T) {
                $Trait::$impl(&mut self.0, rhs)
            }
        }
    };
    (impl<$T:ident> $Trait:ident $impl:ident for $t:ty => $F:ident $f:ident) => {
        impl<$T> $Trait for $t
        where
            $t: $F<Output = $t> + Copy,
        {
            fn $impl(&mut self, rhs: Self) {
                *self = $F::$f(*self, rhs);
            }
        }
        impl<$T> $Trait<$T> for $t
        where
            $t: $F<$T, Output = $t> + Copy,
        {
            fn $impl(&mut self, rhs: $T) {
                *self = $F::$f(*self, rhs);
            }
        }
    };
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
/// Wrapper type of arithmetic `saturating_*` operations.
pub struct Saturating<T>(pub T);
pub trait Saturatingable: Sized
where
    Saturating<Self>: Copy
        + Bounded
        + Zero
        + One
        + Eq
        + Ord
        + Default
        + FromStr
        + Display
        + Add<Output = Saturating<Self>>
        + Sub<Output = Saturating<Self>>
        + Mul<Output = Saturating<Self>>
        + Div<Output = Saturating<Self>>
        + Rem<Output = Saturating<Self>>
        + BitAnd<Output = Saturating<Self>>
        + BitOr<Output = Saturating<Self>>
        + BitXor<Output = Saturating<Self>>
        + Shl<u32, Output = Saturating<Self>>
        + Shr<u32, Output = Saturating<Self>>
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + RemAssign
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShlAssign<u32>
        + ShrAssign<u32>
        + Not<Output = Saturating<Self>>
        + Add<Self, Output = Saturating<Self>>
        + Sub<Self, Output = Saturating<Self>>
        + Mul<Self, Output = Saturating<Self>>
        + Div<Self, Output = Saturating<Self>>
        + Rem<Self, Output = Saturating<Self>>
        + BitAnd<Self, Output = Saturating<Self>>
        + BitOr<Self, Output = Saturating<Self>>
        + BitXor<Self, Output = Saturating<Self>>
        + AddAssign<Self>
        + SubAssign<Self>
        + MulAssign<Self>
        + DivAssign<Self>
        + RemAssign<Self>
        + BitAndAssign<Self>
        + BitOrAssign<Self>
        + BitXorAssign<Self>,
{
    fn to_saturating(self) -> Saturating<Self> {
        Saturating(self)
    }
    fn from_saturating(s: Saturating<Self>) -> Self {
        s.0
    }
}

impl<T> fmt::Debug for Saturating<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self.0, f)
    }
}
impl<T> Bounded for Saturating<T>
where
    T: Bounded,
{
    fn maximum() -> Self {
        Self(T::maximum())
    }
    fn minimum() -> Self {
        Self(T::minimum())
    }
}
impl<T> Zero for Saturating<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self(T::zero())
    }
}
impl<T> One for Saturating<T>
where
    T: One,
{
    fn one() -> Self {
        Self(T::one())
    }
}
impl<T> FromStr for Saturating<T>
where
    T: FromStr,
{
    type Err = T::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Self)
    }
}
impl<T> Display for Saturating<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self.0, f)
    }
}
impl<T> IterScan for Saturating<T>
where
    T: IterScan<Output = T>,
{
    type Output = Self;
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        T::scan(iter).map(Self)
    }
}
impl_binop!(impl<T> Div div for Saturating<T>);
impl_binop!(impl<T> Rem rem for Saturating<T>);
impl_binop!(impl<T> BitAnd bitand for Saturating<T>);
impl_binop!(impl<T> BitOr bitor for Saturating<T>);
impl_binop!(impl<T> BitXor bitxor for Saturating<T>);
impl_opassign!(impl<T> AddAssign add_assign for Saturating<T> => Add add);
impl_opassign!(impl<T> SubAssign sub_assign for Saturating<T> => Sub sub);
impl_opassign!(impl<T> MulAssign mul_assign for Saturating<T> => Mul mul);
impl_opassign!(impl<T> DivAssign div_assign for Saturating<T>);
impl_opassign!(impl<T> RemAssign rem_assign for Saturating<T>);
impl_opassign!(impl<T> BitAndAssign bitand_assign for Saturating<T>);
impl_opassign!(impl<T> BitOrAssign bitor_assign for Saturating<T>);
impl_opassign!(impl<T> BitXorAssign bitxor_assign for Saturating<T>);
impl<T> Not for Saturating<T>
where
    T: Not<Output = T>,
{
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(Not::not(self.0))
    }
}

macro_rules! impl_int_base_for_saturating {
    ($($t:ty)*) => {
        $(
            impl Saturatingable for $t {}
            impl Add for Saturating<$t> {
                type Output = Self;
                fn add(self, rhs: Self) -> Self::Output {
                    Self(self.0.saturating_add(rhs.0))
                }
            }
            impl Add<$t> for Saturating<$t> {
                type Output = Self;
                fn add(self, rhs: $t) -> Self::Output {
                    Self(self.0.saturating_add(rhs))
                }
            }
            impl Sub for Saturating<$t> {
                type Output = Self;
                fn sub(self, rhs: Self) -> Self::Output {
                    Self(self.0.saturating_sub(rhs.0))
                }
            }
            impl Sub<$t> for Saturating<$t> {
                type Output = Self;
                fn sub(self, rhs: $t) -> Self::Output {
                    Self(self.0.saturating_sub(rhs))
                }
            }
            impl Mul for Saturating<$t> {
                type Output = Self;
                fn mul(self, rhs: Self) -> Self::Output {
                    Self(self.0.saturating_mul(rhs.0))
                }
            }
            impl Mul<$t> for Saturating<$t> {
                type Output = Self;
                fn mul(self, rhs: $t) -> Self::Output {
                    Self(self.0.saturating_mul(rhs))
                }
            }
            impl Sum for Saturating<$t> {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.fold(Self::zero(), |acc, x| acc + x)
                }
            }
            impl Product for Saturating<$t> {
                fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.fold(Self::one(), |acc, x| acc * x)
                }
            }
            impl IntBase for Saturating<$t> {
                type Error = <$t as IntBase>::Error;
                fn div_euclid(self, rhs: Self) -> Self { Self(self.0.div_euclid(rhs.0)) }
                fn rem_euclid(self, rhs: Self) -> Self { Self(self.0.rem_euclid(rhs.0)) }
                fn pow(self, exp: u32) -> Self { Self(self.0.saturating_pow(exp)) }
                fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error> { <$t as IntBase>::from_str_radix(src, radix).map(Self) }
                fn ilog(self, base: Self) -> u32 { self.0.ilog(base.0) }
                fn ilog2(self) -> u32 { self.0.ilog2() }
                fn ilog10(self) -> u32 { self.0.ilog10() }
            }
            impl From<$t> for Saturating<$t> {
                fn from(t: $t) -> Self {
                    Self(t)
                }
            }
        )*
    };
}
impl_int_base_for_saturating!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

macro_rules! impl_unsigned_signed_for_saturating {
    ($($unsigned:ident $signed:ident)*) => {
        $(
            impl Unsigned for Saturating<$unsigned> {
                type Signed = Saturating<$signed>;
                fn signed(self) -> Self::Signed { Saturating(TryFrom::try_from(self.0).ok().unwrap_or_else($signed::maximum)) }
                fn abs_diff(self, other: Self) -> Self { Self(self.0.abs_diff(other.0)) }
                fn next_power_of_two(self) -> Self { Self(self.0.next_power_of_two()) }
                fn gcd(self, other: Self) -> Self { Self(self.0.gcd(other.0)) }
            }
            impl Signed for Saturating<$signed> {
                type Unsigned = Saturating<$unsigned>;
                fn unsigned(self) -> Self::Unsigned { Saturating(TryFrom::try_from(self.0).ok().unwrap_or_else($unsigned::minimum)) }
                fn abs(self) -> Self { Self(self.0.saturating_abs()) }
                fn abs_diff(self, other: Self) -> Self::Unsigned { Saturating(self.0.abs_diff(other.0)) }
                fn is_negative(self) -> bool { self.0.is_negative() }
                fn is_positive(self) -> bool { self.0.is_positive() }
                fn signum(self) -> Self { Self(self.0.signum()) }
            }
            impl Neg for Saturating<$signed> {
                type Output = Self;
                fn neg(self) -> Self::Output {
                    Self(self.0.saturating_neg())
                }
            }
        )*
    };
}
impl_unsigned_signed_for_saturating!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

macro_rules! impl_binary_repr_for_saturating {
    ($($t:ty)*) => {
        $(
            impl Shl<u32> for Saturating<$t> {
                type Output = Self;
                fn shl(self, rhs: u32) -> Self::Output {
                    Self(self.0.checked_shl(rhs).unwrap_or(0))
                }
            }
            impl Shr<u32> for Saturating<$t> {
                type Output = Self;
                fn shr(self, rhs: u32) -> Self::Output {
                    Self(self.0.checked_shr(rhs).unwrap_or(0))
                }
            }
            impl ShlAssign<u32> for Saturating<$t> {
                fn shl_assign(&mut self, rhs: u32) {
                    *self = Shl::shl(*self, rhs);
                }
            }
            impl ShrAssign<u32> for Saturating<$t> {
                fn shr_assign(&mut self, rhs: u32) {
                    *self = Shr::shr(*self, rhs);
                }
            }
            impl BinaryRepr for Saturating<$t> {
                fn count_ones(self) -> u32 { self.0.count_ones() }
                fn count_zeros(self) -> u32 { self.0.count_zeros() }
                fn leading_ones(self) -> u32 { self.0.leading_ones() }
                fn leading_zeros(self) -> u32 { self.0.leading_zeros() }
                fn reverse_bits(self) -> Self { Self(self.0.reverse_bits()) }
                fn rotate_left(self, n: u32) -> Self { Self(self.0.rotate_left(n)) }
                fn rotate_right(self, n: u32) -> Self { Self(self.0.rotate_right(n)) }
                fn swap_bytes(self) -> Self { Self(self.0.swap_bytes()) }
                fn trailing_ones(self) -> u32 { self.0.trailing_ones() }
                fn trailing_zeros(self) -> u32 { self.0.trailing_zeros() }
            }
        )*
    };
}
impl_binary_repr_for_saturating!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
/// Wrapper type of arithmetic `wrapping_*` operations.
pub struct Wrapping<T>(pub T);
pub trait Wrappingable: Sized
where
    Wrapping<Self>: Copy
        + Bounded
        + Zero
        + One
        + Eq
        + Ord
        + Default
        + FromStr
        + Display
        + Add<Output = Wrapping<Self>>
        + Sub<Output = Wrapping<Self>>
        + Mul<Output = Wrapping<Self>>
        + Div<Output = Wrapping<Self>>
        + Rem<Output = Wrapping<Self>>
        + BitAnd<Output = Wrapping<Self>>
        + BitOr<Output = Wrapping<Self>>
        + BitXor<Output = Wrapping<Self>>
        + Shl<u32, Output = Wrapping<Self>>
        + Shr<u32, Output = Wrapping<Self>>
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + RemAssign
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + ShlAssign<u32>
        + ShrAssign<u32>
        + Not<Output = Wrapping<Self>>
        + Add<Self, Output = Wrapping<Self>>
        + Sub<Self, Output = Wrapping<Self>>
        + Mul<Self, Output = Wrapping<Self>>
        + Div<Self, Output = Wrapping<Self>>
        + Rem<Self, Output = Wrapping<Self>>
        + BitAnd<Self, Output = Wrapping<Self>>
        + BitOr<Self, Output = Wrapping<Self>>
        + BitXor<Self, Output = Wrapping<Self>>
        + AddAssign<Self>
        + SubAssign<Self>
        + MulAssign<Self>
        + DivAssign<Self>
        + RemAssign<Self>
        + BitAndAssign<Self>
        + BitOrAssign<Self>
        + BitXorAssign<Self>,
{
    fn to_wrapping(self) -> Wrapping<Self> {
        Wrapping(self)
    }
    fn from_wrapping(w: Wrapping<Self>) -> Self {
        w.0
    }
}

impl<T> fmt::Debug for Wrapping<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self.0, f)
    }
}
impl<T> Bounded for Wrapping<T>
where
    T: Bounded,
{
    fn maximum() -> Self {
        Self(T::maximum())
    }
    fn minimum() -> Self {
        Self(T::minimum())
    }
}
impl<T> Zero for Wrapping<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self(T::zero())
    }
}
impl<T> One for Wrapping<T>
where
    T: One,
{
    fn one() -> Self {
        Self(T::one())
    }
}
impl<T> FromStr for Wrapping<T>
where
    T: FromStr,
{
    type Err = T::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Self)
    }
}
impl<T> Display for Wrapping<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self.0, f)
    }
}
impl<T> IterScan for Wrapping<T>
where
    T: IterScan<Output = T>,
{
    type Output = Self;
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        T::scan(iter).map(Self)
    }
}
impl_binop!(impl<T> BitAnd bitand for Wrapping<T>);
impl_binop!(impl<T> BitOr bitor for Wrapping<T>);
impl_binop!(impl<T> BitXor bitxor for Wrapping<T>);
impl_opassign!(impl<T> AddAssign add_assign for Wrapping<T> => Add add);
impl_opassign!(impl<T> SubAssign sub_assign for Wrapping<T> => Sub sub);
impl_opassign!(impl<T> MulAssign mul_assign for Wrapping<T> => Mul mul);
impl_opassign!(impl<T> DivAssign div_assign for Wrapping<T> => Div div);
impl_opassign!(impl<T> RemAssign rem_assign for Wrapping<T> => Rem rem);
impl_opassign!(impl<T> BitAndAssign bitand_assign for Wrapping<T>);
impl_opassign!(impl<T> BitOrAssign bitor_assign for Wrapping<T>);
impl_opassign!(impl<T> BitXorAssign bitxor_assign for Wrapping<T>);
impl<T> Not for Wrapping<T>
where
    T: Not<Output = T>,
{
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(Not::not(self.0))
    }
}

macro_rules! impl_int_base_for_wrapping {
    ($($t:ty)*) => {
        $(
            impl Wrappingable for $t {}
            impl Add for Wrapping<$t> {
                type Output = Self;
                fn add(self, rhs: Self) -> Self::Output {
                    Self(self.0.wrapping_add(rhs.0))
                }
            }
            impl Add<$t> for Wrapping<$t> {
                type Output = Self;
                fn add(self, rhs: $t) -> Self::Output {
                    Self(self.0.wrapping_add(rhs))
                }
            }
            impl Sub for Wrapping<$t> {
                type Output = Self;
                fn sub(self, rhs: Self) -> Self::Output {
                    Self(self.0.wrapping_sub(rhs.0))
                }
            }
            impl Sub<$t> for Wrapping<$t> {
                type Output = Self;
                fn sub(self, rhs: $t) -> Self::Output {
                    Self(self.0.wrapping_sub(rhs))
                }
            }
            impl Mul for Wrapping<$t> {
                type Output = Self;
                fn mul(self, rhs: Self) -> Self::Output {
                    Self(self.0.wrapping_mul(rhs.0))
                }
            }
            impl Mul<$t> for Wrapping<$t> {
                type Output = Self;
                fn mul(self, rhs: $t) -> Self::Output {
                    Self(self.0.wrapping_mul(rhs))
                }
            }
            impl Div for Wrapping<$t> {
                type Output = Self;
                fn div(self, rhs: Self) -> Self::Output {
                    Self(self.0.wrapping_div(rhs.0))
                }
            }
            impl Div<$t> for Wrapping<$t> {
                type Output = Self;
                fn div(self, rhs: $t) -> Self::Output {
                    Self(self.0.wrapping_div(rhs))
                }
            }
            impl Rem for Wrapping<$t> {
                type Output = Self;
                fn rem(self, rhs: Self) -> Self::Output {
                    Self(self.0.wrapping_rem(rhs.0))
                }
            }
            impl Rem<$t> for Wrapping<$t> {
                type Output = Self;
                fn rem(self, rhs: $t) -> Self::Output {
                    Self(self.0.wrapping_rem(rhs))
                }
            }
            impl Sum for Wrapping<$t> {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.fold(Self::zero(), |acc, x| acc + x)
                }
            }
            impl Product for Wrapping<$t> {
                fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.fold(Self::one(), |acc, x| acc * x)
                }
            }
            impl IntBase for Wrapping<$t> {
                type Error = <$t as IntBase>::Error;
                fn div_euclid(self, rhs: Self) -> Self { Self(self.0.wrapping_div_euclid(rhs.0)) }
                fn rem_euclid(self, rhs: Self) -> Self { Self(self.0.wrapping_rem_euclid(rhs.0)) }
                fn pow(self, exp: u32) -> Self { Self(self.0.wrapping_pow(exp)) }
                fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error> { <$t as IntBase>::from_str_radix(src, radix).map(Self) }
                fn ilog(self, base: Self) -> u32 { self.0.ilog(base.0) }
                fn ilog2(self) -> u32 { self.0.ilog2() }
                fn ilog10(self) -> u32 { self.0.ilog10() }
            }
            impl From<$t> for Wrapping<$t> {
                fn from(t: $t) -> Self {
                    Self(t)
                }
            }
        )*
    };
}
impl_int_base_for_wrapping!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

macro_rules! impl_unsigned_signed_for_wrapping {
    ($($unsigned:ident $signed:ident)*) => {
        $(
            impl Unsigned for Wrapping<$unsigned> {
                type Signed = Wrapping<$signed>;
                fn signed(self) -> Self::Signed { Wrapping(self.0.signed()) }
                fn abs_diff(self, other: Self) -> Self { Self(self.0.abs_diff(other.0)) }
                fn next_power_of_two(self) -> Self { Self(self.0.next_power_of_two()) }
                fn gcd(self, other: Self) -> Self { Self(self.0.gcd(other.0)) }
            }
            impl Signed for Wrapping<$signed> {
                type Unsigned = Wrapping<$unsigned>;
                fn unsigned(self) -> Self::Unsigned { Wrapping(self.0.unsigned()) }
                fn abs(self) -> Self { Self(self.0.wrapping_abs()) }
                fn abs_diff(self, other: Self) -> Self::Unsigned { Wrapping(self.0.abs_diff(other.0)) }
                fn is_negative(self) -> bool { self.0.is_negative() }
                fn is_positive(self) -> bool { self.0.is_positive() }
                fn signum(self) -> Self { Self(self.0.signum()) }
            }
            impl Neg for Wrapping<$signed> {
                type Output = Self;
                fn neg(self) -> Self::Output {
                    Self(self.0.wrapping_neg())
                }
            }
        )*
    };
}
impl_unsigned_signed_for_wrapping!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

macro_rules! impl_binary_repr_for_wrapping {
    ($($t:ty)*) => {
        $(
            impl Shl<u32> for Wrapping<$t> {
                type Output = Self;
                fn shl(self, rhs: u32) -> Self::Output {
                    Self(self.0.wrapping_shl(rhs))
                }
            }
            impl Shr<u32> for Wrapping<$t> {
                type Output = Self;
                fn shr(self, rhs: u32) -> Self::Output {
                    Self(self.0.wrapping_shr(rhs))
                }
            }
            impl ShlAssign<u32> for Wrapping<$t> {
                fn shl_assign(&mut self, rhs: u32) {
                    *self = Shl::shl(*self, rhs);
                }
            }
            impl ShrAssign<u32> for Wrapping<$t> {
                fn shr_assign(&mut self, rhs: u32) {
                    *self = Shr::shr(*self, rhs);
                }
            }
            impl BinaryRepr for Wrapping<$t> {
                fn count_ones(self) -> u32 { self.0.count_ones() }
                fn count_zeros(self) -> u32 { self.0.count_zeros() }
                fn leading_ones(self) -> u32 { self.0.leading_ones() }
                fn leading_zeros(self) -> u32 { self.0.leading_zeros() }
                fn reverse_bits(self) -> Self { Self(self.0.reverse_bits()) }
                fn rotate_left(self, n: u32) -> Self { Self(self.0.rotate_left(n)) }
                fn rotate_right(self, n: u32) -> Self { Self(self.0.rotate_right(n)) }
                fn swap_bytes(self) -> Self { Self(self.0.swap_bytes()) }
                fn trailing_ones(self) -> u32 { self.0.trailing_ones() }
                fn trailing_zeros(self) -> u32 { self.0.trailing_zeros() }
            }
        )*
    };
}
impl_binary_repr_for_wrapping!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    const Q: usize = 10_000;
    macro_rules! test_unsigned {
        ($($t:ident)*) => {
            $(
                mod $t {
                    use super::*;
                    const A: $t = $t::MAX / 2;
                    fn gcd(mut a: $t, mut b: $t) -> $t {
                        while b != 0 {
                            a %= b;
                            std::mem::swap(&mut a, &mut b);
                        }
                        a
                    }
                    #[test]
                    fn test_gcd() {
                        let mut rng = Xorshift::default();
                            for (a, b) in rng.random_iter((0..=A, 0..=A)).take(Q) {
                            assert_eq!(a.gcd(b), gcd(a, b));
                        }
                        assert_eq!($t::zero().gcd(0), 0);
                        assert_eq!($t::zero().gcd(100), 100);
                    }
                    #[test]
                    fn test_modinv() {
                        let mut rng = Xorshift::default();
                        for _ in 0..Q {
                            let m = rng.random(2..=A);
                            let a = rng.random(1..m);
                            let g = a.gcd(m);
                            let m = m / g;
                            let a = a / g;
                            let x = a.modinv(m);
                            assert!(x < m);
                            assert_eq!(a as u128 * x as u128 % m as u128, 1);
                        }
                    }
                }
            )*
        };
    }
    test_unsigned!(u8 u16 u32 u64 usize);

    macro_rules! test_signed {
        ($($t:ident)*) => {
            $(
                mod $t {
                    use super::*;
                    const A: $t = $t::MAX / 2;
                    #[test]
                    fn test_extgcd() {
                        let mut rng = Xorshift::default();
                        for (a, b) in rng.random_iter((-A..=A, -A..=A)).take(Q) {
                            let ExtendedGcd { g, x, y } = a.extgcd(b);
                            assert_eq!(g, a.abs().unsigned().gcd(b.abs().unsigned()));
                            assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g.signed() as i128);
                        }
                    }
                }
            )*
        };
    }
    test_signed!(i8 i16 i32 i64 isize);
}
