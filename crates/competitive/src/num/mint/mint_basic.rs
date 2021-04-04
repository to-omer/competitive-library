use super::*;

#[macro_export]
macro_rules! define_basic_mintbase {
    ($name:ident, $m:expr, $basety:ty, $upperty:ty, [$($unsigned:ty),*], [$($signed:ty),*]) => {
        pub struct $name;
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
                (x as $upperty * y as $upperty % Self::get_mod() as $upperty) as $basety
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
                let mut a = x;
                let (mut b, mut u, mut s) = (Self::get_mod(), 1, 0);
                let k = a.trailing_zeros();
                a >>= k;
                for _ in 0..k {
                    if u & 1 == 1 {
                        u += Self::get_mod();
                    }
                    u /= 2;
                }
                while a != b {
                    if b < a {
                        std::mem::swap(&mut a, &mut b);
                        std::mem::swap(&mut u, &mut s);
                    }
                    b -= a;
                    if s < u {
                        s += Self::get_mod();
                    }
                    s -= u;
                    let k = b.trailing_zeros();
                    b >>= k;
                    for _ in 0..k {
                        if s & 1 == 1 {
                            s += Self::get_mod();
                        }
                        s /= 2;
                    }
                }
                s
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
        $(crate::define_basic_mintbase!(
            $name,
            $m,
            u32,
            u64,
            [u32, u64, u128, usize],
            [i32, i64, i128, isize]
        );
        pub type $mint_name = MInt<$name>;)*
    };
}
define_basic_mint32!(
    [Modulo998244353, 998_244_353, MInt998244353],
    [Modulo1000000007, 1_000_000_007, MInt1000000007],
    [Modulo1000000009, 1_000_000_009, MInt1000000009],
    [
        DynModuloU32,
        DYN_MODULUS_U32.with(|cell| unsafe { *cell.get() }),
        DynMIntU32
    ]
);

thread_local!(static DYN_MODULUS_U32: std::cell::UnsafeCell<u32> = std::cell::UnsafeCell::new(1_000_000_007));
impl DynModuloU32 {
    pub fn set_mod(m: u32) {
        DYN_MODULUS_U32.with(|cell| unsafe { *cell.get() = m })
    }
}
thread_local!(static DYN_MODULUS_U64: std::cell::UnsafeCell<u64> = std::cell::UnsafeCell::new(1_000_000_007));
define_basic_mintbase!(
    DynModuloU64,
    DYN_MODULUS_U64.with(|cell| unsafe { *cell.get() }),
    u64,
    u128,
    [u64, u128, usize],
    [i64, i128, isize]
);
impl DynModuloU64 {
    pub fn set_mod(m: u64) {
        DYN_MODULUS_U64.with(|cell| unsafe { *cell.get() = m })
    }
}
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
        x | y
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

#[test]
fn test_mint() {
    use crate::tools::Xorshift;
    let mut rng = Xorshift::time();
    const Q: usize = 10_000;
    for _ in 0..Q {
        let a = MInt998244353::new_unchecked(rng.gen(1..MInt998244353::get_mod()));
        let x = a.inv();
        assert!(x.x < MInt998244353::get_mod());
        assert_eq!(a * x, MInt998244353::one());
        assert_eq!(x, a.pow(MInt998244353::get_mod() as usize - 2));
    }
}
