//! Floating-point expansions for computations within the normal f64 exponent range.
//! Intermediate overflow and underflow are not compensated.
//! Display prints the exact component sum; a precision rounds decimal places.

use super::{Bounded, Decimal, Float, One, Scan, ScanSource, Zero};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    num::ParseFloatError,
    ops::{Add, Div, Index, Mul, Neg, Rem, Sub},
};

/// A nonoverlapping expansion of two f64 components.
/// Nominal precision decreases when trailing components become subnormal.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DoubleDouble(f64, f64);

/// A nonoverlapping expansion of four f64 components.
/// Nominal precision decreases when trailing components become subnormal.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuadDouble(f64, f64, f64, f64);

const LN_2_REDUCTION: [f64; 6] = [
    0.6931471805599453,
    2.3190468138462996e-17,
    5.707708438416212e-34,
    -3.5824322106018114e-50,
    -1.352169675798863e-66,
    6.080638740240814e-83,
];

// Successive nearest-even components, generated with MPFR at 4096 bits.
const PI: [f64; 4] = [
    3.141592653589793,
    1.2246467991473532e-16,
    -2.9947698097183397e-33,
    1.1124542208633653e-49,
];
const TAU: [f64; 4] = [
    6.283185307179586,
    2.4492935982947064e-16,
    -5.989539619436679e-33,
    2.2249084417267306e-49,
];
const FRAC_PI_2: [f64; 4] = [
    1.5707963267948966,
    6.123233995736766e-17,
    -1.4973849048591698e-33,
    5.562271104316826e-50,
];
const FRAC_PI_3: [f64; 4] = [
    1.0471975511965979,
    -1.072081766451091e-16,
    -9.982566032394464e-34,
    -7.6956153601821505e-50,
];
const FRAC_PI_4: [f64; 4] = [
    0.7853981633974483,
    3.061616997868383e-17,
    -7.486924524295849e-34,
    2.781135552158413e-50,
];
const FRAC_PI_6: [f64; 4] = [
    0.5235987755982989,
    -5.360408832255455e-17,
    -4.991283016197232e-34,
    -3.8478076800910752e-50,
];
const FRAC_PI_8: [f64; 4] = [
    0.39269908169872414,
    1.5308084989341915e-17,
    -3.7434622621479246e-34,
    1.3905677760792066e-50,
];
const FRAC_1_PI: [f64; 4] = [
    0.3183098861837907,
    -1.9678676675182486e-17,
    -1.0721436282893004e-33,
    8.053563926594112e-50,
];
const FRAC_2_PI: [f64; 4] = [
    0.6366197723675814,
    -3.935735335036497e-17,
    -2.1442872565786008e-33,
    1.6107127853188224e-49,
];
const FRAC_2_SQRT_PI: [f64; 4] = [
    1.1283791670955126,
    1.533545961316588e-17,
    -4.765684596693686e-34,
    -2.007794661655263e-50,
];
const SQRT_2: [f64; 4] = [
    1.4142135623730951,
    -9.667293313452913e-17,
    4.1386753086994136e-33,
    4.935546991468351e-50,
];
const FRAC_1_SQRT_2: [f64; 4] = [
    0.7071067811865476,
    -4.833646656726457e-17,
    2.0693376543497068e-33,
    2.4677734957341755e-50,
];
const E: [f64; 4] = [
    2.718281828459045,
    1.4456468917292502e-16,
    -2.1277171080381768e-33,
    1.5156301598412191e-49,
];
const LOG2_E: [f64; 4] = [
    1.4426950408889634,
    2.0355273740931033e-17,
    -1.0614659956117258e-33,
    -1.3836716780181402e-50,
];
const LOG10_E: [f64; 4] = [
    0.4342944819032518,
    1.098319650216765e-17,
    3.717181233110959e-34,
    7.734484346504299e-51,
];
const LN_2: [f64; 4] = [
    0.6931471805599453,
    2.3190468138462996e-17,
    5.707708438416212e-34,
    -3.5824322106018114e-50,
];
const LN_10: [f64; 4] = [
    2.302585092994046,
    -2.1707562233822494e-16,
    -9.984262454465777e-33,
    -4.023357454450206e-49,
];

const PI_2_REDUCTION: [f64; 6] = [
    1.5707963267948966,
    6.123233995736766e-17,
    -1.4973849048591698e-33,
    5.562271104316826e-50,
    2.836115989820158e-66,
    8.724931080676243e-84,
];

const fn scalar<const N: usize>(value: f64) -> [f64; N] {
    let mut result = [0.0; N];
    result[0] = value;
    result
}

const fn constant<const N: usize>(values: [f64; 4]) -> [f64; N] {
    let mut result = [0.0; N];
    let mut i = 0;
    while i < N {
        result[i] = values[i];
        i += 1;
    }
    result
}

const fn epsilon<const N: usize>() -> [f64; N] {
    scalar(f64::from_bits(
        ((1023 - if N == 2 { 104 } else { 209 }) as u64) << 52,
    ))
}

fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let v = sum - a;
    (sum, (a - (sum - v)) + (b - v))
}

fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    (sum, b - (sum - a))
}

fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let product = a * b;
    // Avoid a libm call when the target has no hardware FMA.
    let error = if cfg!(any(target_arch = "aarch64", target_feature = "fma")) {
        a.mul_add(b, -product)
    } else {
        let split = |value| {
            let scaled = 134217729.0 * value;
            let high = scaled - (scaled - value);
            (high, value - high)
        };
        let (ah, al) = split(a);
        let (bh, bl) = split(b);
        ((ah * bh - product) + ah * bl + al * bh) + al * bl
    };
    (product, error)
}

fn renormalize<const N: usize, const M: usize>(mut values: [f64; M]) -> [f64; N] {
    if N == 2 && M == 3 {
        let (s, e) = two_sum(values[1], values[2]);
        let (s, t) = two_sum(values[0], s);
        let (high, low) = two_sum(s, t + e);
        let mut result = scalar(high);
        result[1] = low;
        return result;
    }
    if N == 4 && M == 5 {
        let (s, e3) = quick_two_sum(values[3], values[4]);
        let (s, e2) = quick_two_sum(values[2], s);
        let (s, e1) = quick_two_sum(values[1], s);
        let (mut r0, mut r1) = quick_two_sum(values[0], s);
        let (mut r2, mut r3) = (0.0, 0.0);
        if r1 != 0.0 {
            (r1, r2) = quick_two_sum(r1, e1);
            if r2 != 0.0 {
                (r2, r3) = quick_two_sum(r2, e2);
            } else {
                (r1, r2) = quick_two_sum(r1, e2);
            }
        } else {
            (r0, r1) = quick_two_sum(r0, e1);
            if r1 != 0.0 {
                (r1, r2) = quick_two_sum(r1, e2);
            } else {
                (r0, r1) = quick_two_sum(r0, e2);
            }
        }
        if r3 != 0.0 {
            r3 += e3;
        } else if r2 != 0.0 {
            (r2, r3) = quick_two_sum(r2, e3);
        } else if r1 != 0.0 {
            (r1, r2) = quick_two_sum(r1, e3);
        } else {
            (r0, r1) = quick_two_sum(r0, e3);
        }
        let mut result = [0.0; N];
        result[..4].copy_from_slice(&[r0, r1, r2, r3]);
        return result;
    }
    let mut sum = values[M - 1];
    for i in (0..M - 1).rev() {
        (sum, values[i + 1]) = two_sum(values[i], sum);
    }
    let mut result = scalar(sum);
    let mut index = 0;
    for value in values.into_iter().skip(1) {
        let (sum, error) = two_sum(result[index], value);
        result[index] = sum;
        if error != 0.0 && index + 1 < N {
            index += 1;
            result[index] = error;
        }
    }
    result
}

// Retain cancellation terms until the final expansion is rounded.
fn sum_terms<const N: usize, const M: usize>(values: [f64; M]) -> [f64; N] {
    let mut expansion = [0.0; M];
    let mut len = 0;
    for mut value in values {
        if value == 0.0 {
            continue;
        }
        let mut end = 0;
        for i in 0..len {
            let error;
            (value, error) = two_sum(value, expansion[i]);
            if error != 0.0 {
                expansion[end] = error;
                end += 1;
            }
        }
        expansion[end] = value;
        len = end + 1;
    }
    expansion[..len].reverse();
    expansion[len..].fill(0.0);
    renormalize(expansion)
}

fn float_exponent(value: f64) -> i32 {
    let bits = value.abs().to_bits();
    let biased = (bits >> 52) as i32;
    if biased == 0 {
        63 - bits.leading_zeros() as i32 - 1074
    } else {
        biased - 1023
    }
}

fn scale<const N: usize>(mut values: [f64; N], mut shift: i32) -> [f64; N] {
    while shift != 0 {
        let step = shift.clamp(-1022, 1023);
        let factor = f64::from_bits(((step + 1023) as u64) << 52);
        values = values.map(|x| x * factor);
        if !values[0].is_finite() || values[0] == 0.0 {
            return scalar(values[0]);
        }
        shift -= step;
    }
    values
}

fn parse<const N: usize>(text: &str) -> Result<[f64; N], ParseFloatError> {
    let first: f64 = text.parse()?;
    let mut result = [0.0; N];
    result[0] = first;
    if !first.is_finite() || first == 0.0 {
        return Ok(result);
    }
    // The primitive parser validates the grammar before expanding the exponent.
    let (mantissa, exponent) = text
        .split_once(['e', 'E'])
        .map_or((text, 0), |(m, e)| (m, e.parse::<i64>().unwrap()));
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.trim_start_matches(['+', '-']);
    let point = mantissa.find('.').unwrap_or(mantissa.len()) as i64 + exponent;
    let mut digits = mantissa.replace('.', "");
    if point <= 0 {
        digits = "0.".to_owned() + &"0".repeat((-point) as usize) + &digits;
    } else if point as usize >= digits.len() {
        digits.push_str(&"0".repeat(point as usize - digits.len()));
    } else {
        digits.insert(point as usize, '.');
    }
    if negative {
        digits.insert(0, '-');
    }
    let mut remainder: Decimal = digits.parse().unwrap();
    for component in &mut result {
        *component = remainder.to_string().parse()?;
        remainder -= Decimal::from(*component);
    }
    Ok(renormalize(result))
}

fn decimal(values: &[f64]) -> Decimal {
    values
        .iter()
        .fold(Decimal::zero(), |sum, &value| sum + Decimal::from(value))
}

fn format(values: &[f64], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if !values[0].is_finite() || values[0] == 0.0 {
        return std::fmt::Display::fmt(&values[0], f);
    }
    let value = decimal(values).to_string();
    let mut digits = value.trim_start_matches('-').to_owned();
    if let Some(precision) = f.precision() {
        let point = digits.find('.').unwrap_or(digits.len());
        if point < digits.len() {
            digits.remove(point);
        }
        let end = point + precision;
        if end < digits.len() {
            let bytes = digits.as_bytes();
            let increment = bytes[end] > b'5'
                || bytes[end] == b'5'
                    && (bytes[end + 1..].iter().any(|&x| x != b'0')
                        || !(bytes[end - 1] - b'0').is_multiple_of(2));
            digits.truncate(end);
            if increment {
                let mut bytes = digits.into_bytes();
                let mut carry = true;
                for digit in bytes.iter_mut().rev() {
                    if *digit != b'9' {
                        *digit += 1;
                        carry = false;
                        break;
                    }
                    *digit = b'0';
                }
                if carry {
                    bytes.insert(0, b'1');
                }
                digits = String::from_utf8(bytes).unwrap();
            }
        } else {
            digits.push_str(&"0".repeat(end - digits.len()));
        }
        if precision != 0 {
            digits.insert(digits.len() - precision, '.');
        }
    }
    f.pad_integral(!values[0].is_sign_negative(), "", &digits)
}

macro_rules! impl_float_expansion {
    ($name:ident, $n:literal, $($field:tt),+) => {
        impl $name {
            const fn components(self) -> [f64; $n] {
                [$(self.$field),+]
            }

            const fn from_components(values: [f64; $n]) -> Self {
                Self($(values[$field]),+)
            }

            fn scale(self, exponent: i32) -> Self {
                Self::from_components(scale(self.components(), exponent))
            }

            fn integral(self, ceil: bool) -> Self {
                if !self.0.is_finite() || self.0 == 0.0 { return self; }
                let mut values = self.components();
                for i in 0..$n {
                    let rounded = if ceil { values[i].ceil() } else { values[i].floor() };
                    if rounded != values[i] {
                        values[i] = rounded;
                        values[i + 1..].fill(0.0);
                        break;
                    }
                }
                let mut result = Self::from_components(renormalize(values));
                if result.is_zero() { result.0 = 0.0f64.copysign(self.0); }
                result
            }

        }

        impl From<f64> for $name {
            fn from(value: f64) -> Self {
                let mut values = [0.0; $n];
                values[0] = value;
                Self::from_components(values)
            }
        }

        impl From<$name> for f64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<$name> for i64 {
            fn from(value: $name) -> Self {
                if value.is_nan() {
                    return 0;
                }
                let value = value.trunc();
                if value >= $name::from(9223372036854775808.0) {
                    return i64::MAX;
                }
                if value <= $name::from(-9223372036854775808.0) {
                    return i64::MIN;
                }
                value.components().into_iter().map(|x| x as i128).sum::<i128>() as i64
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                format(&self.components(), f)
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::num::ParseFloatError;
            fn from_str(text: &str) -> Result<Self, Self::Err> {
                parse(text).map(Self::from_components)
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                for (a, b) in self.components().into_iter().zip(other.components()) {
                    match a.partial_cmp(&b)? {
                        Ordering::Equal => {}
                        order => return Some(order),
                    }
                }
                Some(Ordering::Equal)
            }
        }

        impl Bounded for $name {
            fn maximum() -> Self { Self::MAX }
            fn minimum() -> Self { Self::MIN }
        }

        impl Zero for $name {
            fn zero() -> Self { Self::from(0.0) }
            fn is_zero(&self) -> bool { self.0 == 0.0 }
        }

        impl One for $name {
            fn one() -> Self { Self::from(1.0) }
            fn is_one(&self) -> bool { *self == Self::one() }
        }

        impl Scan for $name {
            type Output = Self;
            fn scan<I: ScanSource>(iter: &mut I) -> Option<Self> {
                iter.next_token()?.parse().ok()
            }
        }

        impl Div for $name {
            type Output = Self;
            fn div(self, rhs: Self) -> Self {
                let mut quotient = [0.0; $n + 1];
                quotient[0] = self.0 / rhs.0;
                if !quotient[0].is_finite() || quotient[0] == 0.0 {
                    return Self::from(quotient[0]);
                }
                let mut remainder = self;
                for i in 0..$n {
                    remainder = remainder - rhs * quotient[i];
                    quotient[i + 1] = remainder.0 / rhs.0;
                }
                Self::from_components(renormalize(quotient))
            }
        }

        impl Rem for $name {
            type Output = Self;
            /// Large quotients can lose low remainder bits during reduction.
            fn rem(self, rhs: Self) -> Self {
                if !self.is_finite() || rhs.is_nan() || rhs.is_zero() {
                    return Self::NAN;
                }
                if rhs.is_infinite() || self.is_zero() || self.abs() < rhs.abs() {
                    return self;
                }
                let divisor = rhs.abs();
                let mut remainder = self.abs();
                while remainder.abs() >= divisor {
                    let mut quotient = (remainder / divisor).trunc();
                    if !quotient.is_finite() { return Self::from(self.0 % rhs.0); }
                    if quotient.is_zero() { quotient = Self::one().copysign(remainder); }
                    remainder = (-quotient).mul_add(divisor, remainder);
                }
                if remainder.0 < 0.0 { remainder = remainder + divisor; }
                remainder.copysign(self)
            }
        }

        impl Float for $name {
            type Bits = [u64; $n];

            fn floor(self) -> Self { self.integral(false) }
            fn ceil(self) -> Self { self.integral(true) }
            fn round(self) -> Self { if self.is_sign_negative() { (self + -0.5).ceil() } else { (self + 0.5).floor() } }
            fn trunc(self) -> Self { self.integral(self.is_sign_negative()) }
            fn fract(self) -> Self { self - self.trunc() }
            fn abs(self) -> Self { if self.0.is_sign_negative() { -self } else { self } }
            fn signum(self) -> Self { Self::from(self.0.signum()) }
            fn copysign(self, sign: Self) -> Self {
                if self.0.is_sign_negative() == sign.0.is_sign_negative() { self } else { -self }
            }

            /// Retains product components until after adding `b`.
            /// Intermediate products must remain in the normal f64 range.
            fn mul_add(self, a: Self, b: Self) -> Self {
                if !self.is_finite() || !a.is_finite() || !b.is_finite() {
                    return Self::from(self.0.mul_add(a.0, b.0));
                }
                let mut terms = [0.0; 2 * $n * $n + $n];
                let mut index = 0;
                for x in self.components() {
                    for y in a.components() {
                        let (product, error) = two_prod(x, y);
                        terms[index] = product;
                        terms[index + 1] = error;
                        index += 2;
                    }
                }
                terms[index..].copy_from_slice(&b.components());
                Self::from_components(sum_terms(terms))
            }

            fn div_euclid(self, rhs: Self) -> Self {
                let quotient = (self / rhs).trunc();
                if (self % rhs).0 < 0.0 {
                    quotient - Self::from(rhs.0.signum())
                } else {
                    quotient
                }
            }

            fn rem_euclid(self, rhs: Self) -> Self {
                let remainder = self % rhs;
                if remainder.0 < 0.0 { remainder + rhs.abs() } else { remainder }
            }

            /// Uses repeated squaring; rounding error can grow with `n.unsigned_abs()`.
            fn powi(self, n: i32) -> Self {
                if n == 0 { return Self::one(); }
                if n == 1 { return self; }
                if !self.is_finite() || self.is_zero() { return Self::from(self.0.powi(n)); }
                let mut factor = if n < 0 { self.recip() } else { self };
                let mut result = Self::one();
                let mut power = n.unsigned_abs();
                while power != 0 {
                    if power & 1 != 0 { result = result * factor; }
                    power >>= 1;
                    if power != 0 { factor = factor * factor; }
                }
                result
            }

            fn powf(self, n: Self) -> Self {
                if n.is_zero() || self.is_one() { return Self::one(); }
                if n.is_one() { return self; }
                if self.is_nan() || n.is_nan() { return Self::NAN; }
                if n.is_infinite() {
                    return match self.abs().partial_cmp(&Self::one()).unwrap() {
                        Ordering::Equal => Self::one(),
                        Ordering::Greater if n.is_sign_positive() => Self::INFINITY,
                        Ordering::Less if n.is_sign_negative() => Self::INFINITY,
                        _ => Self::zero(),
                    };
                }
                let integer = n == n.trunc();
                let odd = integer && !(n % Self::from(2.0)).is_zero();
                if self.is_zero() || self.is_infinite() {
                    let result = Self::from(self.0.abs().powf(n.0));
                    return if self.is_sign_negative() && odd { -result } else { result };
                }
                if self.is_sign_negative() && !integer { return Self::NAN; }
                let result = (self.abs().ln() * n).exp();
                if self.is_sign_negative() && odd { -result } else { result }
            }

            fn sqrt(self) -> Self {
                if self.is_zero() || !self.is_finite() || self.0 < 0.0 {
                    return Self::from(self.0.sqrt());
                }
                let exponent = float_exponent(self.0).div_euclid(2);
                let value = self.scale(-2 * exponent);
                let mut root = Self::from(value.0.sqrt());
                let half_reciprocal = 0.5 / root.0;
                for _ in 0..$n {
                    root = root + (value - root * root) * half_reciprocal;
                }
                root.scale(exponent)
            }

            fn exp(self) -> Self {
                if self.is_nan() { return self; }
                if self.0 > 710.0 { return Self::INFINITY; }
                if self.0 < -746.0 { return Self::zero(); }
                let exponent = (self.0 / std::f64::consts::LN_2).round() as i32;
                let mut terms = [0.0; $n + 12];
                terms[..$n].copy_from_slice(&self.components());
                for (i, value) in LN_2_REDUCTION.into_iter().enumerate() {
                    let (product, error) = two_prod(-(exponent as f64), value);
                    terms[$n + 2 * i] = product;
                    terms[$n + 2 * i + 1] = error;
                }
                let reduced = Self::from_components(sum_terms(terms)).scale(-8);
                let mut value = reduced.exp_m1();
                for _ in 0..8 { value = value * (value + 2.0); }
                (value + 1.0).scale(exponent)
            }

            fn exp2(self) -> Self {
                if self.is_nan() { return self; }
                if self.0 > 1024.0 { return Self::INFINITY; }
                if self.0 < -1075.0 { return Self::zero(); }
                let exponent = self.round().0 as i32;
                ((self - Self::from(exponent as f64)) * Self::LN_2).exp().scale(exponent)
            }

            fn ln(self) -> Self {
                if !self.is_finite() || self.0 <= 0.0 { return Self::from(self.0.ln()); }
                if self >= Self::from(0.5) && self <= Self::from(1.5) { return (self - Self::one()).ln_1p(); }
                let exponent = float_exponent(self.0);
                let value = self.scale(-exponent);
                let mut logarithm = Self::from(value.0.ln());
                for _ in 0..$n / 2 + 1 {
                    logarithm = logarithm + (value * (-logarithm).exp() - Self::one());
                }
                logarithm + Self::LN_2 * exponent as f64
            }

            fn log(self, base: Self) -> Self { self.ln() / base.ln() }
            fn log2(self) -> Self { self.ln() / Self::LN_2 }
            fn log10(self) -> Self { self.ln() / Self::LN_10 }

            fn cbrt(self) -> Self {
                if !self.is_finite() || self.is_zero() { return Self::from(self.0.cbrt()); }
                let exponent = float_exponent(self.0).div_euclid(3);
                let value = self.scale(-3 * exponent);
                let mut root = Self::from(value.0.cbrt());
                for _ in 0..$n / 2 + 1 {
                    root = (root * 2.0 + value / (root * root)) / Self::from(3.0);
                }
                root.scale(exponent)
            }

            fn hypot(self, other: Self) -> Self {
                if self.is_infinite() || other.is_infinite() { return Self::INFINITY; }
                if self.is_nan() || other.is_nan() { return Self::NAN; }
                if self.is_zero() { return other.abs(); }
                if other.is_zero() { return self.abs(); }
                let maximum = self.0.abs().max(other.0.abs());
                let exponent = float_exponent(maximum);
                let a = self.scale(-exponent);
                let b = other.scale(-exponent);
                (a * a + b * b).sqrt().scale(exponent)
            }

            fn sin(self) -> Self { self.sin_cos().0 }
            fn cos(self) -> Self { self.sin_cos().1 }
            fn tan(self) -> Self { let (sin, cos) = self.sin_cos(); sin / cos }

            fn asin(self) -> Self {
                if self.abs() > Self::one() { return Self::NAN; }
                self.atan2(((Self::one() - self.abs()) * (Self::one() + self.abs())).sqrt())
            }

            fn acos(self) -> Self {
                if self.abs() > Self::one() { return Self::NAN; }
                ((Self::one() - self.abs()) * (Self::one() + self.abs())).sqrt().atan2(self)
            }

            fn atan(self) -> Self {
                if self.is_nan() || self.is_zero() { return self; }
                if self.is_infinite() { return Self::FRAC_PI_2.copysign(self); }
                let reciprocal = self.abs() > Self::one();
                let value = if reciprocal { self.abs().recip() } else { self.abs() };
                let mut angle = Self::from(value.0.atan());
                for _ in 0..$n / 2 + 1 {
                    let (sin, cos) = angle.sin_cos();
                    angle = angle + (value * cos - sin) * cos;
                }
                if reciprocal { angle = Self::FRAC_PI_2 - angle; }
                angle.copysign(self)
            }

            fn atan2(self, other: Self) -> Self {
                if self.is_nan() || other.is_nan() { return Self::NAN; }
                let y = self.abs();
                let x = other.abs();
                let mut angle = if y.is_infinite() && x.is_infinite() {
                    Self::FRAC_PI_4
                } else if y.is_infinite() {
                    Self::FRAC_PI_2
                } else if x.is_infinite() || y.is_zero() {
                    Self::zero()
                } else if x.is_zero() {
                    Self::FRAC_PI_2
                } else if y > x {
                    Self::FRAC_PI_2 - (x / y).atan()
                } else {
                    (y / x).atan()
                };
                if other.is_sign_negative() { angle = Self::PI - angle; }
                angle.copysign(self)
            }

            /// Argument reduction uses expansion precision for `abs(self) < 2^52`.
            /// Larger arguments use the leading component's f64 approximation.
            fn sin_cos(self) -> (Self, Self) {
                if !self.is_finite() { return (Self::NAN, Self::NAN); }
                if self.is_zero() { return (self, Self::one()); }
                let (quadrant, reduced) = if self.abs() <= Self::FRAC_PI_4 {
                    (0, self)
                } else {
                    if self.0.abs() >= 4503599627370496.0 {
                        let (sin, cos) = self.0.sin_cos();
                        return (Self::from(sin), Self::from(cos));
                    }
                    let multiple = (self / Self::FRAC_PI_2).round().0;
                    let mut terms = [0.0; $n + 12];
                    terms[..$n].copy_from_slice(&self.components());
                    for (i, value) in PI_2_REDUCTION.into_iter().enumerate() {
                        let (product, error) = two_prod(-multiple, value);
                        terms[$n + 2 * i] = product;
                        terms[$n + 2 * i + 1] = error;
                    }
                    ((multiple as i64).rem_euclid(4), Self::from_components(sum_terms(terms)))
                };
                let square = -(reduced * reduced);
                let mut sin = reduced;
                let mut cos = Self::one();
                let mut sin_term = reduced;
                let mut cos_term = Self::one();
                let mut k = 2u32;
                loop {
                    sin_term = sin_term * square / Self::from((k * (k + 1)) as f64);
                    cos_term = cos_term * square / Self::from((k * (k - 1)) as f64);
                    sin = sin + sin_term;
                    cos = cos + cos_term;
                    if cos_term.0.abs() <= Self::EPSILON.0 * 0.015625 { break; }
                    k += 2;
                }
                match quadrant {
                    0 => (sin, cos),
                    1 => (cos, -sin),
                    2 => (-sin, -cos),
                    _ => (-cos, sin),
                }
            }

            fn exp_m1(self) -> Self {
                if self.is_zero() || self.is_nan() { return self; }
                if self.0.abs() > 0.5 { return self.exp() - Self::one(); }
                let mut term = self;
                let mut sum = self;
                let mut k = 2u32;
                loop {
                    term = term * self / Self::from(k as f64);
                    sum = sum + term;
                    if term.0.abs() <= sum.0.abs() * Self::EPSILON.0 * 0.015625 { break; }
                    k += 1;
                }
                sum
            }

            fn ln_1p(self) -> Self {
                if self.is_zero() || self.is_nan() { return self; }
                if self.0.abs() < Self::EPSILON.0 * 0.015625 { return self; }
                if self.0.abs() > 0.5 { return (self + 1.0).ln(); }
                let z = self / (self + 2.0);
                let square = z * z;
                let mut power = z;
                let mut sum = z;
                let mut k = 3u32;
                loop {
                    power = power * square;
                    let term = power / Self::from(k as f64);
                    sum = sum + term;
                    if term.0.abs() <= sum.0.abs() * Self::EPSILON.0 * 0.015625 { break; }
                    k += 2;
                }
                sum.scale(1)
            }

            fn sinh(self) -> Self {
                if self.0.abs() < Self::EPSILON.0 * 0.015625 { return self; }
                let value = self.abs();
                let result = if value.0 > 700.0 {
                    (value - Self::LN_2).exp()
                } else {
                    let t = value.exp_m1();
                    (t + t / (t + 1.0)).scale(-1)
                };
                result.copysign(self)
            }

            fn cosh(self) -> Self {
                let value = self.abs();
                if value.0 > 700.0 { return (value - Self::LN_2).exp(); }
                let t = value.exp();
                (t + t.recip()).scale(-1)
            }

            fn tanh(self) -> Self {
                let t = (self.abs() * -2.0).exp_m1();
                (-t / (t + 2.0)).copysign(self)
            }

            fn asinh(self) -> Self {
                let value = self.abs();
                let result = if value.0 > 1e150 {
                    value.ln() + Self::LN_2
                } else {
                    let square = value * value;
                    (value + square / ((square + 1.0).sqrt() + 1.0)).ln_1p()
                };
                result.copysign(self)
            }

            fn acosh(self) -> Self {
                if self < Self::one() { return Self::NAN; }
                if self.0 > 1e150 { return self.ln() + Self::LN_2; }
                let t = self - Self::one();
                (t + (t * (self + 1.0)).sqrt()).ln_1p()
            }

            fn atanh(self) -> Self {
                let value = self.abs();
                if value > Self::one() { return Self::NAN; }
                (value.ln_1p() - (-value).ln_1p()).scale(-1).copysign(self)
            }

            fn is_nan(self) -> bool { self.0.is_nan() }
            fn is_infinite(self) -> bool { self.0.is_infinite() }
            fn is_finite(self) -> bool { self.0.is_finite() }
            fn is_normal(self) -> bool { self.0.is_normal() }
            fn classify(self) -> std::num::FpCategory { self.0.classify() }
            fn is_sign_positive(self) -> bool { self.0.is_sign_positive() }
            fn is_sign_negative(self) -> bool { self.0.is_sign_negative() }
            fn recip(self) -> Self { Self::one() / self }
            fn to_degrees(self) -> Self { self * (Self::from(180.0) / Self::PI) }
            fn to_radians(self) -> Self { self * (Self::PI / Self::from(180.0)) }
            fn max(self, other: Self) -> Self { if self > other || other.is_nan() { self } else { other } }
            fn min(self, other: Self) -> Self { if self < other || other.is_nan() { self } else { other } }

            fn midpoint(self, other: Self) -> Self {
                if !self.is_finite() || !other.is_finite() { return Self::from(self.0 + other.0); }
                (self + other) * 0.5
            }

            fn to_bits(self) -> Self::Bits { self.components().map(f64::to_bits) }

            /// Components must be nonoverlapping and ordered by decreasing magnitude.
            fn from_bits(bits: Self::Bits) -> Self {
                Self::from_components(bits.map(f64::from_bits))
            }

            fn total_cmp(&self, other: &Self) -> Ordering {
                for (a, b) in self.components().into_iter().zip(other.components()) {
                    let order = a.total_cmp(&b);
                    if !order.is_eq() { return order; }
                }
                Ordering::Equal
            }

            fn clamp(self, min: Self, max: Self) -> Self {
                assert!(min <= max);
                if self < min { min } else if self > max { max } else { self }
            }

            const RADIX: u32 = 2;
            const MANTISSA_DIGITS: u32 = if $n == 2 { 104 } else { 209 };
            const DIGITS: u32 = if $n == 2 { 31 } else { 62 };
            /// Conservative nominal precision, not the gap to the successor of 1.
            const EPSILON: Self = Self::from_components(epsilon());
            const MIN: Self = Self::from_components(scalar(f64::MIN));
            const MAX: Self = Self::from_components(scalar(f64::MAX));
            const MIN_POSITIVE: Self = Self::from_components(scalar(f64::MIN_POSITIVE));
            const MIN_EXP: i32 = f64::MIN_EXP;
            const MAX_EXP: i32 = f64::MAX_EXP;
            const MIN_10_EXP: i32 = f64::MIN_10_EXP;
            const MAX_10_EXP: i32 = f64::MAX_10_EXP;
            const NAN: Self = Self::from_components(scalar(f64::NAN));
            const INFINITY: Self = Self::from_components(scalar(f64::INFINITY));
            const NEG_INFINITY: Self = Self::from_components(scalar(f64::NEG_INFINITY));
            const PI: Self = Self::from_components(constant(PI));
            const TAU: Self = Self::from_components(constant(TAU));
            const FRAC_PI_2: Self = Self::from_components(constant(FRAC_PI_2));
            const FRAC_PI_3: Self = Self::from_components(constant(FRAC_PI_3));
            const FRAC_PI_4: Self = Self::from_components(constant(FRAC_PI_4));
            const FRAC_PI_6: Self = Self::from_components(constant(FRAC_PI_6));
            const FRAC_PI_8: Self = Self::from_components(constant(FRAC_PI_8));
            const FRAC_1_PI: Self = Self::from_components(constant(FRAC_1_PI));
            const FRAC_2_PI: Self = Self::from_components(constant(FRAC_2_PI));
            const FRAC_2_SQRT_PI: Self = Self::from_components(constant(FRAC_2_SQRT_PI));
            const SQRT_2: Self = Self::from_components(constant(SQRT_2));
            const FRAC_1_SQRT_2: Self = Self::from_components(constant(FRAC_1_SQRT_2));
            const E: Self = Self::from_components(constant(E));
            const LOG2_E: Self = Self::from_components(constant(LOG2_E));
            const LOG10_E: Self = Self::from_components(constant(LOG10_E));
            const LN_2: Self = Self::from_components(constant(LN_2));
            const LN_10: Self = Self::from_components(constant(LN_10));
        }
    };
}

impl Add<f64> for DoubleDouble {
    type Output = Self;
    fn add(self, rhs: f64) -> Self::Output {
        let sum = self.0 + rhs;
        if !sum.is_finite() || self.0 == 0.0 && rhs == 0.0 {
            return Self::from(sum);
        }
        let (sum, error) = two_sum(self.0, rhs);
        let (high, low) = quick_two_sum(sum, error + self.1);
        Self(high, low)
    }
}

impl Add<DoubleDouble> for DoubleDouble {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let sum = self.0 + rhs.0;
        if !sum.is_finite() || self.0 == 0.0 && rhs.0 == 0.0 {
            return Self::from(sum);
        }
        let (sum, error) = two_sum(self.0, rhs.0);
        let (tail, tail_error) = two_sum(self.1, rhs.1);
        let (sum, error) = quick_two_sum(sum, error + tail);
        let (high, low) = quick_two_sum(sum, error + tail_error);
        Self(high, low)
    }
}

impl Sub for DoubleDouble {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Neg for DoubleDouble {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

impl Mul<f64> for DoubleDouble {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let product = self.0 * rhs;
        if !product.is_finite() || product == 0.0 {
            return Self::from(product);
        }
        let (product, error) = two_prod(self.0, rhs);
        let error = if cfg!(any(target_arch = "aarch64", target_feature = "fma")) {
            self.1.mul_add(rhs, error)
        } else {
            self.1 * rhs + error
        };
        let (high, low) = quick_two_sum(product, error);
        Self(high, low)
    }
}

impl Mul<DoubleDouble> for DoubleDouble {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let product = self.0 * rhs.0;
        if !product.is_finite() || product == 0.0 {
            return Self::from(product);
        }
        let (product, error) = two_prod(self.0, rhs.0);
        let error = if cfg!(any(target_arch = "aarch64", target_feature = "fma")) {
            self.0.mul_add(rhs.1, self.1.mul_add(rhs.0, error))
        } else {
            self.0 * rhs.1 + self.1 * rhs.0 + error
        };
        let (high, low) = quick_two_sum(product, error);
        Self(high, low)
    }
}

impl_float_expansion!(DoubleDouble, 2, 0, 1);

impl QuadDouble {
    fn renormalize(a0: f64, a1: f64, a2: f64, a3: f64, a4: f64) -> Self {
        Self::from_components(renormalize([a0, a1, a2, a3, a4]))
    }
}

fn three_three_sum(a: f64, b: f64, c: f64) -> (f64, f64, f64) {
    let (u, v) = two_sum(a, b);
    let (r0, w) = two_sum(u, c);
    let (r1, r2) = two_sum(v, w);
    (r0, r1, r2)
}

fn three_two_sum(a: f64, b: f64, c: f64) -> (f64, f64) {
    let (u, v) = two_sum(a, b);
    let (r0, w) = two_sum(u, c);
    let r1 = v + w;
    (r0, r1)
}

fn multiple_two_sum(xs: &[f64]) -> (f64, f64) {
    let (mut high, mut low) = (xs[0], 0.0);
    for &x in &xs[1..] {
        let (sum, error) = two_sum(high, x);
        high = sum;
        low += error;
    }
    (high, low)
}

impl Add<f64> for QuadDouble {
    type Output = Self;
    fn add(self, rhs: f64) -> Self::Output {
        let sum = self.0 + rhs;
        if !sum.is_finite() || self.0 == 0.0 && rhs == 0.0 {
            return Self::from(sum);
        }
        let (t0, e) = two_sum(self.0, rhs);
        let (t1, e) = two_sum(self.1, e);
        let (t2, e) = two_sum(self.2, e);
        let (t3, t4) = two_sum(self.3, e);
        Self::renormalize(t0, t1, t2, t3, t4)
    }
}

impl Add<QuadDouble> for QuadDouble {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let sum = self.0 + rhs.0;
        if !sum.is_finite() || self.0 == 0.0 && rhs.0 == 0.0 {
            return Self::from(sum);
        }
        let (s0, e0) = two_sum(self.0, rhs.0);
        let (s1, e1) = two_sum(self.1, rhs.1);
        let (s2, e2) = two_sum(self.2, rhs.2);
        let (s3, e3) = two_sum(self.3, rhs.3);
        let (s1, e0) = two_sum(s1, e0);
        let (s2, e0, e1) = three_three_sum(s2, e0, e1);
        let (s3, e0) = three_two_sum(s3, e0, e2);
        Self::renormalize(s0, s1, s2, s3, e0 + e1 + e3)
    }
}

impl Sub for QuadDouble {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Neg for QuadDouble {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2, -self.3)
    }
}

impl Mul<f64> for QuadDouble {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let product = self.0 * rhs;
        if !product.is_finite() || product == 0.0 {
            return Self::from(product);
        }
        let (t0, e0) = two_prod(self.0, rhs);
        let (p1, e1) = two_prod(self.1, rhs);
        let (p2, e2) = two_prod(self.2, rhs);
        let p3 = self.3 * rhs;

        let (t1, e4) = two_sum(p1, e0);
        let (t2, e5, e6) = three_three_sum(p2, e1, e4);
        let (t3, e7) = three_two_sum(p3, e2, e5);
        let t4 = e7 + e6;
        Self::renormalize(t0, t1, t2, t3, t4)
    }
}

impl Mul<QuadDouble> for QuadDouble {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let product = self.0 * rhs.0;
        if !product.is_finite() || product == 0.0 {
            return Self::from(product);
        }
        let (t0, q00) = two_prod(self.0, rhs.0);

        let (p01, q01) = two_prod(self.0, rhs.1);
        let (p10, q10) = two_prod(self.1, rhs.0);

        let (p02, q02) = two_prod(self.0, rhs.2);
        let (p11, q11) = two_prod(self.1, rhs.1);
        let (p20, q20) = two_prod(self.2, rhs.0);

        let p03 = self.0 * rhs.3;
        let p12 = self.1 * rhs.2;
        let p21 = self.2 * rhs.1;
        let p30 = self.3 * rhs.0;

        let (t1, e1, e2) = three_three_sum(q00, p01, p10);
        let (t2, e3) = multiple_two_sum(&[e1, q01, q10, p02, p11, p20]);
        let t3 = e2 + e3 + q02 + q11 + q20 + p03 + p12 + p21 + p30;
        Self::renormalize(t0, t1, t2, t3, 0.0)
    }
}

impl Index<usize> for QuadDouble {
    type Output = f64;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!(),
        }
    }
}

impl_float_expansion!(QuadDouble, 4, 0, 1, 2, 3);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::integer_boundary_values};

    fn assert_close(actual: &[f64], expected: Decimal, bits: i32) {
        let error: f64 = (decimal(actual) - &expected).to_string().parse().unwrap();
        let magnitude: f64 = expected.to_string().parse().unwrap();
        let tolerance = magnitude.abs() * 2f64.powi(-bits);
        assert!(
            error.abs() <= tolerance,
            "actual={actual:?}, expected={expected}, error={error}, tolerance={tolerance}"
        );
    }

    #[test]
    fn test_arithmetic() {
        let mut rng = Xorshift::default();
        let mask = (1u128 << 106) - 1;
        let boundaries: Vec<_> = integer_boundary_values!(u128)
            .into_iter()
            .filter(|&x| x <= mask)
            .collect();
        let cases = boundaries
            .iter()
            .copied()
            .flat_map(|a| [0, 1, a, mask - a, mask].map(|b| (a, b)))
            .chain((0..15000).map(|_| (rng.random(0..=mask), rng.random(0..=mask))));
        for (a, b) in cases {
            let mut product = Decimal::zero();
            let mut power = Decimal::from(a);
            let mut multiplier = b;
            while multiplier != 0 {
                if multiplier & 1 != 0 {
                    product += &power;
                }
                power = &power + &power;
                multiplier >>= 1;
            }
            macro_rules! check {
                ($t:ty, $bits:expr) => {{
                    let convert = |value: u128| {
                        <$t>::from((value >> 53) as f64) * 9007199254740992.0
                            + (value & ((1 << 53) - 1)) as f64
                    };
                    let x = convert(a);
                    let y = convert(b);
                    assert_eq!(decimal(&x.components()), Decimal::from(a));
                    assert_eq!(decimal(&y.components()), Decimal::from(b));
                    let sum = x + y;
                    let difference = x - y;
                    let multiplied = x * y;
                    assert_close(
                        &sum.components(),
                        Decimal::from(a) + Decimal::from(b),
                        $bits,
                    );
                    assert_close(
                        &difference.components(),
                        Decimal::from(a) - Decimal::from(b),
                        $bits,
                    );
                    assert_close(&multiplied.components(), product.clone(), $bits);
                    assert_close(&((-x) * y).components(), -product.clone(), $bits);
                }};
            }
            check!(DoubleDouble, 100);
            check!(QuadDouble, 200);
        }
        let q = QuadDouble::from(1.0) + 2f64.powi(-54) + 2f64.powi(-108) + 2f64.powi(-162);
        assert_eq!(
            q + q,
            QuadDouble(2.0, 2f64.powi(-53), 2f64.powi(-107), 2f64.powi(-161))
        );
        let a = QuadDouble(1.0, 2f64.powi(-53), -2f64.powi(-107), 0.0);
        let b = QuadDouble(-1.0 - f64::EPSILON, 2f64.powi(-53), -2f64.powi(-107), 0.0);
        let expected = QuadDouble::from(-2f64.powi(-106));
        assert_eq!(a + b, expected);
    }

    #[test]
    fn test_conversion() {
        let mut rng = Xorshift::default();
        macro_rules! check {
            ($t:ty) => {{
                let texts = [
                    "0",
                    "-0",
                    "+0.1",
                    "1e-300",
                    "-2.5E+10",
                    "1.",
                    ".5",
                    "NaN",
                    "inf",
                    "-inf",
                    "1e9999999999999999999999999",
                    "-1e-99999999999999999999999",
                ];
                for text in texts {
                    let value: $t = text.parse().unwrap();
                    let parsed: $t = value.to_string().parse().unwrap();
                    assert!(
                        value.is_nan() && parsed.is_nan() || value == parsed,
                        "{text}"
                    );
                    assert_eq!(value.is_sign_negative(), parsed.is_sign_negative());
                }
                assert_eq!(
                    "0.1".parse::<$t>().unwrap().components()[1],
                    -5.551115123125783e-18
                );
                for _ in 0..15000 {
                    let high = rng.random(-1_000_000..=1_000_000) as f64
                        * 10f64.powi(rng.random(-50..=50));
                    let low = high * (rng.random(-500_000..=500_000) as f64 * 1e-6) * f64::EPSILON;
                    let value = <$t>::from(high) + low;
                    let exact = Decimal::from(high) + Decimal::from(low);
                    assert_eq!(decimal(&value.components()), exact);
                    let parsed: $t = value.to_string().parse().unwrap();
                    assert_eq!(parsed, value);
                }
                for x in [
                    -9223372036854775808.0,
                    -10.5,
                    -0.5,
                    0.5,
                    10.5,
                    9223372036854775808.0,
                    f64::INFINITY,
                    f64::NAN,
                ] {
                    assert_eq!(i64::from(<$t>::from(x)), x as i64);
                }
                let max = <$t>::from(9223372036854775808.0) + -1.0;
                assert_eq!(i64::from(max), i64::MAX);
                for (text, expected) in [
                    ("0.125", "0.12"),
                    ("0.375", "0.38"),
                    ("-0.001", "-0.00"),
                    ("9.999", "10.00"),
                ] {
                    let value: $t = text.parse().unwrap();
                    assert_eq!(format!("{value:.2}"), expected);
                }
            }};
        }
        check!(DoubleDouble);
        check!(QuadDouble);
    }

    #[test]
    fn test_order_and_rounding() {
        macro_rules! check {
            ($t:ty) => {{
                let one = <$t>::one();
                let zero = <$t>::zero();
                let negative_zero = <$t>::from(-0.0);
                assert_eq!(zero, negative_zero);
                assert!(negative_zero.total_cmp(&zero).is_lt());
                assert_ne!(<$t>::NAN, <$t>::NAN);
                assert_eq!(<$t>::NAN.partial_cmp(&zero), None);
                assert!(!(one + 2f64.powi(-54)).is_one());
                for value in [
                    -3.,
                    -2.5,
                    -1.,
                    -0.5,
                    -0.,
                    0.,
                    0.5,
                    1.,
                    2.5,
                    3.,
                    2f64.powi(100),
                ] {
                    let x = <$t>::from(value);
                    assert_eq!(x.floor(), <$t>::from(value.floor()));
                    assert_eq!(x.ceil(), <$t>::from(value.ceil()));
                    assert_eq!(x.round(), <$t>::from(value.round()));
                    assert_eq!(x.trunc(), <$t>::from(value.trunc()));
                }
                let epsilon = 2f64.powi(-100);
                let below = one + -epsilon;
                let above = one + epsilon;
                assert!(below < one && one < above);
                assert_eq!(below.floor(), zero);
                assert_eq!(above.ceil(), <$t>::from(2.0));
                assert_eq!((<$t>::from(0.5) + -epsilon).round(), zero);
                assert_eq!((<$t>::from(0.5) + epsilon).round(), one);
                let large = <$t>::from(2f64.powi(100)) + 0.75;
                assert_eq!(large.floor(), <$t>::from(2f64.powi(100)));
                assert_eq!(large.ceil(), <$t>::from(2f64.powi(100)) + 1.0);
                assert_eq!(<$t>::NAN.min(one), one);
                assert_eq!(one.max(<$t>::NAN), one);
                assert!(<$t>::NAN.clamp(zero, one).is_nan());
                let a = <$t>::from_bits(<$t>::MAX.to_bits());
                assert_eq!(a, <$t>::maximum());
            }};
        }
        check!(DoubleDouble);
        check!(QuadDouble);
        let a = QuadDouble::from(1.0) + 2f64.powi(-54);
        let b = a + 2f64.powi(-108);
        assert!(a < b);
        assert!(a.total_cmp(&b).is_lt());
    }

    #[test]
    fn test_mul_add_and_remainder() {
        macro_rules! check {
            ($t:ty) => {{
                let a = <$t>::one() + 2f64.powi(-54);
                let c = -(<$t>::one() + 2f64.powi(-53));
                assert_eq!(a.mul_add(a, c), <$t>::from(2f64.powi(-108)));
                for (x, y) in [(7.0, 3.0), (-7.0, 3.0), (7.0, -3.0), (1000.5, 0.25)] {
                    let a = <$t>::from(x);
                    let b = <$t>::from(y);
                    assert_eq!(a % b, <$t>::from(x % y));
                    assert_eq!(a.rem_euclid(b), <$t>::from(x.rem_euclid(y)));
                    assert_eq!(a.div_euclid(b), <$t>::from(x.div_euclid(y)));
                }
            }};
        }
        check!(DoubleDouble);
        check!(QuadDouble);
    }

    #[test]
    fn test_special_values() {
        macro_rules! check {
            ($t:ty) => {{
                for scalar in [0.0, -0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
                    let x = <$t>::from(scalar);
                    let values = [
                        (x.sqrt(), scalar.sqrt()),
                        (x.cbrt(), scalar.cbrt()),
                        (x.ln(), scalar.ln()),
                        (x.ln_1p(), scalar.ln_1p()),
                        (x.exp(), scalar.exp()),
                        (x.exp_m1(), scalar.exp_m1()),
                        (x.sin(), scalar.sin()),
                        (x.cos(), scalar.cos()),
                        (x.atan(), scalar.atan()),
                        (x.sinh(), scalar.sinh()),
                        (x.cosh(), scalar.cosh()),
                        (x.tanh(), scalar.tanh()),
                        (x.asinh(), scalar.asinh()),
                        (x.acosh(), scalar.acosh()),
                        (x.atanh(), scalar.atanh()),
                        (x.signum(), scalar.signum()),
                    ];
                    for (actual, expected) in values {
                        if expected.is_nan() {
                            assert!(actual.is_nan());
                        } else {
                            assert_eq!(
                                f64::from(actual).to_bits(),
                                expected.to_bits(),
                                "input={scalar}, actual={actual:?}, expected={expected}"
                            );
                        }
                    }
                    assert_eq!(x.classify(), scalar.classify());
                    for other in [0.0, -0.0, f64::INFINITY, f64::NEG_INFINITY] {
                        let y = <$t>::from(other);
                        let actual = x.atan2(y);
                        let expected = scalar.atan2(other);
                        if expected.is_nan() {
                            assert!(actual.is_nan());
                        } else {
                            assert_eq!(f64::from(actual).to_bits(), expected.to_bits());
                        }
                        let expected = scalar.hypot(other);
                        if expected.is_nan() {
                            assert!(x.hypot(y).is_nan());
                        } else {
                            assert_eq!(f64::from(x.hypot(y)), expected);
                        }
                    }
                }
                assert!((<$t>::from(-0.0) + <$t>::from(-0.0)).is_sign_negative());
                assert_eq!(<$t>::from(-1.0).powf(<$t>::INFINITY), <$t>::one());
                assert_eq!(<$t>::NAN.powi(0), <$t>::one());
                assert!(<$t>::from(-2.0).powf(<$t>::from(0.5)).is_nan());
            }};
        }
        check!(DoubleDouble);
        check!(QuadDouble);
    }

    #[test]
    fn test_elementary_functions() {
        // Independent MPFR references at 4096 bits; inputs are exact f64 sums.
        let cases = [
            (
                "ln",
                [1.5, 1.1102230246251565e-16],
                [0.0, 0.0],
                "0.40546510810816445599288142380811575904706961211521472965837916178770288992",
            ),
            (
                "ln",
                [1.5, -1.1102230246251565e-16],
                [0.0, 0.0],
                "0.405465108108164307963144807120577035896180533338902750787946133803342602577",
            ),
            (
                "ln",
                [0.5, 2.7755575615628914e-17],
                [0.0, 0.0],
                "-0.693147180559945253906080890200351087637872239607865054922552110597171346276",
            ),
            (
                "ln",
                [0.5, -2.7755575615628914e-17],
                [0.0, 0.0],
                "-0.693147180559945364928383352716005130001039048690010342883516044278073342499",
            ),
            (
                "sqrt",
                [2.0, 0.0],
                [0.0, 0.0],
                "1.41421356237309504880168872420969807856967187537694807317667973799073247846",
            ),
            (
                "sqrt",
                [1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "1.00000000000000002775557561562891312540480282482334789236325185179674993391",
            ),
            (
                "cbrt",
                [-2.0, 0.0],
                [0.0, 0.0],
                "-1.25992104989487316476721060727822835057025146470150798008197511215529967651",
            ),
            (
                "cbrt",
                [1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "1.00000000000000001850371707708594199800631546600508633522993649727928056951",
            ),
            (
                "exp",
                [0.1, 0.0],
                [0.0, 0.0],
                "1.10517091807564763094663882345877965774166341637421962084148346131070925705",
            ),
            (
                "exp2",
                [0.1, 0.0],
                [0.0, 0.0],
                "1.07177346253629316833691172971933411119849523074102829709488841001834875705",
            ),
            (
                "exp_m1",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.105170918075647630946638823458779657741663416374219620841483461310709257053",
            ),
            (
                "exp_m1",
                [-2.0, 0.0],
                [0.0, 0.0],
                "-0.864664716763387308106000505027515596592368454090424118531841127345926625899",
            ),
            (
                "ln",
                [0.1, 0.0],
                [0.0, 0.0],
                "-2.30258509299404562850684022342653872716347359387638277683520000207135033398",
            ),
            (
                "ln",
                [1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "0.0000000000000000555111512312578254804376278947523901991981278988962222756935407097214122011",
            ),
            (
                "ln",
                [1.0, -5.551115123125783e-17],
                [0.0, 0.0],
                "-0.0000000000000000555111512312578285619255389143297550887628360347846797205290577537162612062",
            ),
            (
                "log2",
                [0.1, 0.0],
                [0.0, 0.0],
                "-3.32192809488736226778465683411645024904081376644015490756863155418292765322",
            ),
            (
                "log2",
                [1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "0.0000000000000000800856625953729399268240176265844257044861248416330071233859278076491277199",
            ),
            (
                "log10",
                [0.1, 0.0],
                [0.0, 0.0],
                "-0.999999999999999975891813336167822901132918478299868892754522651502856198248",
            ),
            (
                "log10",
                [1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "0.0000000000000000241081866638321770988670815217001311072454773484971438017519661784850987456",
            ),
            (
                "ln_1p",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.0953101798043248650904204170314766268673281912604003530179780404841729391453",
            ),
            (
                "ln_1p",
                [-1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "-37.4299477502370467085305345587415346760770072554537837225167205126432555864",
            ),
            (
                "sin",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.0998334166468281578301968678586166677359655721343701541459880671560001068719",
            ),
            (
                "sin",
                [1.5707963267948966, 6.123233995736766e-17],
                [0.0, 0.0],
                "0.999999999999999999999999999999999999999999999999999999999999999998878919223",
            ),
            (
                "cos",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.995004165278025765541375198862345256348080083515277267924625439962744963644",
            ),
            (
                "cos",
                [1.5707963267948966, 6.123233995736766e-17],
                [0.0, 0.0],
                "-0.00000000000000000000000000000000149738490485916977732079713393772509498666970184102790448307135784728530031",
            ),
            (
                "tan",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.10033467208545055066507850254801492712782258228855078633976653426574660632",
            ),
            (
                "tan",
                [1.5707963267948966, 6.123233995736766e-17],
                [0.0, 0.0],
                "-667830961000672557834948096545679.895621313886078988606234681001040404309719",
            ),
            (
                "asin",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.100167421161559801924603795051402009580336686524923363912069192009517915013",
            ),
            (
                "asin",
                [1.0, -5.551115123125783e-17],
                [0.0, 0.0],
                "1.5707963162581844915078136961554243985645573535703605188448909156220441271",
            ),
            (
                "asin",
                [1e-100, 0.0],
                [0.0, 0.0],
                "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000001999189980260288361964776078853415942018260300593659569926",
            ),
            (
                "acos",
                [0.1, 0.0],
                [0.0, 0.0],
                "1.47062890563333681730671789658834943251824801316262954657540310414439028813",
            ),
            (
                "acos",
                [1.0, -5.551115123125783e-17],
                [0.0, 0.0],
                "0.000000010536712127723507995484327043534027346117192391642581380531864076038933169",
            ),
            (
                "atan",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.0996686524911620328745997071312707873394963529317876001521936652794384428168",
            ),
            (
                "atan",
                [1.0, -5.551115123125783e-17],
                [0.0, 0.0],
                "0.785398163397448281860085230190961440086522892678913165607438622395913936803",
            ),
            (
                "atan",
                [1e-100, 0.0],
                [0.0, 0.0],
                "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000001999189980260288361964776078853415942018260300593659569926",
            ),
            (
                "sinh",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.100166750019844031402623219620922031216299381308421247410339067636109355245",
            ),
            (
                "cosh",
                [0.1, 0.0],
                [0.0, 0.0],
                "1.00500416805580359954401560383785762652536403506579837343114439367459990181",
            ),
            (
                "tanh",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.0996679946249558226142770436986248332553787540229117893161319719939332156229",
            ),
            ("tanh", [700.0, 0.0], [0.0, 0.0], "1.0"),
            (
                "asinh",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.0998340788992075688508691193520261448996666831864071090348946541451889472874",
            ),
            (
                "acosh",
                [1.0, 5.551115123125783e-17],
                [0.0, 0.0],
                "0.0000000105367121277235078980001569764860129317209081216314559121044204805521613352",
            ),
            (
                "acosh",
                [2.0, 0.0],
                [0.0, 0.0],
                "1.31695789692481670862504634730796844402698197146751647976847225692046018542",
            ),
            (
                "atanh",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.100335347731075586242913545116385112163132962064169493070123018829336923057",
            ),
            (
                "atanh",
                [1.0, -5.551115123125783e-17],
                [0.0, 0.0],
                "19.0615474653984959950960955322853986741878634050481767127846258796479903789",
            ),
            ("hypot", [3.0, 0.0], [4.0, 0.0], "5.0"),
            (
                "atan2",
                [1.0, 0.0],
                [-1.0, 0.0],
                "2.35619449019234492884698253745962716314787704953132936573120844423086230471",
            ),
            (
                "atan2",
                [-1.0, 0.0],
                [-2.0, 0.0],
                "-2.67794504458898712224838715181828848216863234508898555716401150358761854212",
            ),
            (
                "powf",
                [2.0, 0.0],
                [0.1, 0.0],
                "1.07177346253629316833691172971933411119849523074102829709488841001834875705",
            ),
            (
                "powf",
                [0.1, 0.0],
                [2.0, 0.0],
                "0.0100000000000000011102230246251565712385107782865939613956470813588370966096",
            ),
            (
                "powf",
                [1.0, 5.551115123125783e-17],
                [1000.0, 0.0],
                "1.00000000000005551115123125936622439313771185879675181305908468988334583258",
            ),
            (
                "powi",
                [2.0, 0.0],
                [-1074.0, 0.0],
                "0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000049406564584124654417656879286822137236505980261432476442558568250067550727",
            ),
            (
                "powi",
                [1.0, 5.551115123125783e-17],
                [10000.0, 0.0],
                "1.00000000000055511151231273232919992728631670577414246738570986197383794213",
            ),
            (
                "log",
                [0.1, 0.0],
                [3.0, 0.0],
                "-2.09590327428938455376814015611586582452331928310610291766393913108436981935",
            ),
            (
                "log",
                [1.0, 5.551115123125783e-17],
                [1.0, -5.551115123125783e-17],
                "-0.999999999999999944488848768742174519562372105247595546056751609394060623435",
            ),
            (
                "to_degrees",
                [0.1, 0.0],
                [0.0, 0.0],
                "5.72957795130823240573528362776218276349511714433549624045142118898388801514",
            ),
            (
                "to_radians",
                [0.1, 0.0],
                [0.0, 0.0],
                "0.00174532925199432967380892682428394325453289701612429517795251754279990364457",
            ),
        ];
        for (op, x, y, expected) in cases {
            macro_rules! check {
                ($t:ty,$bits:expr) => {{
                    let a = <$t>::from(x[0]) + x[1];
                    let b = <$t>::from(y[0]) + y[1];
                    let actual = match op {
                        "sqrt" => a.sqrt(),
                        "cbrt" => a.cbrt(),
                        "exp" => a.exp(),
                        "exp2" => a.exp2(),
                        "exp_m1" => a.exp_m1(),
                        "ln" => a.ln(),
                        "log2" => a.log2(),
                        "log10" => a.log10(),
                        "log" => a.log(b),
                        "ln_1p" => a.ln_1p(),
                        "sin" => a.sin(),
                        "cos" => a.cos(),
                        "tan" => a.tan(),
                        "asin" => a.asin(),
                        "acos" => a.acos(),
                        "atan" => a.atan(),
                        "atan2" => a.atan2(b),
                        "sinh" => a.sinh(),
                        "cosh" => a.cosh(),
                        "tanh" => a.tanh(),
                        "asinh" => a.asinh(),
                        "acosh" => a.acosh(),
                        "atanh" => a.atanh(),
                        "hypot" => a.hypot(b),
                        "powi" => a.powi(y[0] as i32),
                        "powf" => a.powf(b),
                        "to_degrees" => a.to_degrees(),
                        "to_radians" => a.to_radians(),
                        _ => unreachable!(),
                    };
                    assert!(actual.is_finite(), "{op}({a:?},{b:?})={actual:?}");
                    let bits = if op == "powi" {
                        $bits - y[0].abs().max(1.0).log2().ceil() as i32
                    } else {
                        $bits
                    };
                    assert_close(&actual.components(), expected.parse().unwrap(), bits);
                }};
            }
            check!(DoubleDouble, 100);
            check!(QuadDouble, 200);
        }
    }
}
