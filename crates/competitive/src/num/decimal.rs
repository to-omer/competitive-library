use super::{One, Scan, ScanSource, Zero};
use std::{cmp::Ordering, ops::Neg};

pub mod addsub;
pub mod convert;

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug, Hash)]
enum Sign {
    Minus,
    Zero,
    Plus,
}

impl Neg for Sign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Sign::Minus => Sign::Plus,
            Sign::Zero => Sign::Zero,
            Sign::Plus => Sign::Minus,
        }
    }
}

const ZERO: Decimal = Decimal {
    sign: Sign::Zero,
    integer: Vec::new(),
    decimal: Vec::new(),
};

const POW10: [u64; RADIX_LEN + 1] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
];

const RADIX: u64 = POW10[RADIX_LEN];
const RADIX_LEN: usize = 18;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Decimal {
    sign: Sign,
    integer: Vec<u64>,
    decimal: Vec<u64>,
}

impl Default for Decimal {
    fn default() -> Self {
        Decimal::zero()
    }
}

impl Zero for Decimal {
    fn zero() -> Self {
        ZERO
    }

    fn is_zero(&self) -> bool {
        self.sign == Sign::Zero
    }
}

impl One for Decimal {
    fn one() -> Self {
        Decimal {
            sign: Sign::Plus,
            integer: vec![1],
            decimal: Vec::new(),
        }
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sign.cmp(&other.sign).then_with(|| match self.sign {
            Sign::Minus => other.cmp_absolute_parts(self),
            Sign::Zero => Ordering::Equal,
            Sign::Plus => self.cmp_absolute_parts(other),
        })
    }
}

impl Neg for Decimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            sign: -self.sign,
            integer: self.integer,
            decimal: self.decimal,
        }
    }
}

impl Decimal {
    fn cmp_absolute_parts(&self, other: &Self) -> Ordering {
        self.integer
            .len()
            .cmp(&other.integer.len())
            .then_with(|| self.integer.iter().rev().cmp(other.integer.iter().rev()))
            .then_with(|| self.decimal.iter().cmp(other.decimal.iter()))
    }
    fn normalize(&mut self) {
        if let Some(&0) = self.decimal.last() {
            let len = self
                .decimal
                .iter()
                .rposition(|&d| d != 0)
                .map_or(0, |i| i + 1);
            self.decimal.truncate(len);
        }
        if self.decimal.len() < self.decimal.capacity() / 4 {
            self.decimal.shrink_to_fit();
        }
        if let Some(&0) = self.integer.last() {
            let len = self
                .integer
                .iter()
                .rposition(|&d| d != 0)
                .map_or(0, |i| i + 1);
            self.integer.truncate(len);
        }
        if self.integer.len() < self.integer.capacity() / 4 {
            self.integer.shrink_to_fit();
        }
        if self.integer.is_empty() && self.decimal.is_empty() {
            self.sign = Sign::Zero;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use crate::tools::testutil::integer_boundary_values;

    #[test]
    fn test_decimal_arithmetic() {
        let mut rng = Xorshift::default();
        let mut cases: Vec<_> = (-10..=10)
            .flat_map(|a| (-10..=10).flat_map(move |b| (-4..=4).map(move |scale| (a, b, scale))))
            .collect();
        for _ in 0..10_000 {
            let digits = rng.random(0..=35u32);
            let bound = 10i128.pow(digits);
            let a = rng.random(-bound..=bound);
            let b = match rng.random(0..4) {
                0 => a,
                1 => -a,
                _ => rng.random(-bound..=bound),
            };
            let scale = rng.random(-80..=80i32);
            cases.push((a, b, scale));
        }
        for (a, b, scale) in cases {
            let format = |x: i128| {
                if x == 0 {
                    return "0".to_owned();
                }
                let mut s = x.abs().to_string();
                if scale > 0 {
                    let places = scale as usize;
                    if s.len() <= places {
                        s = "0".repeat(places + 1 - s.len()) + &s;
                    }
                    s.insert(s.len() - places, '.');
                    s = s.trim_end_matches('0').trim_end_matches('.').to_owned();
                } else {
                    s.push_str(&"0".repeat((-scale) as usize));
                }
                if x < 0 {
                    s.insert(0, '-');
                }
                s
            };
            let x: Decimal = format(a).parse().unwrap();
            let y: Decimal = format(b).parse().unwrap();
            assert_eq!(x.to_string(), format(a));
            assert_eq!(x.cmp(&y), a.cmp(&b));
            assert_eq!(x.partial_cmp(&y), Some(a.cmp(&b)));
            assert_eq!((-x.clone()).to_string(), format(-a));
            assert_eq!((x.clone() + y.clone()).to_string(), format(a + b));
            assert_eq!((&x + &y).to_string(), format(a + b));
            assert_eq!((x.clone() - y.clone()).to_string(), format(a - b));
            assert_eq!((&x - &y).to_string(), format(a - b));
            let mut z = x.clone();
            z += &y;
            assert_eq!(z.to_string(), format(a + b));
            z -= &y;
            assert_eq!(z, x);
            assert_eq!(x.is_zero(), a == 0);
        }
    }

    #[test]
    fn test_decimal_conversion() {
        let mut rng = Xorshift::default();
        macro_rules! check_integer {
            ($($t:ty),*) => {$(
                for x in integer_boundary_values!($t).into_iter().chain(rng.random_iter(..).take(1000)) {
                    let decimal = Decimal::from(x);
                    assert_eq!(decimal.to_string(), x.to_string());
                    assert_eq!(x.to_string().parse::<Decimal>().unwrap(), decimal);
                }
            )*};
        }
        check_integer!(
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        );
        for _ in 0..1000 {
            let x = rng.random(-1_000_000..=1_000_000) as f64 / 10f64.powi(rng.random(0..=12));
            assert_eq!(Decimal::from(x).to_string(), x.to_string());
            let x = x as f32;
            assert_eq!(Decimal::from(x).to_string(), x.to_string());
            let n = rng.random(0..=100);
            let digits: String = rng
                .random_iter(b'0'..=b'9')
                .take(n)
                .map(char::from)
                .collect();
            let sign = if rng.random(0..2) == 0 { "+" } else { "-" };
            let s = format!("{}00{}.{}00", sign, digits, digits);
            let decimal: Decimal = s.parse().unwrap();
            assert_eq!(decimal.to_string().parse::<Decimal>().unwrap(), decimal);
            let mut invalid = s;
            invalid.insert(
                rng.random(0..=invalid.len()),
                char::from(rng.random(b'a'..=b'z')),
            );
            assert!(invalid.parse::<Decimal>().is_err(), "{invalid}");
        }
    }
}
