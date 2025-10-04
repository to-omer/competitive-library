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
    fn mod_inv(self, modulo: Self) -> Self {
        debug_assert!(!modulo.is_zero(), "modulo must be non-zero");
        let extgcd = self.signed().extgcd(modulo.signed());
        debug_assert!(extgcd.g.is_one(), "not coprime");
        extgcd.x.rem_euclid(modulo.signed()).unsigned()
    }
    fn mod_add(self, rhs: Self, modulo: Self) -> Self;
    fn mod_sub(self, rhs: Self, modulo: Self) -> Self;
    fn mod_mul(self, rhs: Self, modulo: Self) -> Self;
    fn mod_neg(self, modulo: Self) -> Self {
        debug_assert!(!modulo.is_zero(), "modulo must be non-zero");
        debug_assert!(self < modulo, "self must be less than modulo");
        if self.is_zero() {
            Self::zero()
        } else {
            modulo - self
        }
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
    ($([$($tt:tt)*])*) => {
        $(impl_unsigned_signed!($($tt)*);)*
    };
    ($unsigned:ident $signed:ident $upperty:ident) => {
        impl_unsigned_signed!(
            @inner $unsigned $signed
            fn mod_mul(self, rhs: Self, modulo: Self) -> Self {
                debug_assert!(!modulo.is_zero(), "modulo must be non-zero");
                (self as $upperty * rhs as $upperty % modulo as $upperty) as $unsigned
            }
        );
    };
    (u128 i128) => {
        impl_unsigned_signed!(
            @inner u128 i128
            fn mod_mul(self, rhs: Self, modulo: Self) -> Self {
                debug_assert!(!modulo.is_zero(), "modulo must be non-zero");
                const MASK64: u128 = 0xffff_ffff_ffff_ffff;
                let (au, ad) = (self >> 64, self & MASK64);
                let (bu, bd) = (rhs >> 64, rhs & MASK64);
                let p0 = ad * bd % modulo;
                let p2 = au * bu % modulo;
                let mut x = [
                    p0 as u64,
                    (p0 >> 64) as u64,
                    p2 as u64,
                    (p2 >> 64) as u64,
                ];
                let p1 = (au * bd % modulo).mod_add(ad * bu % modulo, modulo);
                let (p1_lo, p1_hi) = ((p1 & MASK64) as u64, (p1 >> 64) as u64);
                let (s1, c1) = x[1].overflowing_add(p1_lo);
                x[1] = s1 as u64;
                let (s2, c2) = x[2].overflowing_add(p1_hi + c1 as u64);
                x[2] = s2 as u64;
                let (s3, _) = x[3].overflowing_add(c2 as u64);
                x[3] = s3 as u64;
                rem_u256_by_u128(x, modulo)
            }
        );
    };
    (@inner $unsigned:ident $signed:ident $mod_mul:item) => {
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
            fn mod_add(self, rhs: Self, modulo: Self) -> Self {
                debug_assert!(!modulo.is_zero(), "modulo must be non-zero");
                debug_assert!(self < modulo, "self must be less than modulo");
                debug_assert!(rhs < modulo, "rhs must be less than modulo");
                let s = self.wrapping_add(rhs);
                if (s < self) || (s >= modulo) { s.wrapping_sub(modulo) } else { s }
            }
            fn mod_sub(self, rhs: Self, modulo: Self) -> Self {
                debug_assert!(!modulo.is_zero(), "modulo must be non-zero");
                debug_assert!(self < modulo, "self must be less than modulo");
                debug_assert!(rhs < modulo, "rhs must be less than modulo");
                let d = self.wrapping_sub(rhs);
                if self < rhs { d.wrapping_add(modulo) } else { d }
            }
            $mod_mul
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
    };
}
impl_unsigned_signed!([u8 i8 u16] [u16 i16 u32] [u32 i32 u64] [u64 i64 u128] [u128 i128] [usize isize u128]);

fn rem_u256_by_u128(u: [u64; 4], v: u128) -> u128 {
    // FIXME: use carrying_add and carrying_sub when stabilized
    #[inline(always)]
    fn sub_with_borrow_u64(lhs: u64, rhs: u64, borrow: bool) -> (u64, bool) {
        let (res, overflow) = lhs.overflowing_sub(rhs);
        if borrow {
            let (res, overflow_borrow) = res.overflowing_sub(1);
            (res, overflow | overflow_borrow)
        } else {
            (res, overflow)
        }
    }

    #[inline(always)]
    fn add_with_carry_u64(lhs: u64, rhs: u64, carry: bool) -> (u64, bool) {
        let (res, overflow) = lhs.overflowing_add(rhs);
        if carry {
            let (res, overflow_carry) = res.overflowing_add(1);
            (res, overflow | overflow_carry)
        } else {
            (res, overflow)
        }
    }

    debug_assert!(v != 0);
    let v_hi = (v >> 64) as u64;
    if v_hi == 0 {
        let d = v as u64 as u128;
        let mut rem: u128 = 0;
        for &w in u.iter().rev() {
            rem = (rem << 64 | w as u128) % d;
        }
        return rem;
    }

    let v_lo = v as u64;
    let v_shift = v_hi.leading_zeros();
    let (vn1, vn0) = if v_shift == 0 {
        (v_hi, v_lo)
    } else {
        let hi = v_hi << v_shift | v_lo >> (64 - v_shift);
        let lo = v_lo << v_shift;
        (hi, lo)
    };

    let mut un = [0u64; 5];
    if v_shift == 0 {
        un[0] = u[0];
        un[1] = u[1];
        un[2] = u[2];
        un[3] = u[3];
    } else {
        un[0] = u[0] << v_shift;
        un[1] = u[1] << v_shift | u[0] >> (64 - v_shift);
        un[2] = u[2] << v_shift | u[1] >> (64 - v_shift);
        un[3] = u[3] << v_shift | u[2] >> (64 - v_shift);
        un[4] = u[3] >> (64 - v_shift);
    }

    for j in (0..=2).rev() {
        let num = (un[j + 2] as u128) << 64 | un[j + 1] as u128;
        let (mut qhat, mut rhat) = if un[j + 2] == vn1 {
            (u64::MAX, un[j + 1])
        } else {
            let d = vn1 as u128;
            ((num / d) as u64, (num % d) as u64)
        };
        while qhat as u128 * vn0 as u128 > (rhat as u128) << 64 | un[j] as u128 {
            qhat -= 1;
            let t = rhat as u128 + vn1 as u128;
            if t >= 1u128 << 64 {
                break;
            }
            rhat = t as u64;
        }

        let p0 = qhat as u128 * vn0 as u128;
        let p1 = qhat as u128 * vn1 as u128;
        let (p0_hi, p0_lo) = ((p0 >> 64) as u64, p0 as u64);
        let (p1_hi, p1_lo) = ((p1 >> 64) as u64, p1 as u64);

        let (r0, borrow) = sub_with_borrow_u64(un[j], p0_lo, false);
        un[j] = r0;

        let (r1, borrow1) = sub_with_borrow_u64(un[j + 1], p0_hi, borrow);
        let (r1, borrow2) = sub_with_borrow_u64(r1, p1_lo, false);
        let borrow = borrow1 || borrow2;
        un[j + 1] = r1;

        let (r2, borrow) = sub_with_borrow_u64(un[j + 2], p1_hi, borrow);
        un[j + 2] = r2;

        if borrow {
            let (s0, carry) = add_with_carry_u64(un[j], vn0, false);
            un[j] = s0;
            let (s1, carry) = add_with_carry_u64(un[j + 1], vn1, carry);
            un[j + 1] = s1;
            let (s2, _) = un[j + 2].overflowing_add(carry as u64);
            un[j + 2] = s2;
        }
    }

    ((un[1] as u128) << 64 | un[0] as u128) >> v_hi.leading_zeros()
}

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
                fn mod_add(self, rhs: Self, modulo: Self) -> Self { Self(self.0.mod_add(rhs.0, modulo.0)) }
                fn mod_sub(self, rhs: Self, modulo: Self) -> Self { Self(self.0.mod_sub(rhs.0, modulo.0)) }
                fn mod_mul(self, rhs: Self, modulo: Self) -> Self { Self(self.0.mod_mul(rhs.0, modulo.0)) }
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
                fn mod_add(self, rhs: Self, modulo: Self) -> Self { Self(self.0.mod_add(rhs.0, modulo.0)) }
                fn mod_sub(self, rhs: Self, modulo: Self) -> Self { Self(self.0.mod_sub(rhs.0, modulo.0)) }
                fn mod_mul(self, rhs: Self, modulo: Self) -> Self { Self(self.0.mod_mul(rhs.0, modulo.0)) }
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

    mod int_base {
        macro_rules! test_intbase {
            ($($t:ident)*) => {
                $(
                    mod $t {
                        use super::super::*;

                        #[test]
                        fn test_intbase() {
                            assert_eq!(<$t as IntBase>::div_euclid(10, 3), 3);
                            assert_eq!(<$t as IntBase>::rem_euclid(10, 3), 1);
                            assert_eq!(<$t as IntBase>::pow(10, 2), 100);
                            assert_eq!(<$t as IntBase>::from_str_radix("1a", 16).unwrap(), 26 as $t);
                            assert_eq!(<$t as IntBase>::ilog(100 as $t, 10 as $t), 2);
                            assert_eq!(<$t as IntBase>::ilog2(16 as $t), 4);
                            assert_eq!(<$t as IntBase>::ilog10(100 as $t), 2);
                        }
                    }
                )*
            };
        }
        test_intbase!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
    }

    mod unsigned {
        use super::*;

        macro_rules! test_unsigned {
            ($($t:ident)*) => {
                $(
                    mod $t {
                        use super::super::*;
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
                        fn test_mod_inv() {
                            let mut rng = Xorshift::default();
                            for _ in 0..Q {
                                let m = rng.random(2..=A);
                                let a = rng.random(1..m);
                                let g = a.gcd(m);
                                let m = m / g;
                                let a = a / g;
                                let x = a.mod_inv(m);
                                assert!(x < m);
                                assert_eq!(a as u128 * x as u128 % m as u128, 1);
                            }
                        }
                        #[test]
                        fn test_mod_operate() {
                            let mut rng = Xorshift::default();
                            for _ in 0..Q {
                                for ub in [10, A] {
                                    let m = rng.random(2..=ub);
                                    let a = rng.random(0..m);
                                    let b = rng.random(0..m);
                                    assert_eq!(a.mod_add(b, m), ((a as u128 + b as u128) % m as u128) as $t);
                                    assert_eq!(a.mod_sub(b, m), ((a as u128 + m as u128 - b as u128) % m as u128) as $t);
                                    assert_eq!(a.mod_mul(b, m), (a as u128 * b as u128 % m as u128) as $t);
                                    assert_eq!(a.mod_mul(b, m), (a as u128).mod_mul(b as u128, m as u128) as $t);
                                    assert_eq!(a.mod_neg(m), ((m as u128 - a as u128) % m as u128) as $t);
                                }
                            }
                        }
                        #[test]
                        fn test_unsigned() {
                            assert_eq!(<$t as Unsigned>::signed(0), 0);
                            assert_eq!(<$t as Unsigned>::abs_diff(10, 20), 10);
                            assert_eq!(<$t as Unsigned>::next_power_of_two(10), 16);
                            assert_eq!(<$t as Unsigned>::gcd(100, 80), 20);
                            assert_eq!(<$t as Unsigned>::lcm(12, 15), 60);
                            assert_eq!(<$t as Unsigned>::lcm(0, 1), 0);
                            assert_eq!(<$t as Unsigned>::lcm(0, 0), 0);
                        }
                    }
                )*
            };
        }
        test_unsigned!(u8 u16 u32 u64 usize);

        #[test]
        fn test_mod_mul_u128() {
            fn naive_mod_mul(a: u128, b: u128, m: u128) -> u128 {
                assert!(m != 0);
                let a = [a as u64, (a >> 64) as u64];
                let b = [b as u64, (b >> 64) as u64];
                let mut res = 0u128;
                for (i, &a) in a.iter().enumerate() {
                    for (j, &b) in b.iter().enumerate() {
                        let mut x = (a as u128) * (b as u128) % m;
                        for _ in 0..(i + j) * 64 {
                            x = x.mod_add(x, m);
                        }
                        res = res.mod_add(x, m);
                    }
                }
                res
            }

            let mut rng = Xorshift::default();
            for _ in 0..100 {
                for a in [1, 10, u32::MAX as _, u64::MAX as _, u128::MAX] {
                    for b in [1, 10, u32::MAX as _, u64::MAX as _, u128::MAX] {
                        for c in [1, 10, u32::MAX as _, u64::MAX as _, u128::MAX] {
                            let m = rng.random(1..=c);
                            let x = rng.random(0..a.min(m));
                            let y = rng.random(0..b.min(m));
                            assert_eq!(x.mod_mul(y, m), naive_mod_mul(x, y, m));
                            let x = rng.random(0..a);
                            let y = rng.random(0..b);
                            assert_eq!(x.mod_mul(y, m), naive_mod_mul(x, y, m));
                        }
                    }
                }
            }
        }

        #[test]
        fn test_rem() {
            fn naive_rem(u: [u64; 4], v: u128) -> u128 {
                assert!(v != 0);
                let mut u = [
                    ((u[1] as u128) << 64) | (u[0] as u128),
                    ((u[3] as u128) << 64) | (u[2] as u128),
                ];
                let mut v_mul_2 = vec![[v, 0]];
                while v_mul_2.last().unwrap()[1].leading_zeros() != 0 {
                    let [v_lo, v_hi] = *v_mul_2.last().unwrap();
                    v_mul_2.push([v_lo << 1, v_hi << 1 | (v_lo >> 127)]);
                }
                v_mul_2.reverse();
                for [v_lo, v_hi] in v_mul_2 {
                    let [u_lo, u_hi] = u;
                    if (u_hi > v_hi) || (u_hi == v_hi && u_lo >= v_lo) {
                        let (new_lo, carry) = u_lo.overflowing_sub(v_lo);
                        let new_hi = u_hi - v_hi - (carry as u128);
                        u = [new_lo, new_hi];
                    }
                }
                u[0]
            }
            let mut rng = Xorshift::default();
            for _ in 0..1000 {
                let mut u = [0u64; 4];
                for k in 0..4 {
                    for a in [1, 10, u32::MAX as _, u64::MAX] {
                        u[k] = rng.random(..a);
                        for b in [1, 10, u128::MAX] {
                            let v = rng.random(1..=b);
                            assert_eq!(rem_u256_by_u128(u, v), naive_rem(u, v));
                        }
                    }
                }
            }
        }
    }

    mod signed {
        macro_rules! test_signed {
            ($($t:ident)*) => {
                $(
                    mod $t {
                        use super::super::*;
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
                        #[test]
                        fn test_signed() {
                            assert_eq!(<$t as Signed>::unsigned(0), 0);
                            assert_eq!(<$t as Signed>::abs(-10), 10);
                            assert_eq!(<$t as Signed>::abs_diff(10, -20), 30);
                            assert!(!<$t as Signed>::is_negative(10));
                            assert!(<$t as Signed>::is_negative(-10));
                            assert!(<$t as Signed>::is_positive(10));
                            assert!(!<$t as Signed>::is_positive(-10));
                            assert_eq!(<$t as Signed>::signum(10), 1);
                            assert_eq!(<$t as Signed>::signum(-10), -1);
                            assert_eq!(<$t as Signed>::signum(0), 0);
                        }
                    }
                )*
            };
        }
        test_signed!(i8 i16 i32 i64 isize);
    }

    macro_rules! test_binary_repr {
        ($($t:ident)*) => {
            $(
                mod $t {
                    use super::super::*;
                    #[test]
                    fn test_binary_repr() {
                        assert_eq!(<$t as BinaryRepr>::count_ones(0b1010), 2);
                        assert_eq!(<$t as BinaryRepr>::count_zeros(0b1010), <$t>::BITS - 2);
                        assert_eq!(<$t as BinaryRepr>::leading_ones(!0b0010), <$t>::BITS - 2);
                        assert_eq!(<$t as BinaryRepr>::leading_zeros(0b0010), <$t>::BITS - 2);
                        assert_eq!(<$t as BinaryRepr>::reverse_bits(0b101), 0b101 << (<$t>::BITS - 3));
                        assert_eq!(<$t as BinaryRepr>::rotate_left(0b0001_0010, 2), 0b0100_1000);
                        assert_eq!(<$t as BinaryRepr>::rotate_right(0b0001_0010, <$t>::BITS - 2), 0b0100_1000);
                        assert_eq!(<$t as BinaryRepr>::swap_bytes(0b0001_0010), 0b0001_0010 << (<$t>::BITS - 8));
                        assert_eq!(<$t as BinaryRepr>::trailing_ones(!0b0100), 2);
                        assert_eq!(<$t as BinaryRepr>::trailing_zeros(0b0100), 2);
                    }
                }
            )*
        }
    }
    mod binary_repr {
        test_binary_repr!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
    }

    mod saturating {
        macro_rules! test_saturating {
            ($($unsigned:ident $signed:ident)*) => {
                $(
                    mod $unsigned {
                        use super::super::*;
                        type S = Saturating<$unsigned>;

                        test_saturating!(@common $unsigned);
                        test_saturating!(@unsigned $unsigned);
                    }
                    mod $signed {
                        use super::super::*;
                        type S = Saturating<$signed>;

                        test_saturating!(@common $signed);
                        test_saturating!(@signed $signed);
                    }
                )*
            };
            (@common $t:ident) => {
                macro_rules! assign {
                    ($op:tt, $left:expr, $right:expr) => {{
                        let mut a = $left;
                        a $op $right;
                        a
                    }};
                }

                #[test]
                fn test_saturating() {
                    assert_eq!((1 as $t).to_saturating(), S::from(1));
                    assert_eq!($t::from_saturating((1 as $t).to_saturating()), 1 as $t);
                    assert_eq!(S::maximum(), S::from($t::MAX));
                    assert_eq!(S::minimum(), S::from($t::MIN));
                    assert_eq!(S::zero(), S::from(0));
                    assert_eq!(S::one(), S::from(1));
                    assert_eq!(S::from(1 as $t).to_string(), "1");
                    assert_eq!(S::from_str("123").unwrap(), S::from(123));
                    assert_eq!(format!("{:?}", S::from(123)), "123");
                    assert_eq!(S::scan(&mut ["123"].iter().map(|s| *s)).unwrap(), S::from(123));
                    assert_eq!(S::from(1) + S::from(2), S::from(3));
                    assert_eq!(S::from(1) + 2, S::from(3));
                    assert_eq!(S::from(3) - S::from(1), S::from(2));
                    assert_eq!(S::from(3) - 1, S::from(2));
                    assert_eq!(S::from(2) * S::from(3), S::from(6));
                    assert_eq!(S::from(2) * 3, S::from(6));
                    assert_eq!(S::from(6) / S::from(3), S::from(2));
                    assert_eq!(S::from(6) / 3, S::from(2));
                    assert_eq!(S::from(7) % S::from(4), S::from(3));
                    assert_eq!(S::from(7) % 4, S::from(3));
                    assert_eq!(S::from(1) & S::from(3), S::from(1));
                    assert_eq!(S::from(1) & 3, S::from(1));
                    assert_eq!(S::from(1) | S::from(2), S::from(3));
                    assert_eq!(S::from(1) | 2, S::from(3));
                    assert_eq!(S::from(3) ^ S::from(1), S::from(2));
                    assert_eq!(S::from(3) ^ 1, S::from(2));
                    assert_eq!(S::from(1) << 2, S::from(4));
                    assert_eq!(S::from(4) >> 2, S::from(1));
                    assert_eq!(assign!(+=, S::from(1), S::from(2)), S::from(3));
                    assert_eq!(assign!(+=, S::from(1), 2), S::from(3));
                    assert_eq!(assign!(-=, S::from(3), S::from(1)), S::from(2));
                    assert_eq!(assign!(-=, S::from(3), 1), S::from(2));
                    assert_eq!(assign!(*=, S::from(2), S::from(3)), S::from(6));
                    assert_eq!(assign!(*=, S::from(2), 3), S::from(6));
                    assert_eq!(assign!(/=, S::from(6), S::from(3)), S::from(2));
                    assert_eq!(assign!(/=, S::from(6), 3), S::from(2));
                    assert_eq!(assign!(%=, S::from(7), S::from(4)), S::from(3));
                    assert_eq!(assign!(%=, S::from(7), 4), S::from(3));
                    assert_eq!(assign!(&=, S::from(1), S::from(3)), S::from(1));
                    assert_eq!(assign!(&=, S::from(1), 3), S::from(1));
                    assert_eq!(assign!(|=, S::from(1), S::from(2)), S::from(3));
                    assert_eq!(assign!(|=, S::from(1), 2), S::from(3));
                    assert_eq!(assign!(^=, S::from(3), S::from(1)), S::from(2));
                    assert_eq!(assign!(^=, S::from(3), 1), S::from(2));
                    assert_eq!(assign!(<<=, S::from(1), 2), S::from(4));
                    assert_eq!(assign!(>>=, S::from(4), 2), S::from(1));
                    assert_eq!(!S::from(1), S::from(!(1 as $t)));
                    assert_eq!([S::from(1), S::from(2)].into_iter().sum::<S>(), S::from(3));
                    assert_eq!([S::from(2), S::from(3)].into_iter().product::<S>(), S::from(6));
                    assert_eq!(S::from(10).div_euclid(S::from(3)), S::from(3));
                    assert_eq!(S::from(10).rem_euclid(S::from(3)), S::from(1));
                    assert_eq!(S::from(10).pow(2), S::from(100));
                    assert_eq!(S::from_str_radix("1a", 16).unwrap(), S::from(26));
                    assert_eq!(S::from(100).ilog(S::from(10)), 2);
                    assert_eq!(S::from(16).ilog2(), 4);
                    assert_eq!(S::from(100).ilog10(), 2);
                    assert_eq!(S::from(0b1010).count_ones(), 2);
                    assert_eq!(S::from(0b1010).count_zeros(), <$t>::BITS - 2);
                    assert_eq!(S::from(!0b0010).leading_ones(), <$t>::BITS - 2);
                    assert_eq!(S::from(0b0010).leading_zeros(), <$t>::BITS - 2);
                    assert_eq!(S::from(0b101).reverse_bits(), S::from(0b101 << (<$t>::BITS - 3)));
                    assert_eq!(S::from(0b0001_0010).rotate_left(2), S::from(0b0100_1000));
                    assert_eq!(S::from(0b0001_0010).rotate_right(<$t>::BITS - 2), S::from(0b0100_1000));
                    assert_eq!(S::from(0b0001_0010).swap_bytes(), S::from(0b0001_0010 << (<$t>::BITS - 8)));
                    assert_eq!(S::from(!0b0100).trailing_ones(), 2);
                    assert_eq!(S::from(0b0100).trailing_zeros(), 2);
                }
            };
            (@unsigned $t:ident) => {
                #[test]
                fn test_saturating_unsigned() {
                    assert_eq!(S::from(0).signed(), Saturating::from(0));
                    assert_eq!(S::from(10).abs_diff(S::from(20)), S::from(10));
                    assert_eq!(S::from(10).next_power_of_two(), S::from(16));
                    assert_eq!(S::from(100).gcd(S::from(80)), S::from(20));
                    assert_eq!(S::from(100).mod_add(S::from(80), S::from(150)), S::from(30));
                    assert_eq!(S::from(100).mod_sub(S::from(80), S::from(150)), S::from(20));
                    assert_eq!(S::from(100).mod_mul(S::from(80), S::from(150)), S::from(50));
                }
            };
            (@signed $t:ident) => {
                #[test]
                fn test_saturating_signed() {
                    assert_eq!(S::from(0).unsigned(), Saturating::from(0));
                    assert_eq!(S::from(-10).abs(), S::from(10));
                    assert_eq!(S::from(10).abs_diff(S::from(-20)), Saturating::from(30));
                    assert!(!S::from(10).is_negative());
                    assert!(S::from(-10).is_negative());
                    assert!(S::from(10).is_positive());
                    assert!(!S::from(-10).is_positive());
                    assert_eq!(S::from(10).signum(), S::from(1));
                    assert_eq!(S::from(-10).signum(), S::from(-1));
                    assert_eq!(S::from(0).signum(), S::from(0));
                    assert_eq!(-S::from(1), S::from(-1));
                }
            };
        }
        test_saturating!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
    }

    mod wrapping {
        macro_rules! test_wrapping {
            ($($unsigned:ident $signed:ident)*) => {
                $(
                    mod $unsigned {
                        use super::super::*;
                        type W = Wrapping<$unsigned>;

                        test_wrapping!(@common $unsigned);
                        test_wrapping!(@unsigned $unsigned);
                    }
                    mod $signed {
                        use super::super::*;
                        type W = Wrapping<$signed>;

                        test_wrapping!(@common $signed);
                        test_wrapping!(@signed $signed);
                    }
                )*
            };
            (@common $t:ident) => {
                macro_rules! assign {
                    ($op:tt, $left:expr, $right:expr) => {{
                        let mut a = $left;
                        a $op $right;
                        a
                    }};
                }

                #[test]
                fn test_wrapping() {
                    assert_eq!((1 as $t).to_wrapping(), W::from(1));
                    assert_eq!($t::from_wrapping((1 as $t).to_wrapping()), 1 as $t);
                    assert_eq!(W::maximum(), W::from($t::MAX));
                    assert_eq!(W::minimum(), W::from($t::MIN));
                    assert_eq!(W::zero(), W::from(0));
                    assert_eq!(W::one(), W::from(1));
                    assert_eq!(W::from(1 as $t).to_string(), "1");
                    assert_eq!(W::from_str("123").unwrap(), W::from(123));
                    assert_eq!(format!("{:?}", W::from(123)), "123");
                    assert_eq!(W::scan(&mut ["123"].iter().map(|s| *s)).unwrap(), W::from(123));
                    assert_eq!(W::from(1) + W::from(2), W::from(3));
                    assert_eq!(W::from(1) + 2, W::from(3));
                    assert_eq!(W::from(3) - W::from(1), W::from(2));
                    assert_eq!(W::from(3) - 1, W::from(2));
                    assert_eq!(W::from(2) * W::from(3), W::from(6));
                    assert_eq!(W::from(2) * 3, W::from(6));
                    assert_eq!(W::from(6) / W::from(3), W::from(2));
                    assert_eq!(W::from(6) / 3, W::from(2));
                    assert_eq!(W::from(7) % W::from(4), W::from(3));
                    assert_eq!(W::from(7) % 4, W::from(3));
                    assert_eq!(W::from(1) & W::from(3), W::from(1));
                    assert_eq!(W::from(1) & 3, W::from(1));
                    assert_eq!(W::from(1) | W::from(2), W::from(3));
                    assert_eq!(W::from(1) | 2, W::from(3));
                    assert_eq!(W::from(3) ^ W::from(1), W::from(2));
                    assert_eq!(W::from(3) ^ 1, W::from(2));
                    assert_eq!(W::from(1) << 2, W::from(4));
                    assert_eq!(W::from(4) >> 2, W::from(1));
                    assert_eq!(assign!(+=, W::from(1), W::from(2)), W::from(3));
                    assert_eq!(assign!(+=, W::from(1), 2), W::from(3));
                    assert_eq!(assign!(-=, W::from(3), W::from(1)), W::from(2));
                    assert_eq!(assign!(-=, W::from(3), 1), W::from(2));
                    assert_eq!(assign!(*=, W::from(2), W::from(3)), W::from(6));
                    assert_eq!(assign!(*=, W::from(2), 3), W::from(6));
                    assert_eq!(assign!(/=, W::from(6), W::from(3)), W::from(2));
                    assert_eq!(assign!(/=, W::from(6), 3), W::from(2));
                    assert_eq!(assign!(%=, W::from(7), W::from(4)), W::from(3));
                    assert_eq!(assign!(%=, W::from(7), 4), W::from(3));
                    assert_eq!(assign!(&=, W::from(1), W::from(3)), W::from(1));
                    assert_eq!(assign!(&=, W::from(1), 3), W::from(1));
                    assert_eq!(assign!(|=, W::from(1), W::from(2)), W::from(3));
                    assert_eq!(assign!(|=, W::from(1), 2), W::from(3));
                    assert_eq!(assign!(^=, W::from(3), W::from(1)), W::from(2));
                    assert_eq!(assign!(^=, W::from(3), 1), W::from(2));
                    assert_eq!(assign!(<<=, W::from(1), 2), W::from(4));
                    assert_eq!(assign!(>>=, W::from(4), 2), W::from(1));
                    assert_eq!(!W::from(1), W::from(!(1 as $t)));
                    assert_eq!([W::from(1), W::from(2)].into_iter().sum::<W>(), W::from(3));
                    assert_eq!([W::from(2), W::from(3)].into_iter().product::<W>(), W::from(6));
                    assert_eq!(W::from(10).div_euclid(W::from(3)), W::from(3));
                    assert_eq!(W::from(10).rem_euclid(W::from(3)), W::from(1));
                    assert_eq!(W::from(10).pow(2), W::from(100));
                    assert_eq!(W::from_str_radix("1a", 16).unwrap(), W::from(26));
                    assert_eq!(W::from(100).ilog(W::from(10)), 2);
                    assert_eq!(W::from(16).ilog2(), 4);
                    assert_eq!(W::from(100).ilog10(), 2);
                    assert_eq!(W::from(0b1010).count_ones(), 2);
                    assert_eq!(W::from(0b1010).count_zeros(), <$t>::BITS - 2);
                    assert_eq!(W::from(!0b0010).leading_ones(), <$t>::BITS - 2);
                    assert_eq!(W::from(0b0010).leading_zeros(), <$t>::BITS - 2);
                    assert_eq!(W::from(0b101).reverse_bits(), W::from(0b101 << (<$t>::BITS - 3)));
                    assert_eq!(W::from(0b0001_0010).rotate_left(2), W::from(0b0100_1000));
                    assert_eq!(W::from(0b0001_0010).rotate_right(<$t>::BITS - 2), W::from(0b0100_1000));
                    assert_eq!(W::from(0b0001_0010).swap_bytes(), W::from(0b0001_0010 << (<$t>::BITS - 8)));
                    assert_eq!(W::from(!0b0100).trailing_ones(), 2);
                    assert_eq!(W::from(0b0100).trailing_zeros(), 2);
                }
            };
            (@unsigned $t:ident) => {
                #[test]
                fn test_wrapping_unsigned() {
                    assert_eq!(W::from(0).signed(), Wrapping::from(0));
                    assert_eq!(W::from(10).abs_diff(W::from(20)), W::from(10));
                    assert_eq!(W::from(10).next_power_of_two(), W::from(16));
                    assert_eq!(W::from(100).gcd(W::from(80)), W::from(20));
                    assert_eq!(W::from(100).mod_add(W::from(80), W::from(150)), W::from(30));
                    assert_eq!(W::from(100).mod_sub(W::from(80), W::from(150)), W::from(20));
                    assert_eq!(W::from(100).mod_mul(W::from(80), W::from(150)), W::from(50));
                }
            };
            (@signed $t:ident) => {
                #[test]
                fn test_wrapping_signed() {
                    assert_eq!(W::from(0).unsigned(), Wrapping::from(0));
                    assert_eq!(W::from(-10).abs(), W::from(10));
                    assert_eq!(W::from(10).abs_diff(W::from(-20)), Wrapping::from(30));
                    assert!(!W::from(10).is_negative());
                    assert!(W::from(-10).is_negative());
                    assert!(W::from(10).is_positive());
                    assert!(!W::from(-10).is_positive());
                    assert_eq!(W::from(10).signum(), W::from(1));
                    assert_eq!(W::from(-10).signum(), W::from(-1));
                    assert_eq!(W::from(0).signum(), W::from(0));
                    assert_eq!(-W::from(1), W::from(-1));
                }
            };
        }
        test_wrapping!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
    }
}
