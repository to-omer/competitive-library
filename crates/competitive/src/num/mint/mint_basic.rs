use super::*;
use std::{cell::UnsafeCell, mem::swap};

#[macro_export]
macro_rules! define_basic_mintbase {
    ($name:ident, $m:expr, $basety:ty, $signedty:ty, $upperty:ty, [$($unsigned:ty),*], [$($signed:ty),*]) => {
        $crate::define_basic_mintbase!(
            @impl #[inline] scalar,
            $name,
            $m,
            $basety,
            $signedty,
            $upperty,
            [$($unsigned),*],
            [$($signed),*]
        );
    };
    (@simd32 $name:ident, $m:expr, u32, i32, u64, [$($unsigned:ty),*], [$($signed:ty),*]) => {
        $crate::define_basic_mintbase!(
            @impl #[inline(always)] simd32,
            $name,
            $m,
            u32,
            i32,
            u64,
            [$($unsigned),*],
            [$($signed),*]
        );
    };
    (@impl #[$inline:meta] $dot_product:ident, $name:ident, $m:expr, $basety:ty, $signedty:ty, $upperty:ty, [$($unsigned:ty),*], [$($signed:ty),*]) => {
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
            #[$inline]
            fn mod_dot_product(x: &[MInt<Self>], y: &[MInt<Self>]) -> Self::Inner {
                $crate::define_basic_mintbase!(@dot_product $dot_product, $name, x, y, $basety, $upperty)
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
        $crate::define_basic_mintbase!(@simd_functions $dot_product, $name);
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
    (@dot_product scalar, $name:ident, $x:ident, $y:ident, $basety:ty, $upperty:ty) => {{
        assert_eq!($x.len(), $y.len());
        let modulus = Self::get_mod() as $upperty;
        let max_value = modulus - 1;
        let block = ((<$upperty>::MAX - max_value) / (max_value * max_value)).min(64) as usize;
        let mut result = 0 as $upperty;
        for (x, y) in $x.chunks(block).zip($y.chunks(block)) {
            let sum: $upperty = x
                .iter()
                .zip(y)
                .map(|(&x, &y)| x.inner() as $upperty * y.inner() as $upperty)
                .sum();
            result += sum % modulus;
            if result >= modulus {
                result -= modulus;
            }
        }
        result as $basety
    }};
    (@dot_product simd32, $name:ident, $x:ident, $y:ident, $basety:ty, $upperty:ty) => {{
        #[cfg(target_arch = "x86_64")]
        {
            if $x.len() >= 64 {
                if $x.len() >= 512
                    && avx512_enabled()
                    && is_x86_feature_detected!("avx512f")
                {
                    // SAFETY: feature detection checked AVX-512F.
                    return unsafe { $name::dot_product_avx512($x, $y) };
                }
                if is_x86_feature_detected!("avx2") {
                    // SAFETY: feature detection checked AVX2.
                    return unsafe { $name::dot_product_avx2($x, $y) };
                }
            }
        }
        $crate::define_basic_mintbase!(@dot_product scalar, $name, $x, $y, $basety, $upperty)
    }};
    (@simd_functions scalar, $name:ident) => {};
    (@simd_functions simd32, $name:ident) => {
        #[cfg(target_arch = "x86_64")]
        impl $name {
            #[allow(unsafe_op_in_unsafe_fn)]
            #[target_feature(enable = "avx2")]
            unsafe fn dot_product_avx2(x: &[MInt<Self>], y: &[MInt<Self>]) -> u32 {
                use std::arch::x86_64::*;

                assert_eq!(x.len(), y.len());
                let modulus = Self::get_mod() as u64;
                let max_value = modulus - 1;
                let max_product = max_value * max_value;
                let products_per_lane = ((u64::MAX - max_value) / max_product) as usize;
                let vectors = products_per_lane.min(18);
                let len = x.len();
                let x = x.as_ptr().cast::<u32>();
                let y = y.as_ptr().cast::<u32>();
                let mut result = 0u64;
                let mut offset = 0;
                while offset + 8 <= len {
                    let batch = ((len - offset) / 8).min(vectors);
                    let mut even = _mm256_setzero_si256();
                    let mut odd = _mm256_setzero_si256();
                    for i in 0..batch {
                        let index = offset + i * 8;
                        let xv = _mm256_loadu_si256(x.add(index).cast());
                        let yv = _mm256_loadu_si256(y.add(index).cast());
                        even = _mm256_add_epi64(even, _mm256_mul_epu32(xv, yv));
                        odd = _mm256_add_epi64(
                            odd,
                            _mm256_mul_epu32(
                                _mm256_srli_epi64::<32>(xv),
                                _mm256_srli_epi64::<32>(yv),
                            ),
                        );
                    }
                    let mut lanes = [0u64; 8];
                    _mm256_storeu_si256(lanes.as_mut_ptr().cast(), even);
                    _mm256_storeu_si256(lanes.as_mut_ptr().add(4).cast(), odd);
                    for lane in lanes {
                        result += lane % modulus;
                        if result >= modulus {
                            result -= modulus;
                        }
                    }
                    offset += batch * 8;
                }
                let block = products_per_lane.min(64);
                for i in (offset..len).step_by(block) {
                    let end = (i + block).min(len);
                    let mut sum = 0u64;
                    for j in i..end {
                        sum += *x.add(j) as u64 * *y.add(j) as u64;
                    }
                    result += sum % modulus;
                    if result >= modulus {
                        result -= modulus;
                    }
                }
                result as u32
            }

            #[allow(unsafe_op_in_unsafe_fn)]
            #[target_feature(enable = "avx512f")]
            unsafe fn dot_product_avx512(x: &[MInt<Self>], y: &[MInt<Self>]) -> u32 {
                use std::arch::x86_64::*;

                assert_eq!(x.len(), y.len());
                let modulus = Self::get_mod() as u64;
                let max_value = modulus - 1;
                let max_product = max_value * max_value;
                let products_per_lane = ((u64::MAX - max_value) / max_product) as usize;
                let vectors = products_per_lane.min(18);
                let len = x.len();
                let x = x.as_ptr().cast::<u32>();
                let y = y.as_ptr().cast::<u32>();
                let mut result = 0u64;
                let mut offset = 0;
                while offset + 16 <= len {
                    let batch = ((len - offset) / 16).min(vectors);
                    let mut even0 = _mm512_setzero_si512();
                    let mut odd0 = _mm512_setzero_si512();
                    let mut even1 = _mm512_setzero_si512();
                    let mut odd1 = _mm512_setzero_si512();
                    let mut i = 0;
                    while i + 1 < batch {
                        let index = offset + i * 16;
                        let xv0 = _mm512_loadu_si512(x.add(index).cast());
                        let yv0 = _mm512_loadu_si512(y.add(index).cast());
                        let xv1 = _mm512_loadu_si512(x.add(index + 16).cast());
                        let yv1 = _mm512_loadu_si512(y.add(index + 16).cast());
                        even0 = _mm512_add_epi64(even0, _mm512_mul_epu32(xv0, yv0));
                        odd0 = _mm512_add_epi64(
                            odd0,
                            _mm512_mul_epu32(
                                _mm512_srli_epi64::<32>(xv0),
                                _mm512_srli_epi64::<32>(yv0),
                            ),
                        );
                        even1 = _mm512_add_epi64(even1, _mm512_mul_epu32(xv1, yv1));
                        odd1 = _mm512_add_epi64(
                            odd1,
                            _mm512_mul_epu32(
                                _mm512_srli_epi64::<32>(xv1),
                                _mm512_srli_epi64::<32>(yv1),
                            ),
                        );
                        i += 2;
                    }
                    if i < batch {
                        let index = offset + i * 16;
                        let xv = _mm512_loadu_si512(x.add(index).cast());
                        let yv = _mm512_loadu_si512(y.add(index).cast());
                        even0 = _mm512_add_epi64(even0, _mm512_mul_epu32(xv, yv));
                        odd0 = _mm512_add_epi64(
                            odd0,
                            _mm512_mul_epu32(
                                _mm512_srli_epi64::<32>(xv),
                                _mm512_srli_epi64::<32>(yv),
                            ),
                        );
                    }
                    let mut lanes = [0u64; 16];
                    _mm512_storeu_si512(
                        lanes.as_mut_ptr().cast(),
                        _mm512_add_epi64(even0, even1),
                    );
                    _mm512_storeu_si512(
                        lanes.as_mut_ptr().add(8).cast(),
                        _mm512_add_epi64(odd0, odd1),
                    );
                    for lane in lanes {
                        result += lane % modulus;
                        if result >= modulus {
                            result -= modulus;
                        }
                    }
                    offset += batch * 16;
                }
                let block = products_per_lane.min(64);
                for i in (offset..len).step_by(block) {
                    let end = (i + block).min(len);
                    let mut sum = 0u64;
                    for j in i..end {
                        sum += *x.add(j) as u64 * *y.add(j) as u64;
                    }
                    result += sum % modulus;
                    if result >= modulus {
                        result -= modulus;
                    }
                }
                result as u32
            }
        }
    };
}

#[macro_export]
macro_rules! define_basic_mint32 {
    ($([$name:ident, $m:expr, $mint_name:ident]),*) => {
        $(define_basic_mintbase!(@simd32
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

define_basic_mintbase!(@simd32
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
                1
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
