#![allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics are confined to feature-gated functions.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn first_ge_u16x32_avx2(values: &[u16; 32], key: u16) -> usize {
    let sign = _mm256_set1_epi16(i16::MIN);
    let key = _mm256_xor_si256(_mm256_set1_epi16(key as i16), sign);
    let low = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()), sign);
    let high = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(16).cast()), sign);
    let low = _mm256_movemask_epi8(_mm256_cmpgt_epi16(key, low)) as u32 as u64;
    let high = _mm256_movemask_epi8(_mm256_cmpgt_epi16(key, high)) as u32 as u64;
    (!(low | high << 32)).trailing_zeros() as usize / 2
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn first_gt_u16x32_avx2(values: &[u16; 32], key: u16) -> usize {
    let sign = _mm256_set1_epi16(i16::MIN);
    let key = _mm256_xor_si256(_mm256_set1_epi16(key as i16), sign);
    let low = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()), sign);
    let high = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(16).cast()), sign);
    let low = _mm256_movemask_epi8(_mm256_cmpgt_epi16(low, key)) as u32 as u64;
    let high = _mm256_movemask_epi8(_mm256_cmpgt_epi16(high, key)) as u32 as u64;
    (low | high << 32).trailing_zeros() as usize / 2
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn first_ge_u32x16_avx2(values: &[u32; 16], key: u32) -> usize {
    let sign = _mm256_set1_epi32(i32::MIN);
    let key = _mm256_xor_si256(_mm256_set1_epi32(key as i32), sign);
    let low = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()), sign);
    let high = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(8).cast()), sign);
    let low = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(key, low)));
    let high = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(key, high)));
    let mask = ((!low as u32) & 0xff) | (((!high as u32) & 0xff) << 8);
    (mask as u16).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn first_gt_u32x16_avx2(values: &[u32; 16], key: u32) -> usize {
    let sign = _mm256_set1_epi32(i32::MIN);
    let key = _mm256_xor_si256(_mm256_set1_epi32(key as i32), sign);
    let low = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()), sign);
    let high = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(8).cast()), sign);
    let low = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(low, key)));
    let high = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(high, key)));
    let mask = low as u32 | ((high as u32) << 8);
    (mask as u16).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn first_ge_u64x8_avx2(values: &[u64; 8], key: u64) -> usize {
    let sign = _mm256_set1_epi64x(i64::MIN);
    let key = _mm256_xor_si256(_mm256_set1_epi64x(key as i64), sign);
    let low = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()), sign);
    let high = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(4).cast()), sign);
    let low = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(key, low)));
    let high = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(key, high)));
    let mask = ((!low as u32) & 0x0f) | (((!high as u32) & 0x0f) << 4);
    (mask as u8).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn first_gt_u64x8_avx2(values: &[u64; 8], key: u64) -> usize {
    let sign = _mm256_set1_epi64x(i64::MIN);
    let key = _mm256_xor_si256(_mm256_set1_epi64x(key as i64), sign);
    let low = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()), sign);
    let high = _mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(4).cast()), sign);
    let low = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(low, key)));
    let high = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(high, key)));
    let mask = low as u32 | ((high as u32) << 4);
    (mask as u8).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
#[inline]
pub unsafe fn first_ge_u16x32_avx512(values: &[u16; 32], key: u16) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let key = _mm512_set1_epi16(key as i16);
    (!_mm512_cmplt_epu16_mask(values, key)).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
#[inline]
pub unsafe fn first_gt_u16x32_avx512(values: &[u16; 32], key: u16) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let key = _mm512_set1_epi16(key as i16);
    _mm512_cmpgt_epu16_mask(values, key).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn first_ge_u32x16_avx512(values: &[u32; 16], key: u32) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let key = _mm512_set1_epi32(key as i32);
    (!_mm512_cmplt_epu32_mask(values, key)).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn first_gt_u32x16_avx512(values: &[u32; 16], key: u32) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let key = _mm512_set1_epi32(key as i32);
    _mm512_cmpgt_epu32_mask(values, key).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn first_ge_u64x8_avx512(values: &[u64; 8], key: u64) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let key = _mm512_set1_epi64(key as i64);
    (!_mm512_cmplt_epu64_mask(values, key)).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn first_gt_u64x8_avx512(values: &[u64; 8], key: u64) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let key = _mm512_set1_epi64(key as i64);
    _mm512_cmpgt_epu64_mask(values, key).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn add_suffix_u32x16_avx2(values: &mut [u32; 16], index: usize, delta: u32) {
    let index = _mm256_set1_epi32(index as i32 - 1);
    let delta = _mm256_set1_epi32(delta as i32);
    let low_mask = _mm256_cmpgt_epi32(_mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7), index);
    let high_mask = _mm256_cmpgt_epi32(_mm256_setr_epi32(8, 9, 10, 11, 12, 13, 14, 15), index);
    let low = _mm256_add_epi32(
        _mm256_loadu_si256(values.as_ptr().cast()),
        _mm256_and_si256(delta, low_mask),
    );
    let high = _mm256_add_epi32(
        _mm256_loadu_si256(values.as_ptr().add(8).cast()),
        _mm256_and_si256(delta, high_mask),
    );
    _mm256_storeu_si256(values.as_mut_ptr().cast(), low);
    _mm256_storeu_si256(values.as_mut_ptr().add(8).cast(), high);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn add_suffix_u64x8_avx2(values: &mut [u64; 8], index: usize, delta: u64) {
    let index = _mm256_set1_epi64x(index as i64 - 1);
    let delta = _mm256_set1_epi64x(delta as i64);
    let low_mask = _mm256_cmpgt_epi64(_mm256_setr_epi64x(0, 1, 2, 3), index);
    let high_mask = _mm256_cmpgt_epi64(_mm256_setr_epi64x(4, 5, 6, 7), index);
    let low = _mm256_add_epi64(
        _mm256_loadu_si256(values.as_ptr().cast()),
        _mm256_and_si256(delta, low_mask),
    );
    let high = _mm256_add_epi64(
        _mm256_loadu_si256(values.as_ptr().add(4).cast()),
        _mm256_and_si256(delta, high_mask),
    );
    _mm256_storeu_si256(values.as_mut_ptr().cast(), low);
    _mm256_storeu_si256(values.as_mut_ptr().add(4).cast(), high);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn add_suffix_u32x16_avx512(values: &mut [u32; 16], index: usize, delta: u32) {
    let values_vector = _mm512_loadu_si512(values.as_ptr().cast());
    let values_vector = _mm512_mask_add_epi32(
        values_vector,
        u16::MAX << index,
        values_vector,
        _mm512_set1_epi32(delta as i32),
    );
    _mm512_storeu_si512(values.as_mut_ptr().cast(), values_vector);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn add_suffix_u64x8_avx512(values: &mut [u64; 8], index: usize, delta: u64) {
    let values_vector = _mm512_loadu_si512(values.as_ptr().cast());
    let values_vector = _mm512_mask_add_epi64(
        values_vector,
        u8::MAX << index,
        values_vector,
        _mm512_set1_epi64(delta as i64),
    );
    _mm512_storeu_si512(values.as_mut_ptr().cast(), values_vector);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn max_index_u32x16_avx2(values: &[u32; 16]) -> usize {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(8).cast());
    let mut maximum = _mm256_max_epu32(low, high);
    maximum = _mm256_max_epu32(maximum, _mm256_permute2x128_si256::<0x01>(maximum, maximum));
    maximum = _mm256_max_epu32(maximum, _mm256_shuffle_epi32::<0x4e>(maximum));
    maximum = _mm256_max_epu32(maximum, _mm256_shuffle_epi32::<0xb1>(maximum));
    let maximum = _mm256_set1_epi32(_mm256_extract_epi32::<0>(maximum));
    let low = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpeq_epi32(low, maximum)));
    let high = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpeq_epi32(high, maximum)));
    ((low as u32 | ((high as u32) << 8)) as u16).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn max_index_u32x16_avx512(values: &[u32; 16]) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let maximum = _mm512_set1_epi32(_mm512_reduce_max_epu32(values) as i32);
    _mm512_cmpeq_epi32_mask(values, maximum).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn max_epu64(left: __m256i, right: __m256i) -> __m256i {
    let sign = _mm256_set1_epi64x(i64::MIN);
    let greater = _mm256_cmpgt_epi64(_mm256_xor_si256(left, sign), _mm256_xor_si256(right, sign));
    _mm256_blendv_epi8(right, left, greater)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn max_index_u64x8_avx2(values: &[u64; 8]) -> usize {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(4).cast());
    let mut maximum = max_epu64(low, high);
    maximum = max_epu64(maximum, _mm256_permute4x64_epi64::<0x4e>(maximum));
    maximum = max_epu64(maximum, _mm256_permute4x64_epi64::<0xb1>(maximum));
    let maximum = _mm256_set1_epi64x(_mm256_extract_epi64::<0>(maximum));
    let low = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpeq_epi64(low, maximum)));
    let high = _mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpeq_epi64(high, maximum)));
    ((low as u32 | ((high as u32) << 4)) as u8).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn max_index_u64x8_avx512(values: &[u64; 8]) -> usize {
    let values = _mm512_loadu_si512(values.as_ptr().cast());
    let maximum = _mm512_set1_epi64(_mm512_reduce_max_epu64(values) as i64);
    _mm512_cmpeq_epi64_mask(values, maximum).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn max_index_u128x4_avx2(low: &[u64; 4], high: &[u64; 4]) -> usize {
    let low = _mm256_loadu_si256(low.as_ptr().cast());
    let high = _mm256_loadu_si256(high.as_ptr().cast());
    let mut maximum_high = high;
    maximum_high = max_epu64(maximum_high, _mm256_permute4x64_epi64::<0x4e>(maximum_high));
    maximum_high = max_epu64(maximum_high, _mm256_permute4x64_epi64::<0xb1>(maximum_high));
    let high_equal = _mm256_cmpeq_epi64(high, maximum_high);
    let high_mask = _mm256_movemask_pd(_mm256_castsi256_pd(high_equal)) as u32 & 15;
    if high_mask.is_power_of_two() {
        return high_mask.trailing_zeros() as usize;
    }
    let mut maximum_low = _mm256_and_si256(low, high_equal);
    maximum_low = max_epu64(maximum_low, _mm256_permute4x64_epi64::<0x4e>(maximum_low));
    maximum_low = max_epu64(maximum_low, _mm256_permute4x64_epi64::<0xb1>(maximum_low));
    let both = _mm256_and_si256(high_equal, _mm256_cmpeq_epi64(low, maximum_low));
    (_mm256_movemask_pd(_mm256_castsi256_pd(both)) as u32 & 15).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,avx512f,avx512vl")]
#[inline]
pub unsafe fn max_index_u128x4_avx512(low: &[u64; 4], high: &[u64; 4]) -> usize {
    let low = _mm256_loadu_si256(low.as_ptr().cast());
    let high = _mm256_loadu_si256(high.as_ptr().cast());
    let mut maximum_high = high;
    maximum_high = _mm256_max_epu64(maximum_high, _mm256_permute4x64_epi64::<0x4e>(maximum_high));
    maximum_high = _mm256_max_epu64(maximum_high, _mm256_permute4x64_epi64::<0xb1>(maximum_high));
    let high_equal = _mm256_cmpeq_epi64(high, maximum_high);
    let high_mask = _mm256_movemask_pd(_mm256_castsi256_pd(high_equal)) as u32 & 15;
    if high_mask.is_power_of_two() {
        return high_mask.trailing_zeros() as usize;
    }
    let mut maximum_low = _mm256_and_si256(low, high_equal);
    maximum_low = _mm256_max_epu64(maximum_low, _mm256_permute4x64_epi64::<0x4e>(maximum_low));
    maximum_low = _mm256_max_epu64(maximum_low, _mm256_permute4x64_epi64::<0xb1>(maximum_low));
    let both = _mm256_and_si256(high_equal, _mm256_cmpeq_epi64(low, maximum_low));
    (_mm256_movemask_pd(_mm256_castsi256_pd(both)) as u32 & 15).trailing_zeros() as usize
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn reduce_min_i32x8(mut values: __m256i) -> i32 {
    values = _mm256_min_epi32(values, _mm256_permute2x128_si256::<0x01>(values, values));
    values = _mm256_min_epi32(values, _mm256_shuffle_epi32::<0x4e>(values));
    values = _mm256_min_epi32(values, _mm256_shuffle_epi32::<0xb1>(values));
    _mm256_extract_epi32::<0>(values)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn reduce_max_i32x8(mut values: __m256i) -> i32 {
    values = _mm256_max_epi32(values, _mm256_permute2x128_si256::<0x01>(values, values));
    values = _mm256_max_epi32(values, _mm256_shuffle_epi32::<0x4e>(values));
    values = _mm256_max_epi32(values, _mm256_shuffle_epi32::<0xb1>(values));
    _mm256_extract_epi32::<0>(values)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn reduce_sum_i32x8(mut values: __m256i) -> i32 {
    values = _mm256_add_epi32(values, _mm256_permute2x128_si256::<0x01>(values, values));
    values = _mm256_add_epi32(values, _mm256_shuffle_epi32::<0x4e>(values));
    values = _mm256_add_epi32(values, _mm256_shuffle_epi32::<0xb1>(values));
    _mm256_extract_epi32::<0>(values)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn minimum_i32x16_avx2(values: &[i32; 16]) -> i32 {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(8).cast());
    reduce_min_i32x8(_mm256_min_epi32(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn maximum_i32x16_avx2(values: &[i32; 16]) -> i32 {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(8).cast());
    reduce_max_i32x8(_mm256_max_epi32(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn sum_i32x16_avx2(values: &[i32; 16]) -> i32 {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(8).cast());
    reduce_sum_i32x8(_mm256_add_epi32(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn range_mask_i32x8(start: usize, end: usize, offset: i32) -> __m256i {
    let lanes = _mm256_setr_epi32(
        offset,
        offset + 1,
        offset + 2,
        offset + 3,
        offset + 4,
        offset + 5,
        offset + 6,
        offset + 7,
    );
    _mm256_and_si256(
        _mm256_cmpgt_epi32(lanes, _mm256_set1_epi32(start as i32 - 1)),
        _mm256_cmpgt_epi32(_mm256_set1_epi32(end as i32), lanes),
    )
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn minimum_range_i32x16_avx2(values: &[i32; 16], start: usize, end: usize) -> i32 {
    let unit = _mm256_set1_epi32(i32::MAX);
    let low = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().cast()),
        range_mask_i32x8(start, end, 0),
    );
    let high = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().add(8).cast()),
        range_mask_i32x8(start, end, 8),
    );
    reduce_min_i32x8(_mm256_min_epi32(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn maximum_range_i32x16_avx2(values: &[i32; 16], start: usize, end: usize) -> i32 {
    let unit = _mm256_set1_epi32(i32::MIN);
    let low = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().cast()),
        range_mask_i32x8(start, end, 0),
    );
    let high = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().add(8).cast()),
        range_mask_i32x8(start, end, 8),
    );
    reduce_max_i32x8(_mm256_max_epi32(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn sum_range_i32x16_avx2(values: &[i32; 16], start: usize, end: usize) -> i32 {
    let low = _mm256_and_si256(
        _mm256_loadu_si256(values.as_ptr().cast()),
        range_mask_i32x8(start, end, 0),
    );
    let high = _mm256_and_si256(
        _mm256_loadu_si256(values.as_ptr().add(8).cast()),
        range_mask_i32x8(start, end, 8),
    );
    reduce_sum_i32x8(_mm256_add_epi32(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn minimum_i32x16_avx512(values: &[i32; 16]) -> i32 {
    _mm512_reduce_min_epi32(_mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn maximum_i32x16_avx512(values: &[i32; 16]) -> i32 {
    _mm512_reduce_max_epi32(_mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn sum_i32x16_avx512(values: &[i32; 16]) -> i32 {
    _mm512_reduce_add_epi32(_mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn minimum_range_i32x16_avx512(values: &[i32; 16], start: usize, end: usize) -> i32 {
    let mask = (u16::MAX << start) & (u16::MAX >> (16 - end));
    _mm512_mask_reduce_min_epi32(mask, _mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn maximum_range_i32x16_avx512(values: &[i32; 16], start: usize, end: usize) -> i32 {
    let mask = (u16::MAX << start) & (u16::MAX >> (16 - end));
    _mm512_mask_reduce_max_epi32(mask, _mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn sum_range_i32x16_avx512(values: &[i32; 16], start: usize, end: usize) -> i32 {
    let mask = (u16::MAX << start) & (u16::MAX >> (16 - end));
    _mm512_mask_reduce_add_epi32(mask, _mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn min_i64x4(left: __m256i, right: __m256i) -> __m256i {
    _mm256_blendv_epi8(left, right, _mm256_cmpgt_epi64(left, right))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn max_i64x4(left: __m256i, right: __m256i) -> __m256i {
    _mm256_blendv_epi8(right, left, _mm256_cmpgt_epi64(left, right))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn reduce_min_i64x4(mut values: __m256i) -> i64 {
    values = min_i64x4(values, _mm256_permute4x64_epi64::<0x4e>(values));
    values = min_i64x4(values, _mm256_permute4x64_epi64::<0xb1>(values));
    _mm256_extract_epi64::<0>(values)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn reduce_max_i64x4(mut values: __m256i) -> i64 {
    values = max_i64x4(values, _mm256_permute4x64_epi64::<0x4e>(values));
    values = max_i64x4(values, _mm256_permute4x64_epi64::<0xb1>(values));
    _mm256_extract_epi64::<0>(values)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn reduce_sum_i64x4(mut values: __m256i) -> i64 {
    values = _mm256_add_epi64(values, _mm256_permute4x64_epi64::<0x4e>(values));
    values = _mm256_add_epi64(values, _mm256_permute4x64_epi64::<0xb1>(values));
    _mm256_extract_epi64::<0>(values)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn minimum_i64x8_avx2(values: &[i64; 8]) -> i64 {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(4).cast());
    reduce_min_i64x4(min_i64x4(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn maximum_i64x8_avx2(values: &[i64; 8]) -> i64 {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(4).cast());
    reduce_max_i64x4(max_i64x4(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn sum_i64x8_avx2(values: &[i64; 8]) -> i64 {
    let low = _mm256_loadu_si256(values.as_ptr().cast());
    let high = _mm256_loadu_si256(values.as_ptr().add(4).cast());
    reduce_sum_i64x4(_mm256_add_epi64(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn range_mask_i64x4(start: usize, end: usize, offset: i64) -> __m256i {
    let lanes = _mm256_setr_epi64x(offset, offset + 1, offset + 2, offset + 3);
    _mm256_and_si256(
        _mm256_cmpgt_epi64(lanes, _mm256_set1_epi64x(start as i64 - 1)),
        _mm256_cmpgt_epi64(_mm256_set1_epi64x(end as i64), lanes),
    )
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn minimum_range_i64x8_avx2(values: &[i64; 8], start: usize, end: usize) -> i64 {
    let unit = _mm256_set1_epi64x(i64::MAX);
    let low = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().cast()),
        range_mask_i64x4(start, end, 0),
    );
    let high = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().add(4).cast()),
        range_mask_i64x4(start, end, 4),
    );
    reduce_min_i64x4(min_i64x4(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn maximum_range_i64x8_avx2(values: &[i64; 8], start: usize, end: usize) -> i64 {
    let unit = _mm256_set1_epi64x(i64::MIN);
    let low = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().cast()),
        range_mask_i64x4(start, end, 0),
    );
    let high = _mm256_blendv_epi8(
        unit,
        _mm256_loadu_si256(values.as_ptr().add(4).cast()),
        range_mask_i64x4(start, end, 4),
    );
    reduce_max_i64x4(max_i64x4(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn sum_range_i64x8_avx2(values: &[i64; 8], start: usize, end: usize) -> i64 {
    let low = _mm256_and_si256(
        _mm256_loadu_si256(values.as_ptr().cast()),
        range_mask_i64x4(start, end, 0),
    );
    let high = _mm256_and_si256(
        _mm256_loadu_si256(values.as_ptr().add(4).cast()),
        range_mask_i64x4(start, end, 4),
    );
    reduce_sum_i64x4(_mm256_add_epi64(low, high))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn minimum_i64x8_avx512(values: &[i64; 8]) -> i64 {
    _mm512_reduce_min_epi64(_mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn maximum_i64x8_avx512(values: &[i64; 8]) -> i64 {
    _mm512_reduce_max_epi64(_mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn sum_i64x8_avx512(values: &[i64; 8]) -> i64 {
    _mm512_reduce_add_epi64(_mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn minimum_range_i64x8_avx512(values: &[i64; 8], start: usize, end: usize) -> i64 {
    let mask = (u8::MAX << start) & (u8::MAX >> (8 - end));
    _mm512_mask_reduce_min_epi64(mask, _mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn maximum_range_i64x8_avx512(values: &[i64; 8], start: usize, end: usize) -> i64 {
    let mask = (u8::MAX << start) & (u8::MAX >> (8 - end));
    _mm512_mask_reduce_max_epi64(mask, _mm512_loadu_si512(values.as_ptr().cast()))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn sum_range_i64x8_avx512(values: &[i64; 8], start: usize, end: usize) -> i64 {
    let mask = (u8::MAX << start) & (u8::MAX >> (8 - end));
    _mm512_mask_reduce_add_epi64(mask, _mm512_loadu_si512(values.as_ptr().cast()))
}
