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

impl Scan for Decimal {
    type Output = Self;
    fn scan<I: ScanSource>(iter: &mut I) -> Option<Self::Output> {
        iter.next_token()?.parse().ok()
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
