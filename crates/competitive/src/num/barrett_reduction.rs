use super::{One, Zero};
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct BarrettReduction<T> {
    m: T,
    im: T,
}

impl<T> BarrettReduction<T>
where
    T: Barrettable,
{
    pub fn new(m: T) -> Self {
        Self {
            m,
            im: T::inv_mod_approx(m),
        }
    }
    pub const fn new_with_im(m: T, im: T) -> Self {
        Self { m, im }
    }
    pub const fn get_mod(&self) -> T {
        self.m
    }
    pub fn div_rem(&self, a: T) -> (T, T) {
        T::barrett_reduce(a, self.m, self.im)
    }
    pub fn div(&self, a: T) -> T {
        self.div_rem(a).0
    }
    pub fn rem(&self, a: T) -> T {
        self.div_rem(a).1
    }
}

pub trait Barrettable:
    Sized
    + Copy
    + PartialOrd
    + Zero
    + One
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
{
    fn inv_mod_approx(m: Self) -> Self;
    fn div_approx(self, im: Self) -> Self;
    fn barrett_reduce(self, m: Self, im: Self) -> (Self, Self) {
        if m == Self::one() {
            return (self, Self::zero());
        }
        let q = self.div_approx(im);
        let r = self - q * m;
        if m <= r {
            (q + Self::one(), r - m)
        } else {
            (q, r)
        }
    }
}

impl Barrettable for u32 {
    fn inv_mod_approx(m: Self) -> Self {
        !0 / m
    }
    fn div_approx(self, im: Self) -> Self {
        ((self as u64 * im as u64) >> 32) as u32
    }
}

impl Barrettable for u64 {
    fn inv_mod_approx(m: Self) -> Self {
        !0 / m
    }
    fn div_approx(self, im: Self) -> Self {
        ((self as u128 * im as u128) >> 64) as u64
    }
}

impl Barrettable for u128 {
    fn inv_mod_approx(m: Self) -> Self {
        !0 / m
    }
    fn div_approx(self, im: Self) -> Self {
        const MASK64: u128 = 0xffff_ffff_ffff_ffff;
        let au = self >> 64;
        let ad = self & MASK64;
        let imu = im >> 64;
        let imd = im & MASK64;
        let mut res = au * imu;
        let x = (ad * imd) >> 64;
        let (x, c) = x.overflowing_add(au * imd);
        res += c as u128;
        let (x, c) = x.overflowing_add(ad * imu);
        res += c as u128;
        res + (x >> 64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    macro_rules! test_barrett {
        ($test_name:ident, $ty:ty, |$rng:ident| $res:expr) => {
            #[test]
            fn $test_name() {
                let mut $rng = Xorshift::default();
                const Q: usize = 10_000;
                for _ in 0..Q {
                    let (a, b): ($ty, $ty) = $res;
                    let barrett = BarrettReduction::<$ty>::new(b);
                    assert_eq!(a / b, barrett.div(a));
                    assert_eq!(a % b, barrett.rem(a));
                }
            }
        };
    }
    test_barrett!(test_barrett_u32_small, u32, |rng| (
        rng.random(..=100),
        rng.random(1..=100)
    ));
    test_barrett!(test_barrett_u64_small, u64, |rng| (
        rng.random(..=100),
        rng.random(1..=100)
    ));
    test_barrett!(test_barrett_u128_small, u128, |rng| {
        (
            rng.random(..=100u64) as u128 * rng.random(..=100u64) as u128,
            rng.random(1..=100u64) as u128 * rng.random(1..=100u64) as u128,
        )
    });

    test_barrett!(test_barrett_u32_large, u32, |rng| (
        rng.random(..=!0),
        rng.random(1..=!0)
    ));
    test_barrett!(test_barrett_u64_large, u64, |rng| (
        rng.random(..=!0),
        rng.random(1..=!0)
    ));
    test_barrett!(test_barrett_u128_large, u128, |rng| {
        (
            rng.random(..=!0u64) as u128 * rng.random(..=!0u64) as u128,
            rng.random(1..=!0u64) as u128 * rng.random(1..=!0u64) as u128,
        )
    });

    test_barrett!(test_barrett_u32_max, u32, |rng| (
        rng.random(!0 - 100..=!0),
        rng.random(!0 - 100..=!0)
    ));
    test_barrett!(test_barrett_u64_max, u64, |rng| (
        rng.random(!0 - 100..=!0),
        rng.random(!0 - 100..=!0)
    ));
    test_barrett!(test_barrett_u128_max, u128, |rng| {
        (
            rng.random(!0 - 100..=!0u64) as u128 * rng.random(!0 - 100..=!0u64) as u128,
            rng.random(!0 - 100..=!0u64) as u128 * rng.random(!0 - 100..=!0u64) as u128,
        )
    });

    test_barrett!(test_barrett_u128_mul, u128, |rng| {
        (
            rng.random(0u64..) as u128 * rng.random(0u64..) as u128,
            rng.random(0u64..) as u128,
        )
    });
}
