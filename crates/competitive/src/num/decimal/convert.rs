use super::*;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDecimalError {
    kind: DecimalErrorKind,
}

impl ParseDecimalError {
    fn empty() -> Self {
        Self {
            kind: DecimalErrorKind::Empty,
        }
    }
    fn invalid_digit() -> Self {
        Self {
            kind: DecimalErrorKind::InvalidDigit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DecimalErrorKind {
    Empty,
    InvalidDigit,
}

impl Display for ParseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            DecimalErrorKind::Empty => write!(f, "empty string"),
            DecimalErrorKind::InvalidDigit => write!(f, "invalid digit"),
        }
    }
}

impl FromStr for Decimal {
    type Err = ParseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseDecimalError::empty());
        }

        let (s, sign) = if let Some(s) = s.strip_prefix('+') {
            (s, Sign::Plus)
        } else if let Some(s) = s.strip_prefix('-') {
            (s, Sign::Minus)
        } else {
            (s, Sign::Plus)
        };

        let (integer_str, decimal_str) = if let Some((integer_str, decimal_str)) = s.split_once('.')
        {
            (integer_str, decimal_str)
        } else {
            (s, "")
        };

        if !integer_str.is_ascii() || !decimal_str.is_ascii() {
            return Err(ParseDecimalError::invalid_digit());
        }

        let integer_bytes = integer_str.trim_start_matches('0').as_bytes();
        let decimal_bytes = decimal_str.trim_end_matches('0').as_bytes();

        let mut integer = Vec::with_capacity(integer_bytes.len().div_ceil(RADIX_LEN));
        for chunk in integer_bytes.rchunks(18) {
            let chunk = unsafe { std::str::from_utf8_unchecked(chunk) };
            match chunk.parse::<u64>() {
                Ok(val) => integer.push(val),
                Err(_) => return Err(ParseDecimalError::invalid_digit()),
            }
        }

        let mut decimal = Vec::with_capacity(decimal_bytes.len().div_ceil(RADIX_LEN));
        for chunk in decimal_bytes.chunks(18) {
            let chunk = unsafe { std::str::from_utf8_unchecked(chunk) };
            match chunk.parse::<u64>() {
                Ok(val) => decimal.push(val * POW10[RADIX_LEN - chunk.len()]),
                Err(_) => return Err(ParseDecimalError::invalid_digit()),
            }
        }

        let sign = if integer.is_empty() && decimal.is_empty() {
            Sign::Zero
        } else {
            sign
        };

        Ok(Decimal {
            sign,
            integer,
            decimal,
        })
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sign {
            Sign::Minus => write!(f, "-")?,
            Sign::Zero => return write!(f, "0"),
            Sign::Plus => {}
        }

        if let Some(last) = self.integer.last() {
            write!(f, "{}", last)?;
            for &val in self.integer.iter().rev().skip(1) {
                write!(f, "{:018}", val)?;
            }
        } else {
            write!(f, "0")?;
        }

        if let Some(last) = self.decimal.last() {
            write!(f, ".")?;
            for &val in self.decimal.iter().take(self.decimal.len() - 1) {
                write!(f, "{:018}", val)?;
            }
            let mut l = 0;
            let mut r = RADIX_LEN;
            while r - l > 1 {
                let m = l.midpoint(r);
                if last % POW10[m] == 0 {
                    l = m;
                } else {
                    r = m;
                }
            }
            debug_assert!(last % POW10[l] == 0);
            debug_assert!(r == RADIX_LEN || last % POW10[r] != 0);
            write!(f, "{:0width$}", last / POW10[l], width = RADIX_LEN - l)?;
        }

        Ok(())
    }
}

impl IterScan for Decimal {
    type Output = Self;
    fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<Self::Output> {
        iter.next()?.parse().ok()
    }
}

macro_rules! impl_from_unsigned {
    ($base:ty; $($t:ty)*) => {
        $(
            impl From<$t> for Decimal {
                fn from(val: $t) -> Self {
                    if val == 0 {
                        return Decimal::zero();
                    }
                    let mut val = val as $base;
                    let mut integer = Vec::new();
                    while val > 0 {
                        integer.push((val % RADIX as $base) as u64);
                        val /= RADIX as $base;
                    }
                    Decimal {
                        sign: Sign::Plus,
                        integer,
                        decimal: Vec::new(),
                    }
                }
            }
        )*
    };
}
impl_from_unsigned!(u64; u8 u16 u32 u64 usize);
impl_from_unsigned!(u128; u128);

macro_rules! impl_from_signed {
    ($base:ty; $($t:ty)*) => {
        $(
            impl From<$t> for Decimal {
                fn from(val: $t) -> Self {
                    let d = Decimal::from(val.unsigned_abs() as $base);
                    if val.is_negative() {
                        -d
                    } else {
                        d
                    }
                }
            }
        )*
    };
}
impl_from_signed!(u64; i8 i16 i32 i64 isize);
impl_from_signed!(u128; i128);

macro_rules! impl_from_through_string {
    ($($t:ty)*) => {
        $(
            impl From<$t> for Decimal {
                fn from(val: $t) -> Self {
                    val.to_string().parse().unwrap()
                }
            }
        )*
    };
}
impl_from_through_string!(f32 f64);

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        "0",
        Ok(Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] });
        "zero"
    )]
    #[test_case(
        "1",
        Ok(Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![] });
        "plus integer"
    )]
    #[test_case(
        "+1",
        Ok(Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![] });
        "plus integer with plus"
    )]
    #[test_case(
        "-1",
        Ok(Decimal { sign: Sign::Minus, integer: vec![1], decimal: vec![] });
        "minus integer"
    )]
    #[test_case(
        "1.2",
        Ok(Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![200000000000000000] });
        "plus decimal"
    )]
    #[test_case(
        "-1.2",
        Ok(Decimal { sign: Sign::Minus, integer: vec![1], decimal: vec![200000000000000000] });
        "minus decimal"
    )]
    #[test_case(
        "000000000000000000001.00000000000000000000",
        Ok(Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![] });
        "zero padding"
    )]
    #[test_case(
        ".1",
        Ok(Decimal { sign: Sign::Plus, integer: vec![], decimal: vec![100000000000000000] });
        "without integer"
    )]
    #[test_case(
        "12345678901234567890.12345678901234567890",
        Ok(Decimal { sign: Sign::Plus, integer: vec![345678901234567890, 12], decimal: vec![123456789012345678, 900000000000000000] });
        "long"
    )]
    #[test_case(
        "",
        Err(ParseDecimalError { kind: DecimalErrorKind::Empty });
        "empty"
    )]
    #[test_case(
        "a.012",
        Err(ParseDecimalError { kind: DecimalErrorKind::InvalidDigit });
        "invalid digit in integer"
    )]
    #[test_case(
        "012.a",
        Err(ParseDecimalError { kind: DecimalErrorKind::InvalidDigit });
        "invalid digit in decimal"
    )]
    fn test_from_str(s: &str, expected: Result<Decimal, ParseDecimalError>) {
        assert_eq!(expected, s.parse());
    }

    #[test_case(
        Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] },
        "0";
        "zero"
    )]
    #[test_case(
        Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![] },
        "1";
        "plus integer"
    )]
    #[test_case(
        Decimal { sign: Sign::Minus, integer: vec![1], decimal: vec![] },
        "-1";
        "minus integer"
    )]
    #[test_case(
        Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![200000000000000000] },
        "1.2";
        "plus decimal"
    )]
    #[test_case(
        Decimal { sign: Sign::Minus, integer: vec![1], decimal: vec![200000000000000000] },
        "-1.2";
        "minus decimal"
    )]
    #[test_case(
        Decimal { sign: Sign::Plus, integer: vec![], decimal: vec![100000000000000000] },
        "0.1";
        "without integer"
    )]
    #[test_case(
        Decimal { sign: Sign::Plus, integer: vec![345678901234567890, 12], decimal: vec![123456789012345678, 900000000000000000] },
        "12345678901234567890.1234567890123456789";
        "long"
    )]
    #[test_case(
        Decimal { sign: Sign::Plus, integer: vec![0], decimal: vec![1] },
        "0.000000000000000001";
        "small decimal"
    )]
    fn test_display(decimal: Decimal, expected: &str) {
        assert_eq!(expected, decimal.to_string());
    }

    #[test_case(u8::MIN, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "u8 zero")]
    #[test_case(u8::MAX, Decimal { sign: Sign::Plus, integer: vec![255], decimal: vec![] }; "u8 max")]
    #[test_case(u16::MIN, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "u16 zero")]
    #[test_case(u16::MAX, Decimal { sign: Sign::Plus, integer: vec![65535], decimal: vec![] }; "u16 max")]
    #[test_case(u32::MIN, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "u32 zero")]
    #[test_case(u32::MAX, Decimal { sign: Sign::Plus, integer: vec![4294967295], decimal: vec![] }; "u32 max")]
    #[test_case(u64::MIN, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "u64 zero")]
    #[test_case(u64::MAX, Decimal { sign: Sign::Plus, integer: vec![446744073709551615, 18], decimal: vec![] }; "u64 max")]
    #[test_case(u128::MIN, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "u128 zero")]
    #[test_case(u128::MAX, Decimal { sign: Sign::Plus, integer: vec![374607431768211455, 282366920938463463, 340], decimal: vec![] }; "u128 max")]
    #[test_case(i8::MIN, Decimal { sign: Sign::Minus, integer: vec![128], decimal: vec![] }; "i8 min")]
    #[test_case(0i8, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "i8 zero")]
    #[test_case(i8::MAX, Decimal { sign: Sign::Plus, integer: vec![127], decimal: vec![] }; "i8 max")]
    #[test_case(i16::MIN, Decimal { sign: Sign::Minus, integer: vec![32768], decimal: vec![] }; "i16 min")]
    #[test_case(0i16, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "i16 zero")]
    #[test_case(i16::MAX, Decimal { sign: Sign::Plus, integer: vec![32767], decimal: vec![] }; "i16 max")]
    #[test_case(i32::MIN, Decimal { sign: Sign::Minus, integer: vec![2147483648], decimal: vec![] }; "i32 min")]
    #[test_case(0i32, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "i32 zero")]
    #[test_case(i32::MAX, Decimal { sign: Sign::Plus, integer: vec![2147483647], decimal: vec![] }; "i32 max")]
    #[test_case(i64::MIN, Decimal { sign: Sign::Minus, integer: vec![223372036854775808, 9], decimal: vec![] }; "i64 min")]
    #[test_case(0i64, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "i64 zero")]
    #[test_case(i64::MAX, Decimal { sign: Sign::Plus, integer: vec![223372036854775807, 9], decimal: vec![] }; "i64 max")]
    #[test_case(i128::MIN, Decimal { sign: Sign::Minus, integer: vec![687303715884105728, 141183460469231731, 170], decimal: vec![] }; "i128 min")]
    #[test_case(0i128, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "i128 zero")]
    #[test_case(i128::MAX, Decimal { sign: Sign::Plus, integer: vec![687303715884105727, 141183460469231731, 170], decimal: vec![] }; "i128 max")]
    #[test_case(0f32, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "f32 zero")]
    #[test_case(1.1f32, Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![100000000000000000] }; "f32 plus")]
    #[test_case(-1.1f32, Decimal { sign: Sign::Minus, integer: vec![1], decimal: vec![100000000000000000] }; "f32 minus")]
    #[test_case(0f64, Decimal { sign: Sign::Zero, integer: vec![], decimal: vec![] }; "f64 zero")]
    #[test_case(1.1f64, Decimal { sign: Sign::Plus, integer: vec![1], decimal: vec![100000000000000000] }; "f64 plus")]
    #[test_case(-1.1f64, Decimal { sign: Sign::Minus, integer: vec![1], decimal: vec![100000000000000000] }; "f64 minus")]
    fn test_from(val: impl Into<Decimal>, expected: Decimal) {
        assert_eq!(expected, val.into());
    }
}
