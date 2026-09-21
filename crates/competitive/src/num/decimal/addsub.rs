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
