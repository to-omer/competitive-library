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
                1
            }
            #[inline]
            fn mod_add(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                let z = x + y;
                let m = Self::get_mod();
                if z >= m {
                    z - m
                } else {
                    z
                }
            }
            #[inline]
            fn mod_sub(x: Self::Inner, y: Self::Inner) -> Self::Inner {
                if x < y {
                    x + Self::get_mod() - y
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
                let p = Self::get_mod() as $signedty;
                let (mut a, mut b) = (x as $signedty, p);
                let (mut u, mut x) = (1, 0);
                while a != 0 {
                    let k = b / a;
                    x -= k * u;
                    b -= k * a;
                    swap(&mut x, &mut u);
                    swap(&mut b, &mut a);
                }
                (if x < 0 { x + p } else { x }) as _
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
                let x = x % <Self as MIntBase>::get_mod() as $signed;
                if x < 0 {
                    (x + <Self as MIntBase>::get_mod() as $signed) as $basety
                } else {
                    x as $basety
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

thread_local!(static DYN_MODULUS_U32: UnsafeCell<BarrettReduction<u64>> = UnsafeCell::new(BarrettReduction::<u64>::new(1_000_000_007)));
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

thread_local!(static DYN_MODULUS_U64: UnsafeCell<BarrettReduction<u128>> = UnsafeCell::new(BarrettReduction::<u128>::new(1_000_000_007)));
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
        if y == 0 {
            1
        } else {
            x
        }
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
                1
            }
        })*
    };
}
impl_to_mint_base_for_modulo2!(
    Modulo2,
    u32,
    [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
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
                let mut rng = Xorshift::time();
                const Q: usize = 10_000;
                for _ in 0..Q {
                    $($mint::set_mod(rng.gen(..$m));)?
                    let a = $mint::new_unchecked(rng.gen(1..$mint::get_mod()));
                    let x = a.inv();
                    assert!(x.inner() < $mint::get_mod());
                    assert_eq!(a * x, $mint::one());
                }
            }
        };
    }
    test_mint!(test_mint2 MInt2);
    test_mint!(test_mint998244353 MInt998244353);
    test_mint!(test_mint1000000007 MInt1000000007);
    test_mint!(test_mint1000000009 MInt1000000009);
}
