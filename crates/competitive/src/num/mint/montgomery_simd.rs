#![allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics and raw pointers are confined here
use std::arch::x86_64::*;

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn montgomery_mul_256(
    a: __m256i,
    b: __m256i,
    r_vec: __m256i,
    mod_vec: __m256i,
) -> __m256i {
    let a13 = _mm256_bsrli_epi128::<4>(a);
    let b13 = _mm256_bsrli_epi128::<4>(b);
    let t02 = _mm256_mul_epu32(a, b);
    let t13 = _mm256_mul_epu32(a13, b13);
    let m02 = _mm256_mul_epu32(t02, r_vec);
    let m13 = _mm256_mul_epu32(t13, r_vec);
    let u02 = _mm256_add_epi64(t02, _mm256_mul_epu32(m02, mod_vec));
    let u13 = _mm256_add_epi64(t13, _mm256_mul_epu32(m13, mod_vec));
    _mm256_or_si256(_mm256_bsrli_epi128::<4>(u02), u13)
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn montgomery_mul_256_fixed(
    a: __m256i,
    b: __m256i,
    b_r: __m256i,
    mod_vec: __m256i,
) -> __m256i {
    let a13 = _mm256_bsrli_epi128::<4>(a);
    let t02 = _mm256_mul_epu32(a, b);
    let t13 = _mm256_mul_epu32(a13, b);
    let m02 = _mm256_mul_epu32(a, b_r);
    let m13 = _mm256_mul_epu32(a13, b_r);
    let u02 = _mm256_add_epi64(t02, _mm256_mul_epu32(m02, mod_vec));
    let u13 = _mm256_add_epi64(t13, _mm256_mul_epu32(m13, mod_vec));
    _mm256_or_si256(_mm256_bsrli_epi128::<4>(u02), u13)
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn add_mod_256(a: __m256i, b: __m256i, mod_vec: __m256i) -> __m256i {
    let sum = _mm256_add_epi32(a, b);
    _mm256_min_epu32(sum, _mm256_sub_epi32(sum, mod_vec))
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn sub_mod_256(a: __m256i, b: __m256i, mod_vec: __m256i) -> __m256i {
    let diff = _mm256_sub_epi32(_mm256_add_epi32(a, mod_vec), b);
    _mm256_min_epu32(diff, _mm256_sub_epi32(diff, mod_vec))
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn montgomery_mul_256_canon(
    a: __m256i,
    b: __m256i,
    r_vec: __m256i,
    mod_vec: __m256i,
) -> __m256i {
    let x = montgomery_mul_256(a, b, r_vec, mod_vec);
    _mm256_min_epu32(x, _mm256_sub_epi32(x, mod_vec))
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn montgomery_add_256(a: __m256i, b: __m256i, mod2_vec: __m256i) -> __m256i {
    let sum = _mm256_add_epi32(a, b);
    _mm256_min_epu32(sum, _mm256_sub_epi32(sum, mod2_vec))
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn montgomery_sub_256(a: __m256i, b: __m256i, mod2_vec: __m256i) -> __m256i {
    let diff = _mm256_sub_epi32(_mm256_add_epi32(a, mod2_vec), b);
    _mm256_min_epu32(diff, _mm256_sub_epi32(diff, mod2_vec))
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
pub unsafe fn montgomery_mul_512(
    a: __m512i,
    b: __m512i,
    r_vec: __m512i,
    mod_vec: __m512i,
) -> __m512i {
    let a13 = _mm512_srli_epi64::<32>(a);
    let b13 = _mm512_srli_epi64::<32>(b);
    let t02 = _mm512_mul_epu32(a, b);
    let t13 = _mm512_mul_epu32(a13, b13);
    let m02 = _mm512_mul_epu32(t02, r_vec);
    let m13 = _mm512_mul_epu32(t13, r_vec);
    let u02 = _mm512_add_epi64(t02, _mm512_mul_epu32(m02, mod_vec));
    let u13 = _mm512_add_epi64(t13, _mm512_mul_epu32(m13, mod_vec));
    _mm512_or_si512(_mm512_srli_epi64::<32>(u02), u13)
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
pub unsafe fn add_mod_512(a: __m512i, b: __m512i, mod_vec: __m512i) -> __m512i {
    let sum = _mm512_add_epi32(a, b);
    _mm512_min_epu32(sum, _mm512_sub_epi32(sum, mod_vec))
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
pub unsafe fn sub_mod_512(a: __m512i, b: __m512i, mod_vec: __m512i) -> __m512i {
    let diff = _mm512_sub_epi32(_mm512_add_epi32(a, mod_vec), b);
    _mm512_min_epu32(diff, _mm512_sub_epi32(diff, mod_vec))
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
pub unsafe fn montgomery_mul_512_canon(
    a: __m512i,
    b: __m512i,
    r_vec: __m512i,
    mod_vec: __m512i,
) -> __m512i {
    let x = montgomery_mul_512(a, b, r_vec, mod_vec);
    _mm512_min_epu32(x, _mm512_sub_epi32(x, mod_vec))
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
pub unsafe fn montgomery_add_512(a: __m512i, b: __m512i, mod2_vec: __m512i) -> __m512i {
    let sum = _mm512_add_epi32(a, b);
    _mm512_min_epu32(sum, _mm512_sub_epi32(sum, mod2_vec))
}

#[inline]
#[target_feature(enable = "avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]
pub unsafe fn montgomery_sub_512(a: __m512i, b: __m512i, mod2_vec: __m512i) -> __m512i {
    let diff = _mm512_sub_epi32(_mm512_add_epi32(a, mod2_vec), b);
    _mm512_min_epu32(diff, _mm512_sub_epi32(diff, mod2_vec))
}
