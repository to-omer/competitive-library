use super::{MInt, MIntBase, MIntDotProduct, montgomery::MontgomeryReduction32};
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
        assert_eq!(x.len(), y.len());
        // reduce() needs sum < modulus * 2^32 to return a canonical residue.
        let modulus = <Self as MontgomeryReduction32>::MOD as u64;
        let block = (((modulus << 32) - 1) / ((modulus - 1) * (modulus - 1))).min(16) as usize;
        let mut result = 0;
        for (x, y) in x.chunks(block).zip(y.chunks(block)) {
            // SAFETY: MInt is transparent over u32 and both slices have equal lengths.
            let sum = unsafe {
                let a = std::slice::from_raw_parts(x.as_ptr().cast::<u32>(), x.len());
                let b = std::slice::from_raw_parts(y.as_ptr().cast::<u32>(), y.len());
                a.iter().zip(b).map(|(&a, &b)| a as u64 * b as u64).sum()
            };
            result = Self::mod_add(result, Self::reduce(sum));
        }
        MInt::new_unchecked(result)
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

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics and raw pointers are confined here
mod simd {
    use super::super::montgomery_simd::{add_mod_256, add_mod_512};
    use super::{MInt, MontgomeryReduction32};
    use std::arch::x86_64::*;

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
