use super::*;
use std::{cell::UnsafeCell, mem::swap};

#[macro_export]
macro_rules! define_basic_mintbase {
    ($name:ident, $m:expr, $basety:ty, $signedty:ty, $upperty:ty, [$($unsigned:ty),*], [$($signed:ty),*]) => {
        pub enum $name {}
        impl MIntBase for $name {
            type Inner = $basety;
            #[inline]
            fn get_mod() -> Self::Inner {
                $m
            }
            #[inline]
            fn mod_zero() -> Self::Inner {
                0
            }
            #[inline]
            fn mod_one() -> Self::Inner {
                (Self::get_mod() != 1) as $basety
            }
            #[inline]
            fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                let m = Self::get_mod();
                let (z, borrow) = x.overflowing_sub(m - y);
                if borrow { z.wrapping_add(m) } else { z }
            }
            #[inline]
            fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                if x < y {
                    Self::get_mod() - (y - x)
                } else {
                    x - y
                }
            }
            #[inline]
            fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                // (x as $upperty * y as $upperty % Self::get_mod() as $upperty) as $basety
                $name::rem(x as $upperty * y as $upperty) as $basety
            }



            #[inline]
            fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                Self::mod_mul(x, Self::mod_inv(y))
            }
            #[inline]
            fn mod_neg(x: Self::Inner) -> Self::Inner {
                if x == 0 {
                    0
                } else {
                    Self::get_mod() - x
                }
            }
            fn mod_inv(x: Self::Inner) -> Self::Inner {
                let (mut a, mut b) = (x, Self::get_mod());
                let (mut u, mut v) = (1, 0);
                let mut negative = true;
                // b * u + a * v equals the modulus, so coefficient updates fit.
                while a != 0 {
                    let k = b / a;
                    v += k * u;
                    b -= k * a;
                    swap(&mut u, &mut v);
                    swap(&mut b, &mut a);
                    negative = !negative;
                }
                if negative { Self::mod_neg(v) } else { v }
            }
        }

        $(impl MIntConvert<$unsigned> for $name {
            #[inline]
            fn from(x: $unsigned) -> Self::Inner {
                (x % <Self as MIntBase>::get_mod() as $unsigned) as $basety
            }
            #[inline]
            fn into(x: Self::Inner) -> $unsigned {
                x as $unsigned
            }
            #[inline]
            fn mod_into() -> $unsigned {
                <Self as MIntBase>::get_mod() as $unsigned
            }
        })*
        $(impl MIntConvert<$signed> for $name {
            #[inline]
            fn from(x: $signed) -> Self::Inner {
                let modulus = (<Self as MIntBase>::get_mod() as $signed).cast_unsigned();
                let value = (x.unsigned_abs() % modulus) as $basety;
                if x < 0 {
                    <Self as MIntBase>::mod_neg(value)
                } else {
                    value
                }
            }
            #[inline]
            fn into(x: Self::Inner) -> $signed {
                x as $signed
            }
            #[inline]
            fn mod_into() -> $signed {
                <Self as MIntBase>::get_mod() as $signed
            }
        })*
    };
}

#[macro_export]
macro_rules! define_basic_mint32 {
    ($([$name:ident, $m:expr, $mint_name:ident]),*) => {
        $(define_basic_mintbase!(
            $name,
            $m,
            u32,
            i32,
            u64,
            [u32, u64, u128, usize],
            [i32, i64, i128, isize]
        );
        impl $name {
            fn rem(x: u64) -> u64 {
                x % $m
            }
        }
        pub type $mint_name = MInt<$name>;)*
    };
}

thread_local!(static DYN_MODULUS_U32: UnsafeCell<BarrettReduction<u64>> = const { UnsafeCell::new(BarrettReduction::<u64>::new_with_im(1_000_000_007, !0 / 1_000_000_007)) });
impl DynModuloU32 {
    pub fn set_mod(m: u32) {
        DYN_MODULUS_U32
            .with(|cell| unsafe { *cell.get() = BarrettReduction::<u64>::new(m as u64) });
    }
    fn rem(x: u64) -> u64 {
        DYN_MODULUS_U32.with(|cell| unsafe { (*cell.get()).rem(x) })
    }
}
impl DynMIntU32 {
    pub fn set_mod(m: u32) {
        DynModuloU32::set_mod(m)
    }
}

thread_local!(static DYN_MODULUS_U64: UnsafeCell<BarrettReduction<u128>> = const { UnsafeCell::new(BarrettReduction::<u128>::new_with_im(1_000_000_007, !0 / 1_000_000_007)) });
impl DynModuloU64 {
    pub fn set_mod(m: u64) {
        DYN_MODULUS_U64
            .with(|cell| unsafe { *cell.get() = BarrettReduction::<u128>::new(m as u128) })
    }
    fn rem(x: u128) -> u128 {
        DYN_MODULUS_U64.with(|cell| unsafe { (*cell.get()).rem(x) })
    }
}
impl DynMIntU64 {
    pub fn set_mod(m: u64) {
        DynModuloU64::set_mod(m)
    }
}

define_basic_mint32!(
    [Modulo998244353, 998_244_353, MInt998244353],
    [Modulo1000000007, 1_000_000_007, MInt1000000007],
    [Modulo1000000009, 1_000_000_009, MInt1000000009]
);

define_basic_mintbase!(
    DynModuloU32,
    DYN_MODULUS_U32.with(|cell| unsafe { (*cell.get()).get_mod() as u32 }),
    u32,
    i32,
    u64,
    [u32, u64, u128, usize],
    [i32, i64, i128, isize]
);
pub type DynMIntU32 = MInt<DynModuloU32>;
define_basic_mintbase!(
    DynModuloU64,
    DYN_MODULUS_U64.with(|cell| unsafe { (*cell.get()).get_mod() as u64 }),
    u64,
    i64,
    u128,
    [u64, u128, usize],
    [i64, i128, isize]
);
pub type DynMIntU64 = MInt<DynModuloU64>;

pub struct Modulo2;
impl MIntBase for Modulo2 {
    type Inner = u32;
    #[inline]
    fn get_mod() -> Self::Inner {
        2
    }
    #[inline]
    fn mod_zero() -> Self::Inner {
        0
    }
    #[inline]
    fn mod_one() -> Self::Inner {
        1
    }
    #[inline]
    fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        x ^ y
    }
    #[inline]
    fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        x ^ y
    }
    #[inline]
    fn mod_mul(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        x & y
    }
    #[inline]
    fn mod_div(x: Self::Inner, y: Self::Inner) -> Self::Inner {
        assert_ne!(y, 0);
        x
    }
    #[inline]
    fn mod_neg(x: Self::Inner) -> Self::Inner {
        x
    }
    #[inline]
    fn mod_inv(x: Self::Inner) -> Self::Inner {
        assert_ne!(x, 0);
        x
    }
    #[inline]
    fn mod_pow(x: Self::Inner, y: usize) -> Self::Inner {
        if y == 0 { 1 } else { x }
    }
}
macro_rules! impl_to_mint_base_for_modulo2 {
    ($name:ident, $basety:ty, [$($t:ty),*]) => {
        $(impl MIntConvert<$t> for $name {
            #[inline]
            fn from(x: $t) -> Self::Inner {
                (x & 1) as $basety
            }
            #[inline]
            fn into(x: Self::Inner) -> $t {
                x as $t
            }
            #[inline]
            fn mod_into() -> $t {
                2
            }
        })*
    };
}
impl_to_mint_base_for_modulo2!(
    Modulo2,
    u32,
    [
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
    ]
);
pub type MInt2 = MInt<Modulo2>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    macro_rules! test_mint {
        ($test_name:ident $mint:ident $($m:expr)?) => {
            #[test]
            fn $test_name() {
                let mut rng = Xorshift::default();
                const Q: usize = 10_000;
                for _ in 0..Q {
                    $($mint::set_mod(rng.gen(..$m));)?
                    let a = $mint::new_unchecked(rng.random(1..$mint::get_mod()));
                    let x = a.inv();
                    assert!(x.inner() < $mint::get_mod());
                    assert_eq!(a * x, $mint::one());
                }
                for _ in 0..100 {
                    let n = rng.random(0..100);
                    let x: Vec<$mint> = (0..n).map(|_| rng.random(..)).collect();
                    let y: Vec<$mint> = (0..n).map(|_| rng.random(..)).collect();
                    assert_eq!(
                        $mint::dot_product(&x, &y),
                        x.iter().zip(&y).map(|(&x, &y)| x * y).sum()
                    );
                }
                for n in 0..=576 {
                    let x = vec![$mint::new_unchecked($mint::get_mod() - 1); n];
                    assert_eq!(
                        $mint::dot_product(&x, &x),
                        x.iter().map(|&x| x * x).sum()
                    );
                }
            }
        };
    }
    test_mint!(test_mint2 MInt2);
    test_mint!(test_mint998244353 MInt998244353);
    test_mint!(test_mint1000000007 MInt1000000007);
    test_mint!(test_mint1000000009 MInt1000000009);

    macro_rules! test_dyn_mint_arithmetic {
        ($name:ident, $mint:ident, $int:ty, $signed:ty) => {
            #[test]
            fn $name() {
                let mut rng = Xorshift::default();
                let moduli: Vec<_> = [1, 2, <$int>::MAX]
                    .into_iter()
                    .chain(rng.random_iter(1..).take(100))
                    .collect();
                for modulus in moduli {
                    $mint::set_mod(modulus);
                    assert_eq!($mint::one().inner() as u128, 1 % modulus as u128);
                    let values: Vec<_> = [0, modulus - 1]
                        .into_iter()
                        .chain(rng.random_iter(0..modulus).take(16))
                        .collect();
                    for &x in &values {
                        if modulus > 1 && crate::math::gcd(x as u64, modulus as u64) == 1 {
                            let inverse = $mint::from(x).inv().inner();
                            assert!(inverse < modulus);
                            assert_eq!(x as u128 * inverse as u128 % modulus as u128, 1);
                        }
                        for &y in &values {
                            let a = $mint::from(x);
                            let b = $mint::from(y);
                            let modulus = modulus as u128;
                            assert_eq!((a + b).inner() as u128, (x as u128 + y as u128) % modulus);
                            assert_eq!(
                                (a - b).inner() as u128,
                                (x as u128 + modulus - y as u128) % modulus
                            );
                        }
                    }
                    for x in [<$signed>::MIN, -1, 0, 1, <$signed>::MAX]
                        .into_iter()
                        .chain(rng.random_iter(..).take(16))
                    {
                        assert_eq!(
                            $mint::from(x).inner() as i128,
                            (x as i128).rem_euclid(modulus as i128)
                        );
                    }
                    for x in [i128::MIN, -1, 0, 1, i128::MAX]
                        .into_iter()
                        .chain(rng.random_iter(..).take(16))
                    {
                        assert_eq!(
                            $mint::from(x).inner() as i128,
                            x.rem_euclid(modulus as i128)
                        );
                    }
                    let lengths: Vec<_> = [0, 1, 63, 64, 65, 511, 512, 513, 600]
                        .into_iter()
                        .chain(rng.random_iter(0..600).take(8))
                        .collect();
                    for n in lengths {
                        let x: Vec<$mint> = rng.random_iter(..).take(n).collect();
                        let y: Vec<$mint> = rng.random_iter(..).take(n).collect();
                        let expected = x.iter().zip(&y).fold(0, |sum, (x, y)| {
                            (sum + x.inner() as u128 * y.inner() as u128) % modulus as u128
                        });
                        assert_eq!($mint::dot_product(&x, &y).inner() as u128, expected);
                    }
                }
                $mint::set_mod(1_000_000_007);
            }
        };
    }
    test_dyn_mint_arithmetic!(test_dyn_mint_u32_arithmetic, DynMIntU32, u32, i32);
    test_dyn_mint_arithmetic!(test_dyn_mint_u64_arithmetic, DynMIntU64, u64, i64);

    #[test]
    fn test_dyn_mint_u32_dot_product() {
        DynMIntU32::set_mod(1_000_000_007);
        let mut rng = Xorshift::default();
        for n in 0..=576 {
            let x: Vec<DynMIntU32> = rng.random_iter(..).take(n).collect();
            let y: Vec<DynMIntU32> = rng.random_iter(..).take(n).collect();
            assert_eq!(
                DynMIntU32::dot_product(&x, &y),
                x.iter().zip(&y).map(|(&x, &y)| x * y).sum()
            );
        }
        DynMIntU32::set_mod(u32::MAX);
        for n in 0..=576 {
            let x = vec![DynMIntU32::new_unchecked(u32::MAX - 1); n];
            assert_eq!(
                DynMIntU32::dot_product(&x, &x),
                x.iter().map(|&x| x * x).sum()
            );
        }
        DynMIntU32::set_mod(1_000_000_007);
    }
}
