#[cfg(target_arch = "x86_64")]
use super::avx512_enabled;
use super::{MInt, MIntBase, MIntDotProduct, mint_basic::*};

#[macro_export]
macro_rules! impl_basic_mint_dot_product {
    (u32, u64; $($name:ident),* $(,)?) => {
        $($crate::impl_basic_mint_dot_product!(@impl #[inline(always)] simd32, $name, u32, u64);)*
    };
    ($basety:ty, $upperty:ty; $($name:ident),* $(,)?) => {
        $($crate::impl_basic_mint_dot_product!(@impl #[inline] scalar, $name, $basety, $upperty);)*
    };
    (@impl #[$inline:meta] $kind:ident, $name:ident, $basety:ty, $upperty:ty) => {
        impl MIntDotProduct for $name {
            fn try_matrix_product(_a: &[Vec<MInt<Self>>], _b: &[Vec<MInt<Self>>]) -> Option<Vec<Vec<MInt<Self>>>> {
                $crate::impl_basic_mint_dot_product!(@matrix_product $kind, _a, _b)
            }
            #[$inline]
            fn dot_product(x: &[MInt<Self>], y: &[MInt<Self>]) -> MInt<Self> {
                $crate::impl_basic_mint_dot_product!(@dot_product $kind, $name, x, y, $basety, $upperty)
            }
            #[inline]
            fn add_scaled_assign(x: &mut [MInt<Self>], y: &[MInt<Self>], a: &MInt<Self>) {
                assert_eq!(x.len(), y.len());
                $crate::impl_basic_mint_dot_product!(@add_scaled $kind, $name, x, y, a);
                for (x, y) in x.iter_mut().zip(y) { *x += *a * *y; }
            }
        }
        $crate::impl_basic_mint_dot_product!(@simd_functions $kind, $name);
    };
    (@dot_product scalar, $name:ident, $x:ident, $y:ident, $basety:ty, $upperty:ty) => {{
        assert_eq!($x.len(), $y.len());
        let modulus = Self::get_mod() as $upperty;
        let max_value = modulus - 1;
        let block = ((<$upperty>::MAX - max_value) / (max_value * max_value).max(1)).min(64) as usize;
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
        MInt::new_unchecked(result as $basety)
    }};
    (@dot_product simd32, $name:ident, $x:ident, $y:ident, $basety:ty, $upperty:ty) => {{
        #[cfg(target_arch = "x86_64")]
        {
            if $x.len() >= 32 {
                if $x.len() >= 512
                    && avx512_enabled()
                    && is_x86_feature_detected!("avx512f")
                {
                    return MInt::new_unchecked(unsafe { $name::dot_product_avx512($x, $y) });
                }
                if is_x86_feature_detected!("avx2") {
                    return MInt::new_unchecked(unsafe { $name::dot_product_avx2($x, $y) });
                }
            }
        }
        $crate::impl_basic_mint_dot_product!(@dot_product scalar, $name, $x, $y, $basety, $upperty)
    }};
    (@matrix_product scalar, $a:ident, $b:ident) => { None };
    (@matrix_product simd32, $a:ident, $b:ident) => {{
        #[cfg(target_arch = "x86_64")]
        if $a.len() >= 32 && $b.len() >= 32 && $b[0].len() >= 32
            && Self::get_mod() > 1 && Self::get_mod() < 1 << 30
            && Self::get_mod() % 2 == 1 && is_x86_feature_detected!("avx2")
        {
            let scale = ((1u64 << 32) % Self::get_mod() as u64) as u32;
            return Some(unsafe { MInt::matrix_product_avx2($a, $b, scale) });
        }
        None
    }};
    (@add_scaled scalar, $name:ident, $x:ident, $y:ident, $a:ident) => {};
    (@add_scaled simd32, $name:ident, $x:ident, $y:ident, $a:ident) => {
        #[cfg(target_arch = "x86_64")]
        if $x.len() >= 16 && Self::get_mod() <= 1 << 31 {
            if $x.len() >= 64 && avx512_enabled() && is_x86_feature_detected!("avx512f") {
                unsafe { Self::add_scaled_avx512($x, $y, $a.inner()) };
                return;
            }
            if is_x86_feature_detected!("avx2") {
                unsafe { Self::add_scaled_avx2($x, $y, $a.inner()) };
                return;
            }
        }
    };
    (@simd_functions scalar, $name:ident) => {};
    (@simd_functions simd32, $name:ident) => {
        #[cfg(target_arch = "x86_64")]
        impl $name {
            #[allow(unsafe_op_in_unsafe_fn)]
            #[target_feature(enable = "avx2")]
            unsafe fn add_scaled_avx2(x: &mut [MInt<Self>], y: &[MInt<Self>], a: u32) {
                use std::arch::x86_64::*;
                let modulus = _mm256_set1_epi32(Self::get_mod() as i32);
                let factor = _mm256_set1_epi32(a as i32);
                // This quotient underestimates floor(a*y/m) by at most one.
                let quotient = _mm256_set1_epi32((((a as u64) << 32) / Self::get_mod() as u64) as i32);
                let update = |old, value| {
                    let lo = _mm256_srli_epi64::<32>(_mm256_mul_epu32(value, quotient));
                    let hi = _mm256_slli_epi64::<32>(_mm256_srli_epi64::<32>(_mm256_mul_epu32(_mm256_srli_epi64::<32>(value), quotient)));
                    let q = _mm256_or_si256(lo, hi);
                    let product = _mm256_sub_epi32(_mm256_mullo_epi32(value, factor), _mm256_mullo_epi32(q, modulus));
                    let product = _mm256_min_epu32(product, _mm256_sub_epi32(product, modulus));
                    let sum = _mm256_add_epi32(old, product);
                    _mm256_min_epu32(sum, _mm256_sub_epi32(sum, modulus))
                };
                let end = x.len() / 8 * 8;
                for i in (0..end).step_by(8) {
                    let value = _mm256_loadu_si256(y.as_ptr().add(i).cast());
                    let old = _mm256_loadu_si256(x.as_ptr().add(i).cast());
                    _mm256_storeu_si256(x.as_mut_ptr().add(i).cast(), update(old, value));
                }
                if end < x.len() {
                    let mask = _mm256_cmpgt_epi32(_mm256_set1_epi32((x.len() - end) as i32), _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7));
                    let value = _mm256_maskload_epi32(y.as_ptr().add(end).cast(), mask);
                    let old = _mm256_maskload_epi32(x.as_ptr().add(end).cast(), mask);
                    _mm256_maskstore_epi32(x.as_mut_ptr().add(end).cast(), mask, update(old, value));
                }
            }

            #[allow(unsafe_op_in_unsafe_fn)]
            #[target_feature(enable = "avx512f")]
            unsafe fn add_scaled_avx512(x: &mut [MInt<Self>], y: &[MInt<Self>], a: u32) {
                use std::arch::x86_64::*;
                let modulus = _mm512_set1_epi32(Self::get_mod() as i32);
                let factor = _mm512_set1_epi32(a as i32);
                // This quotient underestimates floor(a*y/m) by at most one.
                let quotient = _mm512_set1_epi32((((a as u64) << 32) / Self::get_mod() as u64) as i32);
                let end = x.len() / 16 * 16;
                for i in (0..end).step_by(16) {
                    let value = _mm512_loadu_si512(y.as_ptr().add(i).cast());
                    let lo = _mm512_srli_epi64::<32>(_mm512_mul_epu32(value, quotient));
                    let hi = _mm512_slli_epi64::<32>(_mm512_srli_epi64::<32>(_mm512_mul_epu32(_mm512_srli_epi64::<32>(value), quotient)));
                    let q = _mm512_or_si512(lo, hi);
                    let product = _mm512_sub_epi32(_mm512_mullo_epi32(value, factor), _mm512_mullo_epi32(q, modulus));
                    let product = _mm512_min_epu32(product, _mm512_sub_epi32(product, modulus));
                    let old = _mm512_loadu_si512(x.as_ptr().add(i).cast());
                    let sum = _mm512_add_epi32(old, product);
                    let sum = _mm512_min_epu32(sum, _mm512_sub_epi32(sum, modulus));
                    _mm512_storeu_si512(x.as_mut_ptr().add(i).cast(), sum);
                }
                let a = MInt::new_unchecked(a);
                for (x, y) in x[end..].iter_mut().zip(&y[end..]) { *x += a * *y; }
            }

            #[allow(unsafe_op_in_unsafe_fn)]
            #[target_feature(enable = "avx2")]
            unsafe fn dot_product_avx2(x: &[MInt<Self>], y: &[MInt<Self>]) -> u32 {
                use std::arch::x86_64::*;
                assert_eq!(x.len(), y.len());
                let modulus = Self::get_mod() as u64;
                let (products, bound) = if modulus <= 1 << 30 {
                    (8, 2 * modulus)
                } else if modulus <= 1 << 31 {
                    (2, modulus)
                } else {
                    (
                        ((u64::MAX - (modulus - 1)) / ((modulus - 1) * (modulus - 1))) as usize,
                        0,
                    )
                };
                // For moduli up to 2^31, each batch adds less than bound*2^32.
                let bound = _mm256_set1_epi64x((bound << 32) as i64);
                let len = x.len();
                let x = x.as_ptr().cast::<u32>();
                let y = y.as_ptr().cast::<u32>();
                let mut even = _mm256_setzero_si256();
                let mut odd = even;
                let mut offset = 0;
                let mut result = 0u64;
                loop {
                    while offset + 8 <= len {
                        let end = (offset + 8 * products).min(len / 8 * 8);
                        while offset < end {
                            let xv = _mm256_loadu_si256(x.add(offset).cast());
                            let yv = _mm256_loadu_si256(y.add(offset).cast());
                            even = _mm256_add_epi64(even, _mm256_mul_epu32(xv, yv));
                            odd = _mm256_add_epi64(
                                odd,
                                _mm256_mul_epu32(_mm256_srli_epi64::<32>(xv), _mm256_srli_epi64::<32>(yv)),
                            );
                            offset += 8;
                        }
                        if modulus > 1 << 31 {
                            break;
                        }
                        even = _mm256_min_epu32(even, _mm256_sub_epi32(even, bound));
                        odd = _mm256_min_epu32(odd, _mm256_sub_epi32(odd, bound));
                    }
                    let mut lanes = [0u64; 8];
                    _mm256_storeu_si256(lanes.as_mut_ptr().cast(), even);
                    _mm256_storeu_si256(lanes.as_mut_ptr().add(4).cast(), odd);
                    let mut low = 0u64;
                    let mut high = 0u64;
                    for lane in lanes {
                        low += lane as u32 as u64;
                        high += lane >> 32;
                    }
                    result = (result + low + (high % modulus) * ((1u64 << 32) % modulus)) % modulus;
                    if offset + 8 > len {
                        break;
                    }
                    even = _mm256_setzero_si256();
                    odd = even;
                }
                for first in (offset..len).step_by(products) {
                    for i in first..(first + products).min(len) {
                        result += *x.add(i) as u64 * *y.add(i) as u64;
                    }
                    result %= modulus;
                }
                result as u32
            }

            #[allow(unsafe_op_in_unsafe_fn)]
            #[target_feature(enable = "avx512f")]
            unsafe fn dot_product_avx512(x: &[MInt<Self>], y: &[MInt<Self>]) -> u32 {
                use std::arch::x86_64::*;
                assert_eq!(x.len(), y.len());
                let modulus = Self::get_mod() as u64;
                let (products, bound) = if modulus <= 1 << 30 {
                    (8, 2 * modulus)
                } else if modulus <= 1 << 31 {
                    (2, modulus)
                } else {
                    (
                        ((u64::MAX - (modulus - 1)) / ((modulus - 1) * (modulus - 1))) as usize,
                        0,
                    )
                };
                // For moduli up to 2^31, each batch adds less than bound*2^32.
                let bound = _mm512_set1_epi64((bound << 32) as i64);
                let len = x.len();
                let x = x.as_ptr().cast::<u32>();
                let y = y.as_ptr().cast::<u32>();
                let mut even = _mm512_setzero_si512();
                let mut odd = even;
                let mut offset = 0;
                let mut result = 0u64;
                loop {
                    while offset + 16 <= len {
                        let end = (offset + 16 * products).min(len / 16 * 16);
                        while offset < end {
                            let xv = _mm512_loadu_si512(x.add(offset).cast());
                            let yv = _mm512_loadu_si512(y.add(offset).cast());
                            even = _mm512_add_epi64(even, _mm512_mul_epu32(xv, yv));
                            odd = _mm512_add_epi64(
                                odd,
                                _mm512_mul_epu32(_mm512_srli_epi64::<32>(xv), _mm512_srli_epi64::<32>(yv)),
                            );
                            offset += 16;
                        }
                        if modulus > 1 << 31 {
                            break;
                        }
                        even = _mm512_min_epu32(even, _mm512_sub_epi32(even, bound));
                        odd = _mm512_min_epu32(odd, _mm512_sub_epi32(odd, bound));
                    }
                    let mut lanes = [0u64; 16];
                    _mm512_storeu_si512(lanes.as_mut_ptr().cast(), even);
                    _mm512_storeu_si512(lanes.as_mut_ptr().add(8).cast(), odd);
                    let mut low = 0u64;
                    let mut high = 0u64;
                    for lane in lanes {
                        low += lane as u32 as u64;
                        high += lane >> 32;
                    }
                    result = (result + low + (high % modulus) * ((1u64 << 32) % modulus)) % modulus;
                    if offset + 16 > len {
                        break;
                    }
                    even = _mm512_setzero_si512();
                    odd = even;
                }
                for first in (offset..len).step_by(products) {
                    for i in first..(first + products).min(len) {
                        result += *x.add(i) as u64 * *y.add(i) as u64;
                    }
                    result %= modulus;
                }
                result as u32
            }
        }
    };
}

impl_basic_mint_dot_product!(u32, u64; Modulo998244353, Modulo1000000007, Modulo1000000009, DynModuloU32);
impl_basic_mint_dot_product!(u64, u128; DynModuloU64);
impl MIntDotProduct for Modulo2 {}
