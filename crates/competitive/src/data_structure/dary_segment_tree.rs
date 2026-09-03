#[cfg(target_arch = "x86_64")]
use super::simd;
use super::{RangeBoundsExt, SimdBackend, simd_backend};
use std::ops::RangeBounds;

#[repr(C, align(64))]
#[derive(Clone, Debug)]
struct Block<T, const B: usize>([T; B]);

macro_rules! define_dary_segment_tree {
    (
        $name:ident,
        $doc:literal,
        $value:ty,
        $branch:expr,
        $unit:expr,
        $operation:ident,
        $backend:expr,
        $simd_set:literal,
        $reduce_avx2:ident,
        $reduce_range_avx2:ident,
        $reduce_avx512:ident,
        $reduce_range_avx512:ident
    ) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name {
            levels: Vec<Vec<Block<$value, $branch>>>,
            len: usize,
            #[cfg(target_arch = "x86_64")]
            backend: SimdBackend,
        }

        impl $name {
            pub fn new(len: usize) -> Self {
                Self::from_vec(vec![$unit; len])
            }

            pub fn from_vec(values: Vec<$value>) -> Self {
                Self::build(values, $backend)
            }

            #[inline]
            pub fn len(&self) -> usize {
                self.len
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.len == 0
            }

            #[inline]
            pub fn set(&mut self, index: usize, value: $value) {
                assert!(index < self.len);
                self.set_value(index, value);
            }

            #[inline]
            pub fn clear(&mut self, index: usize) {
                self.set(index, $unit);
            }

            #[inline]
            pub fn update(&mut self, index: usize, value: $value) {
                assert!(index < self.len);
                let current = self.levels[0][index / $branch].0[index % $branch];
                self.set_value(index, current.$operation(value));
            }

            #[inline]
            pub fn get(&self, index: usize) -> $value {
                assert!(index < self.len);
                self.levels[0][index / $branch].0[index % $branch]
            }

            #[inline]
            pub fn fold<R>(&self, range: R) -> $value
            where
                R: RangeBounds<usize>,
            {
                let range = range.to_range_bounded(0, self.len).expect("invalid range");
                #[cfg(target_arch = "x86_64")]
                return match self.backend {
                    SimdBackend::Scalar => {
                        self.fold_by(range.start, range.end, Self::reduce_range_scalar)
                    }
                    // SAFETY: construction fixes every block width and selects a supported backend.
                    SimdBackend::Avx2 => unsafe { self.fold_avx2(range.start, range.end) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.fold_avx512(range.start, range.end) },
                };
                #[cfg(not(target_arch = "x86_64"))]
                self.fold_by(range.start, range.end, Self::reduce_range_scalar)
            }

            #[inline]
            pub fn fold_all(&self) -> $value {
                self.levels.last().unwrap()[0].0[0]
            }

            fn build(values: Vec<$value>, backend: SimdBackend) -> Self {
                let _ = &backend;
                let len = values.len();
                let mut current = if values.is_empty() {
                    vec![$unit]
                } else {
                    values
                };
                let mut levels = Vec::new();
                loop {
                    let blocks: Vec<_> = current
                        .chunks($branch)
                        .map(|chunk| {
                            let mut values = [$unit; $branch];
                            values[..chunk.len()].copy_from_slice(chunk);
                            Block(values)
                        })
                        .collect();
                    if current.len() == 1 {
                        levels.push(blocks);
                        break;
                    }
                    current = blocks
                        .iter()
                        .map(|block| Self::reduce_scalar(&block.0))
                        .collect();
                    levels.push(blocks);
                }
                Self {
                    levels,
                    len,
                    #[cfg(target_arch = "x86_64")]
                    backend,
                }
            }

            #[inline]
            fn set_value(&mut self, index: usize, value: $value) {
                if self.levels[0][index / $branch].0[index % $branch] == value {
                    return;
                }
                if !$simd_set {
                    self.set_by(index, value, Self::reduce_scalar);
                    return;
                }
                #[cfg(target_arch = "x86_64")]
                match self.backend {
                    SimdBackend::Scalar => self.set_by(index, value, Self::reduce_scalar),
                    // SAFETY: construction selects a supported backend.
                    SimdBackend::Avx2 => unsafe { self.set_avx2(index, value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.set_avx512(index, value) },
                }
                #[cfg(not(target_arch = "x86_64"))]
                self.set_by(index, value, Self::reduce_scalar);
            }

            #[inline(always)]
            fn reduce_scalar(values: &[$value; $branch]) -> $value {
                let mut result = values[0];
                for &value in &values[1..] {
                    result = result.$operation(value);
                }
                result
            }

            #[inline(always)]
            fn reduce_range_scalar(values: &[$value; $branch], start: usize, end: usize) -> $value {
                values[start..end]
                    .iter()
                    .copied()
                    .reduce(<$value>::$operation)
                    .unwrap_or($unit)
            }

            #[inline(always)]
            fn set_by<F>(&mut self, mut index: usize, value: $value, mut reduce: F)
            where
                F: FnMut(&[$value; $branch]) -> $value,
            {
                self.levels[0][index / $branch].0[index % $branch] = value;
                for level in 0..self.levels.len() - 1 {
                    let block = index / $branch;
                    let aggregate = reduce(&self.levels[level][block].0);
                    index = block;
                    let parent = &mut self.levels[level + 1][index / $branch].0[index % $branch];
                    if *parent == aggregate {
                        break;
                    }
                    *parent = aggregate;
                }
            }

            #[inline(always)]
            fn fold_by<F>(&self, mut left: usize, mut right: usize, mut reduce: F) -> $value
            where
                F: FnMut(&[$value; $branch], usize, usize) -> $value,
            {
                let mut result: $value = $unit;
                for level in &self.levels {
                    if left >= right {
                        break;
                    }
                    let first = left / $branch;
                    let last = (right - 1) / $branch;
                    if first == last {
                        return result.$operation(reduce(
                            &level[first].0,
                            left % $branch,
                            (right - 1) % $branch + 1,
                        ));
                    }
                    if left % $branch != 0 {
                        result =
                            result.$operation(reduce(&level[first].0, left % $branch, $branch));
                        left = (first + 1) * $branch;
                    }
                    if right % $branch != 0 {
                        result = result.$operation(reduce(&level[last].0, 0, right % $branch));
                        right = last * $branch;
                    }
                    left /= $branch;
                    right /= $branch;
                }
                result
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn set_avx2(&mut self, index: usize, value: $value) {
                self.set_by(index, value, |values| unsafe { simd::$reduce_avx2(values) });
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn fold_avx2(&self, left: usize, right: usize) -> $value {
                self.fold_by(left, right, |values, start, end| unsafe {
                    simd::$reduce_range_avx2(values, start, end)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx512f")]
            unsafe fn set_avx512(&mut self, index: usize, value: $value) {
                self.set_by(index, value, |values| unsafe {
                    simd::$reduce_avx512(values)
                });
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx512f")]
            unsafe fn fold_avx512(&self, left: usize, right: usize) -> $value {
                self.fold_by(left, right, |values, start, end| unsafe {
                    simd::$reduce_range_avx512(values, start, end)
                })
            }
        }
    };
}

define_dary_segment_tree!(
    DarySegmentTreeMinI32,
    "A cache-line-oriented d-ary point-update segment tree for range minima over `i32`.",
    i32,
    16,
    i32::MAX,
    min,
    simd_backend(),
    true,
    minimum_i32x16_avx2,
    minimum_range_i32x16_avx2,
    minimum_i32x16_avx512,
    minimum_range_i32x16_avx512
);
define_dary_segment_tree!(
    DarySegmentTreeMaxI32,
    "A cache-line-oriented d-ary point-update segment tree for range maxima over `i32`.",
    i32,
    16,
    i32::MIN,
    max,
    simd_backend(),
    true,
    maximum_i32x16_avx2,
    maximum_range_i32x16_avx2,
    maximum_i32x16_avx512,
    maximum_range_i32x16_avx512
);
define_dary_segment_tree!(
    DarySegmentTreeMinI64,
    "A cache-line-oriented d-ary point-update segment tree for range minima over `i64`.",
    i64,
    8,
    i64::MAX,
    min,
    simd_backend(),
    true,
    minimum_i64x8_avx2,
    minimum_range_i64x8_avx2,
    minimum_i64x8_avx512,
    minimum_range_i64x8_avx512
);
define_dary_segment_tree!(
    DarySegmentTreeMaxI64,
    "A cache-line-oriented d-ary point-update segment tree for range maxima over `i64`.",
    i64,
    8,
    i64::MIN,
    max,
    simd_backend(),
    true,
    maximum_i64x8_avx2,
    maximum_range_i64x8_avx2,
    maximum_i64x8_avx512,
    maximum_range_i64x8_avx512
);
define_dary_segment_tree!(
    DarySegmentTreeAddI32,
    "A cache-line-oriented d-ary point-update segment tree for wrapping range sums over `i32`.",
    i32,
    16,
    0,
    wrapping_add,
    simd_backend(),
    false,
    sum_i32x16_avx2,
    sum_range_i32x16_avx2,
    sum_i32x16_avx512,
    sum_range_i32x16_avx512
);
define_dary_segment_tree!(
    DarySegmentTreeAddI64,
    "A cache-line-oriented d-ary point-update segment tree for wrapping range sums over `i64`.",
    i64,
    8,
    0,
    wrapping_add,
    simd_backend(),
    false,
    sum_i64x8_avx2,
    sum_range_i64x8_avx2,
    sum_i64x8_avx512,
    sum_range_i64x8_avx512
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    #[cfg(target_arch = "x86_64")]
    use crate::tools::avx512_supported;

    #[cfg(target_arch = "x86_64")]
    fn backends() -> Vec<SimdBackend> {
        let mut result = vec![SimdBackend::Scalar];
        if is_x86_feature_detected!("avx2") {
            result.push(SimdBackend::Avx2);
        }
        if avx512_supported() {
            result.push(SimdBackend::Avx512);
        }
        result
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn backends() -> Vec<SimdBackend> {
        vec![SimdBackend::Scalar]
    }

    #[test]
    fn test_dary_segment_tree() {
        let mut rng = Xorshift::default();
        macro_rules! check {
            ($value:ty, $minimum:ty, $maximum:ty, $sum:ty) => {{
                for len in [
                    0, 1, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 255, 256, 257, 4097,
                ] {
                    let mut values: Vec<$value> =
                        (0..len).map(|_| rng.rand64() as $value).collect();
                    if let Some(value) = values.get_mut(0) {
                        *value = <$value>::MIN;
                    }
                    if let Some(value) = values.get_mut(1) {
                        *value = 0;
                    }
                    if let Some(value) = values.get_mut(2) {
                        *value = <$value>::MAX;
                    }
                    for backend in backends() {
                        let mut minimum = <$minimum>::build(values.clone(), backend);
                        let mut maximum = <$maximum>::build(values.clone(), backend);
                        let mut sum = <$sum>::build(values.clone(), backend);
                        let mut expected_minimum = values.clone();
                        let mut expected_maximum = values.clone();
                        let mut expected_sum = values.clone();
                        assert_eq!(minimum.len(), len);
                        assert_eq!(maximum.len(), len);
                        assert_eq!(sum.len(), len);
                        assert_eq!(minimum.is_empty(), len == 0);
                        assert_eq!(maximum.is_empty(), len == 0);
                        assert_eq!(sum.is_empty(), len == 0);
                        for _ in 0..500 {
                            if len != 0 {
                                let index = rng.rand(len as u64) as usize;
                                let value = rng.rand64() as $value;
                                match rng.rand(5) {
                                    0 => {
                                        minimum.set(index, value);
                                        maximum.set(index, value);
                                        sum.set(index, value);
                                        expected_minimum[index] = value;
                                        expected_maximum[index] = value;
                                        expected_sum[index] = value;
                                    }
                                    1 => {
                                        minimum.update(index, value);
                                        maximum.update(index, value);
                                        sum.update(index, value);
                                        expected_minimum[index] =
                                            expected_minimum[index].min(value);
                                        expected_maximum[index] =
                                            expected_maximum[index].max(value);
                                        expected_sum[index] =
                                            expected_sum[index].wrapping_add(value);
                                    }
                                    2 => {
                                        minimum.clear(index);
                                        maximum.clear(index);
                                        sum.clear(index);
                                        expected_minimum[index] = <$value>::MAX;
                                        expected_maximum[index] = <$value>::MIN;
                                        expected_sum[index] = 0;
                                    }
                                    3 => {
                                        assert_eq!(minimum.get(index), expected_minimum[index]);
                                        assert_eq!(maximum.get(index), expected_maximum[index]);
                                        assert_eq!(sum.get(index), expected_sum[index]);
                                    }
                                    _ => {
                                        let left = rng.rand(len as u64 + 1) as usize;
                                        let right =
                                            left + rng.rand((len - left) as u64 + 1) as usize;
                                        assert_eq!(
                                            minimum.fold(left..right),
                                            expected_minimum[left..right]
                                                .iter()
                                                .copied()
                                                .min()
                                                .unwrap_or(<$value>::MAX)
                                        );
                                        assert_eq!(
                                            maximum.fold(left..right),
                                            expected_maximum[left..right]
                                                .iter()
                                                .copied()
                                                .max()
                                                .unwrap_or(<$value>::MIN)
                                        );
                                        assert_eq!(
                                            sum.fold(left..right),
                                            expected_sum[left..right]
                                                .iter()
                                                .copied()
                                                .fold(0, <$value>::wrapping_add)
                                        );
                                    }
                                }
                            }
                            assert_eq!(
                                minimum.fold_all(),
                                expected_minimum
                                    .iter()
                                    .copied()
                                    .min()
                                    .unwrap_or(<$value>::MAX)
                            );
                            assert_eq!(
                                maximum.fold_all(),
                                expected_maximum
                                    .iter()
                                    .copied()
                                    .max()
                                    .unwrap_or(<$value>::MIN)
                            );
                            assert_eq!(
                                sum.fold_all(),
                                expected_sum.iter().copied().fold(0, <$value>::wrapping_add)
                            );
                        }
                    }
                }
            }};
        }

        check!(
            i32,
            DarySegmentTreeMinI32,
            DarySegmentTreeMaxI32,
            DarySegmentTreeAddI32
        );
        check!(
            i64,
            DarySegmentTreeMinI64,
            DarySegmentTreeMaxI64,
            DarySegmentTreeAddI64
        );
    }
}
