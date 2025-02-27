use super::{IterScan, One, Zero};
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
    use test_case::test_case;

    #[test_case("0", "0", Ordering::Equal; "zero")]
    #[test_case("0", "1", Ordering::Less; "zero vs plus")]
    #[test_case("0", "-1", Ordering::Greater; "zero vs minus")]
    #[test_case("1", "0", Ordering::Greater; "plus vs zero")]
    #[test_case("1", "1", Ordering::Equal; "plus vs plus")]
    #[test_case("1", "-1", Ordering::Greater; "plus vs minus")]
    #[test_case("-1", "0", Ordering::Less; "minus vs zero")]
    #[test_case("-1", "1", Ordering::Less; "minus vs plus")]
    #[test_case("-1", "-1", Ordering::Equal; "minus vs minus")]
    #[test_case("1000000000000000000", "1", Ordering::Greater; "long integer")]
    #[test_case("-1000000000000000000", "-1", Ordering::Less; "negative long integer")]
    #[test_case("0.1", "0.01", Ordering::Greater; "decimal")]
    #[test_case("0.1", "0.1", Ordering::Equal; "decimal equal")]
    #[test_case("0.1", "0.2", Ordering::Less; "decimal less")]
    #[test_case("0.1", "0.0000000000000000001", Ordering::Greater; "long decimal")]
    fn test_cmp(a: &str, b: &str, expected: Ordering) {
        let a = a.parse::<Decimal>().unwrap();
        let b = b.parse::<Decimal>().unwrap();
        assert_eq!(a.cmp(&b), expected);
    }

    #[test_case("0", "0"; "zero")]
    #[test_case("1", "-1"; "plus")]
    #[test_case("-1", "1"; "minus")]
    fn test_neg(a: &str, expected: &str) {
        let a = a.parse::<Decimal>().unwrap();
        let expected = expected.parse::<Decimal>().unwrap();
        assert_eq!(-a, expected);
    }
}
