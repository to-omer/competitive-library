use super::*;
use std::{
    mem::replace,
    ops::{Add, AddAssign, Sub, SubAssign},
};

fn add_carry(carry: bool, lhs: u64, rhs: u64, out: &mut u64) -> bool {
    let mut sum = lhs + rhs + carry as u64;
    let cond = sum >= RADIX;
    if cond {
        sum -= RADIX;
    }
    *out = sum;
    cond
}

fn add_absolute_parts(lhs: &mut Decimal, rhs: &Decimal) {
    let mut carry = false;

    // decimal part
    let lhs_decimal_len = lhs.decimal.len();
    if lhs_decimal_len < rhs.decimal.len() {
        for (l, r) in lhs
            .decimal
            .iter_mut()
            .rev()
            .zip(rhs.decimal[..lhs_decimal_len].iter().rev())
        {
            carry = add_carry(carry, *l, *r, l);
        }
        lhs.decimal
            .extend_from_slice(&rhs.decimal[lhs_decimal_len..]);
    } else {
        for (l, r) in lhs.decimal[..rhs.decimal.len()]
            .iter_mut()
            .rev()
            .zip(rhs.decimal.iter().rev())
        {
            carry = add_carry(carry, *l, *r, l);
        }
    }

    // integer part
    let lhs_integer_len = lhs.integer.len();
    if lhs_integer_len < rhs.integer.len() {
        for (l, r) in lhs.integer.iter_mut().zip(&rhs.integer[..lhs_integer_len]) {
            carry = add_carry(carry, *l, *r, l);
        }
        lhs.integer
            .extend_from_slice(&rhs.integer[lhs_integer_len..]);
        if carry {
            for l in lhs.integer[lhs_integer_len..].iter_mut() {
                carry = add_carry(carry, *l, 0, l);
                if !carry {
                    break;
                }
            }
        }
    } else {
        for (l, r) in lhs.integer.iter_mut().zip(&rhs.integer) {
            carry = add_carry(carry, *l, *r, l);
        }
        if carry {
            for l in lhs.integer[rhs.integer.len()..].iter_mut() {
                carry = add_carry(carry, *l, 0, l);
                if !carry {
                    break;
                }
            }
        }
    }

    if carry {
        lhs.integer.push(carry as u64);
    }

    lhs.normalize();
}

fn sub_borrow(borrow: bool, lhs: u64, rhs: u64, out: &mut u64) -> bool {
    let (sum, borrow1) = lhs.overflowing_sub(rhs);
    let (mut sum, borrow2) = sum.overflowing_sub(borrow as u64);
    let borrow = borrow1 || borrow2;
    if borrow {
        sum = sum.wrapping_add(RADIX);
    }
    *out = sum;
    borrow
}

// assume |lhs| >= |rhs|
fn sub_absolute_parts_gte(lhs: &Decimal, rhs: &mut Decimal) {
    debug_assert!(matches!(lhs.cmp_absolute_parts(rhs), Ordering::Greater));

    let mut borrow = false;

    // decimal part
    let rhs_decimal_len = rhs.decimal.len();
    if lhs.decimal.len() > rhs_decimal_len {
        for (l, r) in lhs.decimal[..rhs_decimal_len]
            .iter()
            .rev()
            .zip(rhs.decimal.iter_mut().rev())
        {
            borrow = sub_borrow(borrow, *l, *r, r);
        }
        rhs.decimal
            .extend_from_slice(&lhs.decimal[rhs_decimal_len..]);
    } else {
        for r in rhs.decimal[lhs.decimal.len()..].iter_mut().rev() {
            borrow = sub_borrow(borrow, 0, *r, r);
        }
        for (l, r) in lhs
            .decimal
            .iter()
            .rev()
            .zip(rhs.decimal[..lhs.decimal.len()].iter_mut().rev())
        {
            borrow = sub_borrow(borrow, *l, *r, r);
        }
    }

    // integer part
    let rhs_integer_len = rhs.integer.len();
    if lhs.integer.len() > rhs_integer_len {
        for (l, r) in lhs.integer[..rhs_integer_len]
            .iter()
            .zip(rhs.integer.iter_mut())
        {
            borrow = sub_borrow(borrow, *l, *r, r);
        }
        rhs.integer
            .extend_from_slice(&lhs.integer[rhs_integer_len..]);
        if borrow {
            for r in rhs.integer[rhs_integer_len..].iter_mut() {
                borrow = sub_borrow(borrow, *r, 0, r);
                if !borrow {
                    break;
                }
            }
        }
    } else {
        debug_assert_eq!(lhs.integer.len(), rhs_integer_len);
        for (l, r) in lhs.integer.iter().zip(&mut rhs.integer) {
            borrow = sub_borrow(borrow, *l, *r, r);
        }
    }

    assert!(
        !borrow,
        "Cannot subtract lhs from rhs because lhs is smaller than rhs"
    );

    rhs.normalize();
}

macro_rules! add {
    ($lhs:expr, $lhs_owned:expr, $rhs:expr, $rhs_owned:expr) => {
        match ($lhs.sign, $rhs.sign) {
            (Sign::Zero, _) => $rhs_owned,
            (_, Sign::Zero) => $lhs_owned,
            (Sign::Plus, Sign::Plus) | (Sign::Minus, Sign::Minus) => {
                let mut lhs = $lhs_owned;
                add_absolute_parts(&mut lhs, &$rhs);
                lhs
            }
            (Sign::Plus, Sign::Minus) | (Sign::Minus, Sign::Plus) => {
                match $lhs.cmp_absolute_parts(&$rhs) {
                    Ordering::Less => {
                        let mut lhs = $lhs_owned;
                        sub_absolute_parts_gte(&$rhs, &mut lhs);
                        lhs.sign = $rhs.sign;
                        lhs
                    }
                    Ordering::Equal => ZERO,
                    Ordering::Greater => {
                        let mut rhs = $rhs_owned;
                        sub_absolute_parts_gte(&$lhs, &mut rhs);
                        rhs.sign = $lhs.sign;
                        rhs
                    }
                }
            }
        }
    };
}

macro_rules! sub {
    ($lhs:expr, $lhs_owned:expr, $rhs:expr, $rhs_owned:expr) => {
        match ($lhs.sign, $rhs.sign) {
            (Sign::Zero, _) => -$rhs_owned,
            (_, Sign::Zero) => $lhs_owned,
            (Sign::Plus, Sign::Minus) | (Sign::Minus, Sign::Plus) => {
                let mut lhs = $lhs_owned;
                add_absolute_parts(&mut lhs, &$rhs);
                lhs
            }
            (Sign::Plus, Sign::Plus) | (Sign::Minus, Sign::Minus) => {
                match $lhs.cmp_absolute_parts(&$rhs) {
                    Ordering::Less => {
                        let mut lhs = $lhs_owned;
                        sub_absolute_parts_gte(&$rhs, &mut lhs);
                        lhs.sign = -$rhs.sign;
                        lhs
                    }
                    Ordering::Equal => ZERO,
                    Ordering::Greater => {
                        let mut rhs = $rhs_owned;
                        sub_absolute_parts_gte(&$lhs, &mut rhs);
                        rhs
                    }
                }
            }
        }
    };
}

macro_rules! impl_binop {
    (impl $Trait:ident for Decimal, $method:ident, $macro:ident) => {
        impl $Trait<Decimal> for Decimal {
            type Output = Decimal;

            fn $method(self, rhs: Decimal) -> Self::Output {
                $macro!(self, self, rhs, rhs)
            }
        }

        impl $Trait<&Decimal> for Decimal {
            type Output = Decimal;

            fn $method(self, rhs: &Decimal) -> Self::Output {
                $macro!(self, self, rhs, rhs.clone())
            }
        }

        impl $Trait<Decimal> for &Decimal {
            type Output = Decimal;

            fn $method(self, rhs: Decimal) -> Self::Output {
                $macro!(self, self.clone(), rhs, rhs)
            }
        }

        impl $Trait<&Decimal> for &Decimal {
            type Output = Decimal;

            fn $method(self, rhs: &Decimal) -> Self::Output {
                $macro!(self, self.clone(), rhs, rhs.clone())
            }
        }
    };
}
impl_binop!(impl Add for Decimal, add, add);
impl_binop!(impl Sub for Decimal, sub, sub);

macro_rules! impl_binop_assign {
    (impl $Trait:ident for Decimal, $method:ident, $op:tt) => {
        impl $Trait for Decimal {
            fn $method(&mut self, rhs: Decimal) {
                let lhs = replace(self, ZERO);
                *self = lhs $op rhs;
            }
        }

        impl $Trait<&Decimal> for Decimal {
            fn $method(&mut self, rhs: &Decimal) {
                let lhs = replace(self, ZERO);
                *self = lhs $op rhs;
            }
        }
    };
}

impl_binop_assign!(impl AddAssign for Decimal, add_assign, +);
impl_binop_assign!(impl SubAssign for Decimal, sub_assign, -);

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("0", "0", "0"; "zero")]
    #[test_case("0", "1", "1"; "zero vs plus")]
    #[test_case("0", "-1", "-1"; "zero vs minus")]
    #[test_case("1", "0", "1"; "plus vs zero")]
    #[test_case("-1", "0", "-1"; "minus vs zero")]
    #[test_case("1", "1", "2"; "plus vs plus")]
    #[test_case("1", "-1", "0"; "plus vs minus results zero")]
    #[test_case("2", "-1", "1"; "plus vs minus results plus")]
    #[test_case("1", "-2", "-1"; "plus vs minus results minus")]
    #[test_case("-1", "1", "0"; "minus vs plus results zero")]
    #[test_case("-2", "1", "-1"; "minus vs plus results minus")]
    #[test_case("-1", "2", "1"; "minus vs plus results plus")]
    #[test_case("-1", "-1", "-2"; "minus vs minus")]
    #[test_case(
        "999999999999999999.999999999999999999",
        "000000000000000000.000000000000000001",
        "1000000000000000000";
        "carry"
    )]
    #[test_case(
        "012345678901234567890.1234567890123456789",
        "098765432109876543210.9876543210987654321",
        "111111111011111111101.1111111101111111110";
        "plus long vs plus long"
    )]
    #[test_case(
        "00345678901234567890.1234567890",
        "98765432109876543210.9876543210987654321",
        "99111111011111111101.1111111100987654321";
        "plus short vs plus long"
    )]
    #[test_case(
        "12345678901234567890.1234567890123456789",
        "00765432109876543210.9876543210",
        "13111111011111111101.1111111100123456789";
        "plus long vs plus short"
    )]
    #[test_case(
        "+1000000000000000000.0000000000000000000",
        "-0000000000000000000.0000000000000000001",
        "+0999999999999999999.9999999999999999999";
        "borrow"
    )]
    #[test_case(
        "+098765432109876543210.9876543210987654321",
        "-012345678901234567890.1234567890123456789",
        "+086419753208641975320.8641975320864197532";
        "plus long vs minus long results plus"
    )]
    #[test_case(
        "+012345678901234567890.1234567890123456789",
        "-098765432109876543210.9876543210987654321",
        "-086419753208641975320.8641975320864197532";
        "plus long vs minus long results minus"
    )]
    #[test_case(
        "-098765432109876543210.9876543210987654321",
        "+012345678901234567890.1234567890123456789",
        "-086419753208641975320.8641975320864197532";
        "minus long vs plus long results minus"
    )]
    #[test_case(
        "-012345678901234567890.1234567890123456789",
        "+098765432109876543210.9876543210987654321",
        "+086419753208641975320.8641975320864197532";
        "minus long vs plus long results plus"
    )]
    #[test_case(
        "+098765432109876543210.9876543210987654321",
        "-000945678901234567890.123456789",
        "+097819753208641975320.8641975320987654321";
        "plus long vs minus short results plus"
    )]
    fn test_add(lhs: &str, rhs: &str, expected: &str) {
        let lhs: Decimal = lhs.parse().unwrap();
        let rhs: Decimal = rhs.parse().unwrap();
        let expected: Decimal = expected.parse().unwrap();
        assert_eq!(lhs.clone() + rhs.clone(), expected);
        assert_eq!(lhs.clone() + &rhs, expected);
        assert_eq!(&lhs + rhs.clone(), expected);
        assert_eq!(&lhs + &rhs, expected);
    }

    #[test_case("0", "0", "0"; "zero")]
    #[test_case("0", "1", "-1"; "zero vs plus")]
    #[test_case("0", "-1", "1"; "zero vs minus")]
    #[test_case("1", "0", "1"; "plus vs zero")]
    #[test_case("-1", "0", "-1"; "minus vs zero")]
    #[test_case("1", "-1", "2"; "plus vs minus")]
    #[test_case("1", "1", "0"; "plus vs plus results zero")]
    #[test_case("2", "1", "1"; "plus vs plus results plus")]
    #[test_case("1", "2", "-1"; "plus vs plus results minus")]
    #[test_case("-1", "-1", "0"; "minus vs minus results zero")]
    #[test_case("-2", "-1", "-1"; "minus vs minus results minus")]
    #[test_case("-1", "-2", "1"; "minus vs minus results plus")]
    #[test_case("-1", "1", "-2"; "minus vs plus")]
    #[test_case(
        "+999999999999999999.999999999999999999",
        "-000000000000000000.000000000000000001",
        "+1000000000000000000";
        "carry"
    )]
    #[test_case(
        "+012345678901234567890.1234567890123456789",
        "-098765432109876543210.9876543210987654321",
        "+111111111011111111101.1111111101111111110";
        "plus long vs minus long"
    )]
    #[test_case(
        "+00345678901234567890.1234567890",
        "-98765432109876543210.9876543210987654321",
        "+99111111011111111101.1111111100987654321";
        "plus short vs minus long"
    )]
    #[test_case(
        "+12345678901234567890.1234567890123456789",
        "-00765432109876543210.9876543210",
        "+13111111011111111101.1111111100123456789";
        "plus long vs minus short"
    )]
    #[test_case(
        "+1000000000000000000.0000000000000000000",
        "+0000000000000000000.0000000000000000001",
        "+0999999999999999999.9999999999999999999";
        "borrow"
    )]
    #[test_case(
        "+098765432109876543210.9876543210987654321",
        "+012345678901234567890.1234567890123456789",
        "+086419753208641975320.8641975320864197532";
        "plus long vs plus long results plus"
    )]
    #[test_case(
        "+012345678901234567890.1234567890123456789",
        "+098765432109876543210.9876543210987654321",
        "-086419753208641975320.8641975320864197532";
        "plus long vs plus long results minus"
    )]
    #[test_case(
        "-098765432109876543210.9876543210987654321",
        "-012345678901234567890.1234567890123456789",
        "-086419753208641975320.8641975320864197532";
        "minus long vs minus long results minus"
    )]
    #[test_case(
        "-012345678901234567890.1234567890123456789",
        "-098765432109876543210.9876543210987654321",
        "+086419753208641975320.8641975320864197532";
        "minus long vs minus long results plus"
    )]
    #[test_case(
        "+098765432109876543210.9876543210987654321",
        "+000945678901234567890.123456789",
        "+097819753208641975320.8641975320987654321";
        "plus long vs plus short results plus"
    )]
    fn test_sub(lhs: &str, rhs: &str, expected: &str) {
        let lhs: Decimal = lhs.parse().unwrap();
        let rhs: Decimal = rhs.parse().unwrap();
        let expected: Decimal = expected.parse().unwrap();
        assert_eq!(lhs.clone() - rhs.clone(), expected);
        assert_eq!(lhs.clone() - &rhs, expected);
        assert_eq!(&lhs - rhs.clone(), expected);
        assert_eq!(&lhs - &rhs, expected);
    }
}
