use super::{MInt, MIntDotProduct, montgomery::MontgomeryReduction32};
#[cfg(target_arch = "x86_64")]
use super::{avx512_enabled, avx512_supported};

impl<M> MIntDotProduct for M
where
    M: MontgomeryReduction32,
{
    fn try_matrix_product(
        _a: &[Vec<MInt<Self>>],
        _b: &[Vec<MInt<Self>>],
    ) -> Option<Vec<Vec<MInt<Self>>>> {
        #[cfg(target_arch = "x86_64")]
        if _a.len() >= 32
            && _b.len() >= 32
            && _b[0].len() >= 32
            && <Self as MontgomeryReduction32>::MOD > 1
            && <Self as MontgomeryReduction32>::MOD < 1 << 30
            && <Self as MontgomeryReduction32>::MOD % 2 == 1
            && is_x86_feature_detected!("avx2")
        {
            return Some(unsafe { MInt::matrix_product_avx2(_a, _b, 1) });
        }
        None
    }
    fn dot_product(x: &[MInt<Self>], y: &[MInt<Self>]) -> MInt<Self> {
        dot_product::<Self, false>(x, y)
    }
    fn dot_product_rev(x: &[MInt<Self>], y: &[MInt<Self>]) -> MInt<Self> {
        dot_product::<Self, true>(x, y)
    }
    fn add_scaled_assign(x: &mut [MInt<Self>], y: &[MInt<Self>], a: &MInt<Self>) {
        assert_eq!(x.len(), y.len());
        #[cfg(target_arch = "x86_64")]
        if x.len() >= 16 {
            if x.len() >= 64 && avx512_enabled() && avx512_supported() {
                unsafe { simd::add_scaled_avx512::<Self>(x, y, a) };
                return;
            }
            if is_x86_feature_detected!("avx2") {
                unsafe { simd::add_scaled_avx2::<Self>(x, y, a) };
                return;
            }
        }
        for (x, y) in x.iter_mut().zip(y) {
            *x += *a * *y;
        }
    }
}

fn dot_product<M, const REVERSE: bool>(x: &[MInt<M>], y: &[MInt<M>]) -> MInt<M>
where
    M: MontgomeryReduction32,
{
    assert_eq!(x.len(), y.len());
    let reduce = |sum: u128| {
        // Montgomery reduction accepts values below modulus * 2^32.
        let value = if sum < (M::MOD as u128) << 32 {
            sum as u64
        } else if sum <= u64::MAX as u128 {
            sum as u64 % M::MOD as u64
        } else {
            (sum % M::MOD as u128) as u64
        };
        MInt::new_unchecked(M::reduce(value))
    };
    // SAFETY: MInt is transparent over u32; retain the Montgomery residues.
    let (x, y) = unsafe {
        (
            std::slice::from_raw_parts(x.as_ptr().cast::<u32>(), x.len()),
            std::slice::from_raw_parts(y.as_ptr().cast::<u32>(), y.len()),
        )
    };
    #[cfg(target_arch = "x86_64")]
    if x.len() >= 64 && M::MOD < 1 << 31 {
        if avx512_enabled() && avx512_supported() {
            let sum = unsafe { simd::dot_product_avx512::<REVERSE>(x, y) };
            return reduce(sum);
        }
        if is_x86_feature_detected!("avx2") {
            let sum = unsafe { simd::dot_product_avx2::<REVERSE>(x, y) };
            return reduce(sum);
        }
    }
    let sum = dot_product_scalar::<M, REVERSE>(x, y);
    reduce(sum)
}

fn dot_product_scalar<M, const REVERSE: bool>(x: &[u32], y: &[u32]) -> u128
where
    M: MontgomeryReduction32,
{
    let max_value = (M::MOD - 1) as u64;
    let block = (u64::MAX / (max_value * max_value)).min(16) as usize;
    let mut sum = 0u128;
    for (i, x) in x.chunks(block).enumerate() {
        let start = i * block;
        let part: u64 = if REVERSE {
            x.iter()
                .zip(y[y.len() - start - x.len()..y.len() - start].iter().rev())
                .map(|(&x, &y)| x as u64 * y as u64)
                .sum()
        } else {
            x.iter()
                .zip(&y[start..start + x.len()])
                .map(|(&x, &y)| x as u64 * y as u64)
                .sum()
        };
        sum += part as u128;
    }
    sum
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics and raw pointers are confined here
mod simd {
    use super::super::montgomery_simd::{add_mod_256, add_mod_512};
    use super::{MInt, MontgomeryReduction32};
    use std::arch::x86_64::*;

    /// # Safety
    /// AVX2 must be available. Slices have equal lengths and residues are below 2^31.
    #[target_feature(enable = "avx2")]
    pub unsafe fn dot_product_avx2<const REVERSE: bool>(x: &[u32], y: &[u32]) -> u128 {
        let zero = _mm256_setzero_si256();
        let order = _mm256_setr_epi32(7, 6, 5, 4, 3, 2, 1, 0);
        let mut low = zero;
        let mut high = zero;
        let end = x.len() / 8 * 8;
        for i in (0..end).step_by(8) {
            let a = _mm256_loadu_si256(x.as_ptr().add(i).cast());
            let j = if REVERSE { y.len() - i - 8 } else { i };
            let mut b = _mm256_loadu_si256(y.as_ptr().add(j).cast());
            if REVERSE {
                b = _mm256_permutevar8x32_epi32(b, order);
            }
            let even = _mm256_mul_epu32(a, b);
            let odd = _mm256_mul_epu32(_mm256_srli_epi64::<32>(a), _mm256_srli_epi64::<32>(b));
            let next = _mm256_add_epi64(low, _mm256_add_epi64(even, odd));
            // Each increment is below 2^63, so a falling sign bit detects a carry.
            high = _mm256_sub_epi64(
                high,
                _mm256_cmpgt_epi64(zero, _mm256_andnot_si256(next, low)),
            );
            low = next;
        }
        let low: [u64; 4] = std::mem::transmute(low);
        let high: [u64; 4] = std::mem::transmute(high);
        let mut sum: u128 = low
            .into_iter()
            .zip(high)
            .map(|(l, h)| ((h as u128) << 64) + l as u128)
            .sum();
        for i in end..x.len() {
            sum += x[i] as u128 * y[if REVERSE { y.len() - 1 - i } else { i }] as u128;
        }
        sum
    }

    /// # Safety
    /// AVX-512F/DQ/CD/BW/VL must be available. Slices have equal lengths and residues are below 2^31.
    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn dot_product_avx512<const REVERSE: bool>(x: &[u32], y: &[u32]) -> u128 {
        let mut low = _mm512_setzero_si512();
        let mut high = low;
        let order = _mm512_setr_epi32(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let one = _mm512_set1_epi64(1);
        let end = x.len() / 16 * 16;
        for i in (0..end).step_by(16) {
            let a = _mm512_loadu_si512(x.as_ptr().add(i).cast());
            let j = if REVERSE { y.len() - i - 16 } else { i };
            let mut b = _mm512_loadu_si512(y.as_ptr().add(j).cast());
            if REVERSE {
                b = _mm512_permutexvar_epi32(order, b);
            }
            let even = _mm512_mul_epu32(a, b);
            let odd = _mm512_mul_epu32(_mm512_srli_epi64::<32>(a), _mm512_srli_epi64::<32>(b));
            let next = _mm512_add_epi64(low, _mm512_add_epi64(even, odd));
            high = _mm512_mask_add_epi64(high, _mm512_cmplt_epu64_mask(next, low), high, one);
            low = next;
        }
        let low: [u64; 8] = std::mem::transmute(low);
        let high: [u64; 8] = std::mem::transmute(high);
        let mut sum: u128 = low
            .into_iter()
            .zip(high)
            .map(|(l, h)| ((h as u128) << 64) + l as u128)
            .sum();
        for i in end..x.len() {
            sum += x[i] as u128 * y[if REVERSE { y.len() - 1 - i } else { i }] as u128;
        }
        sum
    }

    /// # Safety
    /// AVX2 must be available, and `x` and `y` must have equal lengths.
    #[target_feature(enable = "avx2")]
    pub unsafe fn add_scaled_avx2<M: MontgomeryReduction32>(
        x: &mut [MInt<M>],
        y: &[MInt<M>],
        a: &MInt<M>,
    ) {
        // SAFETY: MInt is transparent over u32; preserve the Montgomery representation.
        let a = *(a as *const MInt<M>).cast::<u32>();
        let factor = _mm256_set1_epi32(a as i32);
        let factor_r = _mm256_set1_epi32(a.wrapping_mul(M::R) as i32);
        let modulus = _mm256_set1_epi32(M::MOD as i32);
        let end = x.len() / 8 * 8;
        for i in (0..end).step_by(8) {
            let value = _mm256_loadu_si256(y.as_ptr().add(i).cast());
            let odd = _mm256_srli_epi64::<32>(value);
            let lo = _mm256_mul_epu32(value, factor);
            let hi = _mm256_mul_epu32(odd, factor);
            let lo = _mm256_add_epi64(
                lo,
                _mm256_mul_epu32(_mm256_mul_epu32(value, factor_r), modulus),
            );
            let hi = _mm256_add_epi64(
                hi,
                _mm256_mul_epu32(_mm256_mul_epu32(odd, factor_r), modulus),
            );
            let product = _mm256_or_si256(_mm256_srli_epi64::<32>(lo), hi);
            let product = _mm256_min_epu32(product, _mm256_sub_epi32(product, modulus));
            let old = _mm256_loadu_si256(x.as_ptr().add(i).cast());
            let sum = add_mod_256(old, product, modulus);
            _mm256_storeu_si256(x.as_mut_ptr().add(i).cast(), sum);
        }
        let a = MInt::new_unchecked(a);
        for (x, y) in x[end..].iter_mut().zip(&y[end..]) {
            *x += a * *y;
        }
    }

    /// # Safety
    /// AVX-512F/DQ/CD/BW/VL must be available, and `x` and `y` must have equal lengths.
    #[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
    pub unsafe fn add_scaled_avx512<M: MontgomeryReduction32>(
        x: &mut [MInt<M>],
        y: &[MInt<M>],
        a: &MInt<M>,
    ) {
        // SAFETY: MInt is transparent over u32; preserve the Montgomery representation.
        let a = *(a as *const MInt<M>).cast::<u32>();
        let factor = _mm512_set1_epi32(a as i32);
        let factor_r = _mm512_set1_epi32(a.wrapping_mul(M::R) as i32);
        let modulus = _mm512_set1_epi32(M::MOD as i32);
        let end = x.len() / 16 * 16;
        for i in (0..end).step_by(16) {
            let value = _mm512_loadu_si512(y.as_ptr().add(i).cast());
            let odd = _mm512_srli_epi64::<32>(value);
            let lo = _mm512_mul_epu32(value, factor);
            let hi = _mm512_mul_epu32(odd, factor);
            let lo = _mm512_add_epi64(
                lo,
                _mm512_mul_epu32(_mm512_mul_epu32(value, factor_r), modulus),
            );
            let hi = _mm512_add_epi64(
                hi,
                _mm512_mul_epu32(_mm512_mul_epu32(odd, factor_r), modulus),
            );
            let product = _mm512_or_si512(_mm512_srli_epi64::<32>(lo), hi);
            let product = _mm512_min_epu32(product, _mm512_sub_epi32(product, modulus));
            let old = _mm512_loadu_si512(x.as_ptr().add(i).cast());
            let sum = add_mod_512(old, product, modulus);
            _mm512_storeu_si512(x.as_mut_ptr().add(i).cast(), sum);
        }
        let a = MInt::new_unchecked(a);
        for (x, y) in x[end..].iter_mut().zip(&y[end..]) {
            *x += a * *y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::montgomery::{Modulo167772161, Modulo998244353},
        tools::Xorshift,
    };

    #[test]
    fn test_dot_product() {
        fn check<M>()
        where
            M: MontgomeryReduction32,
        {
            let mut rng = Xorshift::default();
            let lengths: Vec<_> = (0..=129)
                .chain([
                    255, 256, 257, 511, 512, 513, 1023, 1024, 1025, 65535, 65536, 65537,
                ])
                .chain((0..32).map(|_| rng.random(130..65535)))
                .collect();
            for n in lengths {
                for maximum in [false, true] {
                    let x: Vec<_> = (0..n + 1)
                        .map(|_| {
                            if maximum {
                                M::MOD - 1
                            } else {
                                rng.random(..M::MOD)
                            }
                        })
                        .collect();
                    let y: Vec<_> = (0..n + 1)
                        .map(|_| {
                            if maximum {
                                M::MOD - 1
                            } else {
                                rng.random(..M::MOD)
                            }
                        })
                        .collect();
                    // Offset slices also exercise unaligned SIMD loads.
                    let (x, y) = (&x[1..], &y[1..]);
                    let a: Vec<_> = x.iter().map(|&x| MInt::<M>::from(x)).collect();
                    let b: Vec<_> = y.iter().map(|&y| MInt::<M>::from(y)).collect();
                    for reverse in [false, true] {
                        let expected: u128 = x
                            .iter()
                            .enumerate()
                            .map(|(i, &x)| {
                                x as u128 * y[if reverse { n - 1 - i } else { i }] as u128
                            })
                            .sum();
                        let scalar = if reverse {
                            dot_product_scalar::<M, true>(x, y)
                        } else {
                            dot_product_scalar::<M, false>(x, y)
                        };
                        assert_eq!(scalar, expected);
                        #[cfg(target_arch = "x86_64")]
                        {
                            if is_x86_feature_detected!("avx2") {
                                let sum = unsafe {
                                    if reverse {
                                        simd::dot_product_avx2::<true>(x, y)
                                    } else {
                                        simd::dot_product_avx2::<false>(x, y)
                                    }
                                };
                                assert_eq!(sum, expected);
                            }
                            if avx512_supported() {
                                let sum = unsafe {
                                    if reverse {
                                        simd::dot_product_avx512::<true>(x, y)
                                    } else {
                                        simd::dot_product_avx512::<false>(x, y)
                                    }
                                };
                                assert_eq!(sum, expected);
                            }
                        }
                        let value = if reverse {
                            M::dot_product_rev(&a, &b)
                        } else {
                            M::dot_product(&a, &b)
                        };
                        assert_eq!(value.inner() as u128, expected % M::MOD as u128);
                    }
                }
            }
        }

        enum Modulo3 {}
        impl MontgomeryReduction32 for Modulo3 {
            const MOD: u32 = 3;
        }
        enum Modulo2147483647 {}
        impl MontgomeryReduction32 for Modulo2147483647 {
            const MOD: u32 = 2_147_483_647;
        }
        check::<Modulo3>();
        check::<Modulo167772161>();
        check::<Modulo998244353>();
        check::<Modulo2147483647>();
    }
}
