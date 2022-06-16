use super::{Bounded, One, Zero};
use std::{
    cmp::Ordering,
    convert::TryInto,
    fmt::Display,
    num::FpCategory,
    ops::{Add, Div, Mul, Neg, Rem, Sub},
    str::FromStr,
};

pub trait Float:
    Copy
    + Default
    + Display
    + FromStr
    + PartialEq
    + PartialOrd
    + Zero
    + One
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + Rem<Output = Self>
{
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
    fn copysign(self, sign: Self) -> Self;
    fn mul_add(self, a: Self, b: Self) -> Self;
    fn div_euclid(self, rhs: Self) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;
    fn powi(self, n: i32) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn ln(self) -> Self;
    fn log(self, base: Self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn cbrt(self) -> Self;
    fn hypot(self, other: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn sin_cos(self) -> (Self, Self);
    fn exp_m1(self) -> Self;
    fn ln_1p(self) -> Self;
    fn sinh(self) -> Self;
    fn cosh(self) -> Self;
    fn tanh(self) -> Self;
    fn asinh(self) -> Self;
    fn acosh(self) -> Self;
    fn atanh(self) -> Self;
    fn is_nan(self) -> bool;
    fn is_infinite(self) -> bool;
    fn is_finite(self) -> bool;
    fn is_normal(self) -> bool;
    fn classify(self) -> FpCategory;
    fn is_sign_positive(self) -> bool;
    fn is_sign_negative(self) -> bool;
    fn recip(self) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn to_bits(self) -> u64;
    fn from_bits(v: u64) -> Self;
    fn total_cmp(&self, other: &Self) -> Ordering;
    const RADIX: u32;
    const MANTISSA_DIGITS: u32;
    const DIGITS: u32;
    const EPSILON: Self;
    const MIN: Self;
    const MIN_POSITIVE: Self;
    const MAX: Self;
    const MIN_EXP: i32;
    const MAX_EXP: i32;
    const MIN_10_EXP: i32;
    const MAX_10_EXP: i32;
    const NAN: Self;
    const INFINITY: Self;
    const NEG_INFINITY: Self;
    const PI: Self;
    const TAU: Self;
    const FRAC_PI_2: Self;
    const FRAC_PI_3: Self;
    const FRAC_PI_4: Self;
    const FRAC_PI_6: Self;
    const FRAC_PI_8: Self;
    const FRAC_1_PI: Self;
    const FRAC_2_PI: Self;
    const FRAC_2_SQRT_PI: Self;
    const SQRT_2: Self;
    const FRAC_1_SQRT_2: Self;
    const E: Self;
    const LOG2_E: Self;
    const LOG10_E: Self;
    const LN_2: Self;
    const LN_10: Self;
}

macro_rules! primitive_float_impls {
    ($({$t:ident $i:ident $u:ident $e:expr})*) => {$(
        impl Float for $t {
            fn floor(self) -> Self { $t::floor(self) }
            fn ceil(self) -> Self { $t::ceil(self) }
            fn round(self) -> Self { $t::round(self) }
            fn trunc(self) -> Self { $t::trunc(self) }
            fn fract(self) -> Self { $t::fract(self) }
            fn abs(self) -> Self { $t::abs(self) }
            fn signum(self) -> Self { $t::signum(self) }
            fn copysign(self, sign: Self) -> Self { $t::copysign(self, sign) }
            fn mul_add(self, a: Self, b: Self) -> Self { $t::mul_add(self, a, b) }
            fn div_euclid(self, rhs: Self) -> Self { $t::div_euclid(self, rhs) }
            fn rem_euclid(self, rhs: Self) -> Self { $t::rem_euclid(self, rhs) }
            fn powi(self, n: i32) -> Self { $t::powi(self, n) }
            fn powf(self, n: Self) -> Self { $t::powf(self, n) }
            fn sqrt(self) -> Self { $t::sqrt(self) }
            fn exp(self) -> Self { $t::exp(self) }
            fn exp2(self) -> Self { $t::exp2(self) }
            fn ln(self) -> Self { $t::ln(self) }
            fn log(self, base: Self) -> Self { $t::log(self, base) }
            fn log2(self) -> Self { $t::log2(self) }
            fn log10(self) -> Self { $t::log10(self) }
            fn cbrt(self) -> Self { $t::cbrt(self) }
            fn hypot(self, other: Self) -> Self { $t::hypot(self, other) }
            fn sin(self) -> Self { $t::sin(self) }
            fn cos(self) -> Self { $t::cos(self) }
            fn tan(self) -> Self { $t::tan(self) }
            fn asin(self) -> Self { $t::asin(self) }
            fn acos(self) -> Self { $t::acos(self) }
            fn atan(self) -> Self { $t::atan(self) }
            fn atan2(self, other: Self) -> Self { $t::atan2(self, other) }
            fn sin_cos(self) -> (Self, Self) { $t::sin_cos(self) }
            fn exp_m1(self) -> Self { $t::exp_m1(self) }
            fn ln_1p(self) -> Self { $t::ln_1p(self) }
            fn sinh(self) -> Self { $t::sinh(self) }
            fn cosh(self) -> Self { $t::cosh(self) }
            fn tanh(self) -> Self { $t::tanh(self) }
            fn asinh(self) -> Self { $t::asinh(self) }
            fn acosh(self) -> Self { $t::acosh(self) }
            fn atanh(self) -> Self { $t::atanh(self) }
            fn is_nan(self) -> bool { $t::is_nan(self) }
            fn is_infinite(self) -> bool { $t::is_infinite(self) }
            fn is_finite(self) -> bool { $t::is_finite(self) }
            fn is_normal(self) -> bool { $t::is_normal(self) }
            fn classify(self) -> std::num::FpCategory { $t::classify(self) }
            fn is_sign_positive(self) -> bool { $t::is_sign_positive(self) }
            fn is_sign_negative(self) -> bool { $t::is_sign_negative(self) }
            fn recip(self) -> Self { $t::recip(self) }
            fn to_degrees(self) -> Self { $t::to_degrees(self) }
            fn to_radians(self) -> Self { $t::to_radians(self) }
            fn max(self, other: Self) -> Self { $t::max(self, other) }
            fn min(self, other: Self) -> Self { $t::min(self, other) }
            fn to_bits(self) -> u64 { $t::to_bits(self).into() }
            fn from_bits(v: u64) -> Self { $t::from_bits(v.try_into().unwrap()) }
            fn total_cmp(&self, other: &Self) -> Ordering {
                let mut left = self.to_bits() as $i;
                let mut right = other.to_bits() as $i;
                left ^= (((left >> $e) as $u) >> 1) as $i;
                right ^= (((right >> $e) as $u) >> 1) as $i;
                left.cmp(&right)
            }
            const RADIX: u32 = std::$t::RADIX;
            const MANTISSA_DIGITS: u32 = std::$t::MANTISSA_DIGITS;
            const DIGITS: u32 = std::$t::DIGITS;
            const EPSILON: Self = std::$t::EPSILON;
            const MIN: Self = std::$t::MIN;
            const MIN_POSITIVE: Self = std::$t::MIN_POSITIVE;
            const MAX: Self = std::$t::MAX;
            const MIN_EXP: i32 = std::$t::MIN_EXP;
            const MAX_EXP: i32 = std::$t::MAX_EXP;
            const MIN_10_EXP: i32 = std::$t::MIN_10_EXP;
            const MAX_10_EXP: i32 = std::$t::MAX_10_EXP;
            const NAN: Self = std::$t::NAN;
            const INFINITY: Self = std::$t::INFINITY;
            const NEG_INFINITY: Self = std::$t::NEG_INFINITY;
            const PI: Self = std::$t::consts::PI;
            const TAU: Self = std::$t::consts::PI * 2.0;
            const FRAC_PI_2: Self = std::$t::consts::FRAC_PI_2;
            const FRAC_PI_3: Self = std::$t::consts::FRAC_PI_3;
            const FRAC_PI_4: Self = std::$t::consts::FRAC_PI_4;
            const FRAC_PI_6: Self = std::$t::consts::FRAC_PI_6;
            const FRAC_PI_8: Self = std::$t::consts::FRAC_PI_8;
            const FRAC_1_PI: Self = std::$t::consts::FRAC_1_PI;
            const FRAC_2_PI: Self = std::$t::consts::FRAC_2_PI;
            const FRAC_2_SQRT_PI: Self = std::$t::consts::FRAC_2_SQRT_PI;
            const SQRT_2: Self = std::$t::consts::SQRT_2;
            const FRAC_1_SQRT_2: Self = std::$t::consts::FRAC_1_SQRT_2;
            const E: Self = std::$t::consts::E;
            const LOG2_E: Self = std::$t::consts::LOG2_E;
            const LOG10_E: Self = std::$t::consts::LOG10_E;
            const LN_2: Self = std::$t::consts::LN_2;
            const LN_10: Self = std::$t::consts::LN_10;
        })*
    };
}
primitive_float_impls!({f32 i32 u32 31} {f64 i64 u64 63});

macro_rules! ord_float_impls {
    ($({$t:ident $n:ident})*) => {$(
        #[derive(Debug, Copy, Clone, PartialEq, Default)]
        #[repr(transparent)]
        pub struct $n(pub $t);
        impl std::fmt::Display for $n {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <$t as std::fmt::Display>::fmt(&self.0, f)
            }
        }
        impl std::str::FromStr for $n {
            type Err = std::num::ParseFloatError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$t as std::str::FromStr>::from_str(s).map(Self)
            }
        }
        impl From<$t> for $n {
            fn from(x: $t) -> Self {
                Self(x)
            }
        }
        impl Zero for $n {
            fn zero() -> Self {
                Self(<$t as Zero>::zero())
            }
        }
        impl One for $n {
            fn one() -> Self {
                Self(<$t as One>::one())
            }
        }
        impl Bounded for $n {
            fn maximum() -> Self {
                Self(<$t as Bounded>::maximum())
            }
            fn minimum() -> Self {
                Self(<$t as Bounded>::minimum())
            }
        }
        impl Add for $n {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(<$t as Add>::add(self.0, rhs.0))
            }
        }
        impl Sub for $n {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(<$t as Sub>::sub(self.0, rhs.0))
            }
        }
        impl Mul for $n {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(<$t as Mul>::mul(self.0, rhs.0))
            }
        }
        impl Div for $n {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(<$t as Div>::div(self.0, rhs.0))
            }
        }
        impl Neg for $n {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self(<$t as Neg>::neg(self.0))
            }
        }
        impl Rem for $n {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(<$t as Rem>::rem(self.0, rhs.0))
            }
        }
        impl Eq for $n {}
        impl PartialOrd for $n {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.total_cmp(other))
            }
        }
        impl Ord for $n {
            fn cmp(&self, other: &Self) -> Ordering {
                self.partial_cmp(other).unwrap()
            }
        }
        impl Float for $n {
            fn floor(self) -> Self { Self(<$t as Float>::floor(self.0)) }
            fn ceil(self) -> Self { Self(<$t as Float>::ceil(self.0)) }
            fn round(self) -> Self { Self(<$t as Float>::round(self.0)) }
            fn trunc(self) -> Self { Self(<$t as Float>::trunc(self.0)) }
            fn fract(self) -> Self { Self(<$t as Float>::fract(self.0)) }
            fn abs(self) -> Self { Self(<$t as Float>::abs(self.0)) }
            fn signum(self) -> Self { Self(<$t as Float>::signum(self.0)) }
            fn copysign(self, sign: Self) -> Self { Self(<$t as Float>::copysign(self.0, sign.0)) }
            fn mul_add(self, a: Self, b: Self) -> Self { Self(<$t as Float>::mul_add(self.0, a.0, b.0)) }
            fn div_euclid(self, rhs: Self) -> Self { Self(<$t as Float>::div_euclid(self.0, rhs.0)) }
            fn rem_euclid(self, rhs: Self) -> Self { Self(<$t as Float>::rem_euclid(self.0, rhs.0)) }
            fn powi(self, n: i32) -> Self { Self(<$t as Float>::powi(self.0, n)) }
            fn powf(self, n: Self) -> Self { Self(<$t as Float>::powf(self.0, n.0)) }
            fn sqrt(self) -> Self { Self(<$t as Float>::sqrt(self.0)) }
            fn exp(self) -> Self { Self(<$t as Float>::exp(self.0)) }
            fn exp2(self) -> Self { Self(<$t as Float>::exp2(self.0)) }
            fn ln(self) -> Self { Self(<$t as Float>::ln(self.0)) }
            fn log(self, base: Self) -> Self { Self(<$t as Float>::log(self.0, base.0)) }
            fn log2(self) -> Self { Self(<$t as Float>::log2(self.0)) }
            fn log10(self) -> Self { Self(<$t as Float>::log10(self.0)) }
            fn cbrt(self) -> Self { Self(<$t as Float>::cbrt(self.0)) }
            fn hypot(self, other: Self) -> Self { Self(<$t as Float>::hypot(self.0, other.0)) }
            fn sin(self) -> Self { Self(<$t as Float>::sin(self.0)) }
            fn cos(self) -> Self { Self(<$t as Float>::cos(self.0)) }
            fn tan(self) -> Self { Self(<$t as Float>::tan(self.0)) }
            fn asin(self) -> Self { Self(<$t as Float>::asin(self.0)) }
            fn acos(self) -> Self { Self(<$t as Float>::acos(self.0)) }
            fn atan(self) -> Self { Self(<$t as Float>::atan(self.0)) }
            fn atan2(self, other: Self) -> Self { Self(<$t as Float>::atan2(self.0, other.0)) }
            fn sin_cos(self) -> (Self, Self) { let (sin, cos) = <$t as Float>::sin_cos(self.0); (Self(sin), Self(cos)) }
            fn exp_m1(self) -> Self { Self(<$t as Float>::exp_m1(self.0)) }
            fn ln_1p(self) -> Self { Self(<$t as Float>::ln_1p(self.0)) }
            fn sinh(self) -> Self { Self(<$t as Float>::sinh(self.0)) }
            fn cosh(self) -> Self { Self(<$t as Float>::cosh(self.0)) }
            fn tanh(self) -> Self { Self(<$t as Float>::tanh(self.0)) }
            fn asinh(self) -> Self { Self(<$t as Float>::asinh(self.0)) }
            fn acosh(self) -> Self { Self(<$t as Float>::acosh(self.0)) }
            fn atanh(self) -> Self { Self(<$t as Float>::atanh(self.0)) }
            fn is_nan(self) -> bool { <$t as Float>::is_nan(self.0) }
            fn is_infinite(self) -> bool { <$t as Float>::is_infinite(self.0) }
            fn is_finite(self) -> bool { <$t as Float>::is_finite(self.0) }
            fn is_normal(self) -> bool { <$t as Float>::is_normal(self.0) }
            fn classify(self) -> std::num::FpCategory { <$t as Float>::classify(self.0) }
            fn is_sign_positive(self) -> bool { <$t as Float>::is_sign_positive(self.0) }
            fn is_sign_negative(self) -> bool { <$t as Float>::is_sign_negative(self.0) }
            fn recip(self) -> Self { Self(<$t as Float>::recip(self.0)) }
            fn to_degrees(self) -> Self { Self(<$t as Float>::to_degrees(self.0)) }
            fn to_radians(self) -> Self { Self(<$t as Float>::to_radians(self.0)) }
            fn max(self, other: Self) -> Self { Self(<$t as Float>::max(self.0, other.0)) }
            fn min(self, other: Self) -> Self { Self(<$t as Float>::min(self.0, other.0)) }
            fn to_bits(self) -> u64 { <$t as Float>::to_bits(self.0) }
            fn from_bits(v: u64) -> Self { Self(<$t as Float>::from_bits(v)) }
            fn total_cmp(&self, other: &Self) -> Ordering { <$t as Float>::total_cmp(&self.0, &other.0) }
            const RADIX: u32 = <$t as Float>::RADIX;
            const MANTISSA_DIGITS: u32 = <$t as Float>::MANTISSA_DIGITS;
            const DIGITS: u32 = <$t as Float>::DIGITS;
            const EPSILON: Self = Self(<$t as Float>::EPSILON);
            const MIN: Self = Self(<$t as Float>::MIN);
            const MIN_POSITIVE: Self = Self(<$t as Float>::MIN_POSITIVE);
            const MAX: Self = Self(<$t as Float>::MAX);
            const MIN_EXP: i32 = <$t as Float>::MIN_EXP;
            const MAX_EXP: i32 = <$t as Float>::MAX_EXP;
            const MIN_10_EXP: i32 = <$t as Float>::MIN_10_EXP;
            const MAX_10_EXP: i32 = <$t as Float>::MAX_10_EXP;
            const NAN: Self = Self(<$t as Float>::NAN);
            const INFINITY: Self = Self(<$t as Float>::INFINITY);
            const NEG_INFINITY: Self = Self(<$t as Float>::NEG_INFINITY);
            const PI: Self = Self(<$t as Float>::PI);
            const TAU: Self = Self(<$t as Float>::TAU);
            const FRAC_PI_2: Self = Self(<$t as Float>::FRAC_PI_2);
            const FRAC_PI_3: Self = Self(<$t as Float>::FRAC_PI_3);
            const FRAC_PI_4: Self = Self(<$t as Float>::FRAC_PI_4);
            const FRAC_PI_6: Self = Self(<$t as Float>::FRAC_PI_6);
            const FRAC_PI_8: Self = Self(<$t as Float>::FRAC_PI_8);
            const FRAC_1_PI: Self = Self(<$t as Float>::FRAC_1_PI);
            const FRAC_2_PI: Self = Self(<$t as Float>::FRAC_2_PI);
            const FRAC_2_SQRT_PI: Self = Self(<$t as Float>::FRAC_2_SQRT_PI);
            const SQRT_2: Self = Self(<$t as Float>::SQRT_2);
            const FRAC_1_SQRT_2: Self = Self(<$t as Float>::FRAC_1_SQRT_2);
            const E: Self = Self(<$t as Float>::E);
            const LOG2_E: Self = Self(<$t as Float>::LOG2_E);
            const LOG10_E: Self = Self(<$t as Float>::LOG10_E);
            const LN_2: Self = Self(<$t as Float>::LN_2);
            const LN_10: Self = Self(<$t as Float>::LN_10);
        })*
    };
}
ord_float_impls!({f32 Float32} {f64 Float64});
