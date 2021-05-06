#[codesnip::skip]
use crate::num::{Bounded, One, Zero};

// primitive integer = arithmetic operations + binary represented operation
// arithmetic operations = integer basic operations + (unsigned operations | signed operations)

pub use integer_impls::{BinaryRepr, ExtendedGcd, IntBase, Saturating, Signed, Unsigned, Wrapping};
mod integer_impls {
    use super::*;
    use std::{
        convert::TryFrom,
        fmt::{self, Display},
        ops::{
            Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
            DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign,
            Sub, SubAssign,
        },
        str::FromStr,
    };

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
    {
        type Error;
        fn div_euclid(self, rhs: Self) -> Self;
        fn rem_euclid(self, rhs: Self) -> Self;
        fn pow(self, exp: u32) -> Self;
        fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error>;
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
                }
            )*
        };
    }
    impl_int_base!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

    /// extended_gcd(a,b): ax + by = g = gcd(a,b)
    pub struct ExtendedGcd<T: Unsigned> {
        pub g: T,
        pub x: T::Signed,
        pub y: T::Signed,
    }

    pub trait Unsigned: IntBase {
        type Signed: Signed<Unsigned = Self>;
        fn signed(self) -> Self::Signed;
        fn abs_sub(self, other: Self) -> Self {
            if self > other {
                self - other
            } else {
                other - self
            }
        }
        fn gcd(self, other: Self) -> Self;
        fn lcm(self, other: Self) -> Self {
            if self.is_zero() && other.is_zero() {
                Self::zero()
            } else {
                self / self.gcd(other) * other
            }
        }
        fn extgcd(self, other: Self) -> ExtendedGcd<Self> {
            let (mut a, mut b) = (self.signed(), other.signed());
            let (mut u, mut v, mut x, mut y) = (
                Self::Signed::one(),
                Self::Signed::zero(),
                Self::Signed::zero(),
                Self::Signed::one(),
            );
            while !a.is_zero() {
                let k = b / a;
                x -= k * u;
                y -= k * v;
                b -= k * a;
                std::mem::swap(&mut x, &mut u);
                std::mem::swap(&mut y, &mut v);
                std::mem::swap(&mut b, &mut a);
            }
            ExtendedGcd {
                g: b.unsigned(),
                x,
                y,
            }
        }
        fn modinv(self, modulo: Self) -> Self {
            let extgcd = self.extgcd(modulo);
            assert!(extgcd.g.is_one());
            extgcd.x.rem_euclid(modulo.signed()).unsigned()
        }
    }
    pub trait Signed: IntBase + Neg<Output = Self> {
        type Unsigned: Unsigned<Signed = Self>;
        fn unsigned(self) -> Self::Unsigned;
        fn abs(self) -> Self;
        fn is_negative(self) -> bool;
        fn is_positive(self) -> bool;
        fn signum(self) -> Self;
    }

    macro_rules! impl_unsigned_signed {
        ($($unsigned:ident $signed:ident)*) => {
            $(
                impl Unsigned for $unsigned {
                    type Signed = $signed;
                    fn signed(self) -> Self::Signed { self as Self::Signed }
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
                    fn abs(self) -> Self { self.abs() }
                    fn is_negative(self) -> bool { self.is_negative() }
                    fn is_positive(self) -> bool { self.is_positive() }
                    fn signum(self) -> Self { self.signum() }
                }
            )*
        };
    }
    impl_unsigned_signed!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

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
        };
    }

    #[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[repr(transparent)]
    pub struct Saturating<T>(pub T);

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
    impl_binop!(impl<T> Div div for Saturating<T>);
    impl_binop!(impl<T> Rem rem for Saturating<T>);
    impl_binop!(impl<T> BitAnd bitand for Saturating<T>);
    impl_binop!(impl<T> BitOr bitor for Saturating<T>);
    impl_binop!(impl<T> BitXor bitxor for Saturating<T>);
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
                impl Add for Saturating<$t> {
                    type Output = Self;
                    fn add(self, rhs: Self) -> Self::Output {
                        Self(self.0.saturating_add(rhs.0))
                    }
                }
                impl Sub for Saturating<$t> {
                    type Output = Self;
                    fn sub(self, rhs: Self) -> Self::Output {
                        Self(self.0.saturating_sub(rhs.0))
                    }
                }
                impl Mul for Saturating<$t> {
                    type Output = Self;
                    fn mul(self, rhs: Self) -> Self::Output {
                        Self(self.0.saturating_mul(rhs.0))
                    }
                }
                impl AddAssign for Saturating<$t> {
                    fn add_assign(&mut self, rhs: Self) {
                        *self = Add::add(*self, rhs);
                    }
                }
                impl SubAssign for Saturating<$t> {
                    fn sub_assign(&mut self, rhs: Self) {
                        *self = Sub::sub(*self, rhs);
                    }
                }
                impl MulAssign for Saturating<$t> {
                    fn mul_assign(&mut self, rhs: Self) {
                        *self = Mul::mul(*self, rhs);
                    }
                }
                impl IntBase for Saturating<$t> {
                    type Error = <$t as IntBase>::Error;
                    fn div_euclid(self, rhs: Self) -> Self {
                        Self(self.0.div_euclid(rhs.0))
                    }
                    fn rem_euclid(self, rhs: Self) -> Self {
                        Self(self.0.rem_euclid(rhs.0))
                    }
                    fn pow(self, exp: u32) -> Self {
                        Self(self.0.saturating_pow(exp))
                    }
                    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error> {
                        <$t as IntBase>::from_str_radix(src, radix).map(Self)
                    }
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
                    fn gcd(self, other: Self) -> Self { Self(self.0.gcd(other.0)) }
                }
                impl Signed for Saturating<$signed> {
                    type Unsigned = Saturating<$unsigned>;
                    fn unsigned(self) -> Self::Unsigned { Saturating(TryFrom::try_from(self.0).ok().unwrap_or_else($unsigned::minimum)) }
                    fn abs(self) -> Self { Self(self.0.saturating_abs()) }
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
                    fn trailing_ones(self) -> u32 { self.0.trailing_ones() }
                    fn trailing_zeros(self) -> u32 { self.0.trailing_zeros() }
                }
            )*
        };
    }
    impl_binary_repr_for_saturating!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

    #[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[repr(transparent)]
    pub struct Wrapping<T>(pub T);

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
    impl_binop!(impl<T> BitAnd bitand for Wrapping<T>);
    impl_binop!(impl<T> BitOr bitor for Wrapping<T>);
    impl_binop!(impl<T> BitXor bitxor for Wrapping<T>);
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
                impl Add for Wrapping<$t> {
                    type Output = Self;
                    fn add(self, rhs: Self) -> Self::Output {
                        Self(self.0.wrapping_add(rhs.0))
                    }
                }
                impl Sub for Wrapping<$t> {
                    type Output = Self;
                    fn sub(self, rhs: Self) -> Self::Output {
                        Self(self.0.wrapping_sub(rhs.0))
                    }
                }
                impl Mul for Wrapping<$t> {
                    type Output = Self;
                    fn mul(self, rhs: Self) -> Self::Output {
                        Self(self.0.wrapping_mul(rhs.0))
                    }
                }
                impl Div for Wrapping<$t> {
                    type Output = Self;
                    fn div(self, rhs: Self) -> Self::Output {
                        Self(self.0.wrapping_div(rhs.0))
                    }
                }
                impl Rem for Wrapping<$t> {
                    type Output = Self;
                    fn rem(self, rhs: Self) -> Self::Output {
                        Self(self.0.wrapping_rem(rhs.0))
                    }
                }
                impl AddAssign for Wrapping<$t> {
                    fn add_assign(&mut self, rhs: Self) {
                        *self = Add::add(*self, rhs);
                    }
                }
                impl SubAssign for Wrapping<$t> {
                    fn sub_assign(&mut self, rhs: Self) {
                        *self = Sub::sub(*self, rhs);
                    }
                }
                impl MulAssign for Wrapping<$t> {
                    fn mul_assign(&mut self, rhs: Self) {
                        *self = Mul::mul(*self, rhs);
                    }
                }
                impl DivAssign for Wrapping<$t> {
                    fn div_assign(&mut self, rhs: Self) {
                        *self = Div::div(*self, rhs);
                    }
                }
                impl RemAssign for Wrapping<$t> {
                    fn rem_assign(&mut self, rhs: Self) {
                        *self = Rem::rem(*self, rhs);
                    }
                }
                impl IntBase for Wrapping<$t> {
                    type Error = <$t as IntBase>::Error;
                    fn div_euclid(self, rhs: Self) -> Self {
                        Self(self.0.wrapping_div_euclid(rhs.0))
                    }
                    fn rem_euclid(self, rhs: Self) -> Self {
                        Self(self.0.wrapping_rem_euclid(rhs.0))
                    }
                    fn pow(self, exp: u32) -> Self {
                        Self(self.0.wrapping_pow(exp))
                    }
                    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::Error> {
                        <$t as IntBase>::from_str_radix(src, radix).map(Self)
                    }
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
                    fn gcd(self, other: Self) -> Self { Self(self.0.gcd(other.0)) }
                }
                impl Signed for Wrapping<$signed> {
                    type Unsigned = Wrapping<$unsigned>;
                    fn unsigned(self) -> Self::Unsigned { Wrapping(self.0.unsigned()) }
                    fn abs(self) -> Self { Self(self.0.wrapping_abs()) }
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
                    fn trailing_ones(self) -> u32 { self.0.trailing_ones() }
                    fn trailing_zeros(self) -> u32 { self.0.trailing_zeros() }
                }
            )*
        };
    }
    impl_binary_repr_for_wrapping!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
}

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
                    const A: $t = $t::max_value() / 2;
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
                            for (a, b) in rng.gen_iter((0..=A, 0..=A)).take(Q) {
                            assert_eq!(a.gcd(b), gcd(a, b));
                        }
                        assert_eq!($t::zero().gcd(0), 0);
                        assert_eq!($t::zero().gcd(100), 100);
                    }
                    #[test]
                    fn test_extgcd() {
                        let mut rng = Xorshift::default();
                        for (a, b) in rng.gen_iter((0..=A, 0..=A)).take(Q) {
                            let ExtendedGcd { g, x, y } = a.extgcd(b);
                            assert_eq!(g, a.gcd(b));
                            assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g as i128);
                        }
                    }
                    #[test]
                    fn test_modinv() {
                        let mut rng = Xorshift::default();
                        for _ in 0..Q {
                            let m = rng.gen(2..=A);
                            let a = rng.gen(1..m);
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
}
