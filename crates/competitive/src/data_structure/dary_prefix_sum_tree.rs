#[cfg(target_arch = "x86_64")]
use super::simd;
use super::{SimdBackend, simd_backend};

#[repr(C, align(64))]
#[derive(Clone, Debug)]
struct PrefixBlock<T, const B: usize>([T; B]);

macro_rules! define_dary_prefix_sum_tree {
    (
        $name:ident,
        $value:ty,
        $branch:expr,
        $add_avx2:ident,
        $first_gt_avx2:ident,
        $add_avx512:ident,
        $first_gt_avx512:ident
    ) => {
        /// A cache-line-oriented d-ary tree for point updates, prefix sums, and prefix searches.
        ///
        /// `BinaryIndexedTree` has a denser layout and can be preferable for small workloads.
        /// Use this type for repeated updates and prefix searches, especially with SIMD support.
        #[derive(Clone, Debug)]
        pub struct $name {
            levels: Vec<Vec<PrefixBlock<$value, $branch>>>,
            len: usize,
            total: $value,
            partition_valid: bool,
            #[cfg(target_arch = "x86_64")]
            backend: SimdBackend,
        }

        impl $name {
            pub fn new(len: usize) -> Self {
                Self::zeroed(len, simd_backend())
            }

            pub fn from_slice(values: &[$value]) -> Self {
                Self::build(values, simd_backend())
            }

            #[inline]
            pub fn len(&self) -> usize {
                self.len
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.len == 0
            }

            /// Adds `value` at `index`. Arithmetic is wrapping.
            #[inline]
            pub fn update(&mut self, index: usize, value: $value) {
                assert!(index < self.len);
                self.add(index, value);
                self.partition_valid &= self.total.checked_add(value).is_some();
                self.total = self.total.wrapping_add(value);
            }

            /// Replaces the value at `index`. Arithmetic is wrapping.
            #[inline]
            pub fn set(&mut self, index: usize, value: $value) {
                let previous = self.get(index);
                self.add(index, value.wrapping_sub(previous));
                self.partition_valid &= self
                    .total
                    .checked_sub(previous)
                    .and_then(|total| total.checked_add(value))
                    .is_some();
                self.total = self.total.wrapping_sub(previous).wrapping_add(value);
            }

            /// Returns the wrapping sum of `0..end`.
            #[inline]
            pub fn accumulate0(&self, mut end: usize) -> $value {
                assert!(end <= self.len);
                if end == self.len {
                    return self.total;
                }
                let mut result: $value = 0;
                for level in &self.levels {
                    let block = end / $branch;
                    let lane = end % $branch;
                    if lane != 0 {
                        // SAFETY: `end < len`; mapping it upward remains within every level.
                        let value = unsafe { level.get_unchecked(block).0.get_unchecked(lane - 1) };
                        result = result.wrapping_add(*value);
                    }
                    end = block;
                }
                result
            }

            /// Returns the wrapping sum of `0..=index`.
            #[inline]
            pub fn accumulate(&self, index: usize) -> $value {
                self.accumulate0(index + 1)
            }

            /// Returns the wrapping sum of `left..right`.
            #[inline]
            pub fn fold(&self, left: usize, right: usize) -> $value {
                assert!(left <= right && right <= self.len);
                self.accumulate0(right).wrapping_sub(self.accumulate0(left))
            }

            #[inline]
            pub fn get(&self, index: usize) -> $value {
                self.fold(index, index + 1)
            }

            #[inline]
            pub fn fold_all(&self) -> $value {
                self.total
            }

            /// Returns the number of leading values whose inclusive prefix sum is at most `value`.
            ///
            /// # Panics
            ///
            /// Panics if a prefix sum has overflowed.
            #[inline]
            pub fn partition_point_acc(&self, value: $value) -> usize {
                assert!(self.partition_valid, "prefix sum overflowed");
                if value >= self.total {
                    return self.len;
                }
                #[cfg(target_arch = "x86_64")]
                return match self.backend {
                    SimdBackend::Scalar => self.partition_point_scalar(value),
                    // SAFETY: `simd_backend` only selects supported instruction sets. Tests and
                    // standalone benchmarks pass supported backends to the private constructor.
                    SimdBackend::Avx2 => unsafe { self.partition_point_avx2(value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.partition_point_avx512(value) },
                };
                #[cfg(not(target_arch = "x86_64"))]
                self.partition_point_scalar(value)
            }

            fn zeroed(len: usize, backend: SimdBackend) -> Self {
                let _ = &backend;
                let mut levels = Vec::new();
                let mut level_len = len;
                while level_len != 0 {
                    level_len = level_len.div_ceil($branch);
                    levels.push(vec![PrefixBlock([0; $branch]); level_len]);
                    if level_len == 1 {
                        break;
                    }
                }
                Self {
                    levels,
                    len,
                    total: 0,
                    partition_valid: true,
                    #[cfg(target_arch = "x86_64")]
                    backend,
                }
            }

            fn build(values: &[$value], backend: SimdBackend) -> Self {
                let _ = &backend;
                let mut levels = Vec::new();
                let mut partition_valid = true;
                let mut current = Vec::with_capacity(values.len().div_ceil($branch));
                let mut blocks = Vec::with_capacity(current.capacity());
                for chunk in values.chunks($branch) {
                    let mut prefix = [0; $branch];
                    let mut sum: $value = 0;
                    for (index, &value) in chunk.iter().enumerate() {
                        partition_valid &= sum.checked_add(value).is_some();
                        sum = sum.wrapping_add(value);
                        prefix[index] = sum;
                    }
                    prefix[chunk.len()..].fill(sum);
                    blocks.push(PrefixBlock(prefix));
                    current.push(sum);
                }
                if !blocks.is_empty() {
                    levels.push(blocks);
                }
                while current.len() > 1 {
                    let mut blocks = Vec::with_capacity(current.len().div_ceil($branch));
                    for chunk in current.chunks($branch) {
                        let mut prefix = [0; $branch];
                        let mut sum: $value = 0;
                        for (index, &value) in chunk.iter().enumerate() {
                            partition_valid &= sum.checked_add(value).is_some();
                            sum = sum.wrapping_add(value);
                            prefix[index] = sum;
                        }
                        prefix[chunk.len()..].fill(sum);
                        blocks.push(PrefixBlock(prefix));
                    }
                    current = blocks.iter().map(|block| block.0[$branch - 1]).collect();
                    levels.push(blocks);
                }
                Self {
                    levels,
                    len: values.len(),
                    total: current.first().copied().unwrap_or(0),
                    partition_valid,
                    #[cfg(target_arch = "x86_64")]
                    backend,
                }
            }

            #[inline]
            fn add(&mut self, index: usize, value: $value) {
                #[cfg(target_arch = "x86_64")]
                match self.backend {
                    SimdBackend::Scalar => self.add_scalar(index, value),
                    // SAFETY: `simd_backend` only selects supported instruction sets. Tests and
                    // standalone benchmarks pass supported backends to the private constructor.
                    SimdBackend::Avx2 => unsafe { self.add_avx2(index, value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.add_avx512(index, value) },
                }
                #[cfg(not(target_arch = "x86_64"))]
                self.add_scalar(index, value);
            }

            #[inline]
            fn add_scalar(&mut self, mut index: usize, value: $value) {
                for level in &mut self.levels {
                    let block = index / $branch;
                    let lane = index % $branch;
                    // SAFETY: the public methods validate the leaf index; construction fixes the
                    // mapping from every child block to its parent.
                    let prefix = &mut unsafe { level.get_unchecked_mut(block) }.0;
                    for prefix in &mut prefix[lane..] {
                        *prefix = prefix.wrapping_add(value);
                    }
                    index = block;
                }
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn add_avx2(&mut self, mut index: usize, value: $value) {
                for level in &mut self.levels {
                    let block = index / $branch;
                    let lane = index % $branch;
                    let prefix = &mut unsafe { level.get_unchecked_mut(block) }.0;
                    unsafe { simd::$add_avx2(prefix, lane, value) };
                    index = block;
                }
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx512f")]
            unsafe fn add_avx512(&mut self, mut index: usize, value: $value) {
                for level in &mut self.levels {
                    let block = index / $branch;
                    let lane = index % $branch;
                    let prefix = &mut unsafe { level.get_unchecked_mut(block) }.0;
                    unsafe { simd::$add_avx512(prefix, lane, value) };
                    index = block;
                }
            }

            #[inline(always)]
            fn partition_point_by<F>(&self, mut value: $value, mut first_gt: F) -> usize
            where
                F: FnMut(&[$value; $branch], $value) -> usize,
            {
                let mut node = 0;
                for level in self.levels.iter().rev() {
                    // SAFETY: `value < total` and non-overflowing prefixes select a real child at
                    // every level.
                    let prefix = &unsafe { level.get_unchecked(node) }.0;
                    let lane = first_gt(prefix, value);
                    if lane != 0 {
                        value = value.wrapping_sub(unsafe { *prefix.get_unchecked(lane - 1) });
                    }
                    node = node * $branch + lane;
                }
                node.min(self.len)
            }

            #[inline(always)]
            fn partition_point_scalar(&self, value: $value) -> usize {
                self.partition_point_by(value, |prefix, value| {
                    prefix.partition_point(|&sum| sum <= value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn partition_point_avx2(&self, value: $value) -> usize {
                self.partition_point_by(value, |prefix, value| unsafe {
                    simd::$first_gt_avx2(prefix, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx512f")]
            unsafe fn partition_point_avx512(&self, value: $value) -> usize {
                self.partition_point_by(value, |prefix, value| unsafe {
                    simd::$first_gt_avx512(prefix, value)
                })
            }
        }
    };
}

define_dary_prefix_sum_tree!(
    DaryPrefixSumTreeU32,
    u32,
    16,
    add_suffix_u32x16_avx2,
    first_gt_u32x16_avx2,
    add_suffix_u32x16_avx512,
    first_gt_u32x16_avx512
);
define_dary_prefix_sum_tree!(
    DaryPrefixSumTreeU64,
    u64,
    8,
    add_suffix_u64x8_avx2,
    first_gt_u64x8_avx2,
    add_suffix_u64x8_avx512,
    first_gt_u64x8_avx512
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
    fn test_dary_prefix_sum_tree() {
        let mut rng = Xorshift::default();
        for len in [0, 1, 7, 8, 15, 16, 17, 255, 256, 257, 4095, 4096, 4097] {
            let values: Vec<_> = (0..len).map(|_| rng.rand(8) as u32).collect();
            for backend in backends() {
                let mut actual = DaryPrefixSumTreeU32::build(&values, backend);
                let mut expected = values.clone();
                for step in 0..500 {
                    if len != 0 {
                        let index = rng.rand(len as u64) as usize;
                        if step % 3 == 0 {
                            let value = rng.rand(32) as u32;
                            actual.set(index, value);
                            expected[index] = value;
                        } else {
                            let value = rng.rand(8) as u32;
                            actual.update(index, value);
                            expected[index] += value;
                        }
                    }
                    for end in [0, len / 2, len] {
                        assert_eq!(actual.accumulate0(end), expected[..end].iter().sum());
                    }
                    if len != 0 {
                        let left = rng.rand(len as u64) as usize;
                        let right = left + rng.rand((len - left + 1) as u64) as usize;
                        assert_eq!(actual.fold(left, right), expected[left..right].iter().sum());
                        assert_eq!(actual.get(left), expected[left]);
                    }
                    let mut sum = 0;
                    let prefix: Vec<_> = expected
                        .iter()
                        .map(|&value| {
                            sum += value;
                            sum
                        })
                        .collect();
                    for value in [0, sum / 2, sum] {
                        assert_eq!(
                            actual.partition_point_acc(value),
                            prefix.partition_point(|&prefix| prefix <= value)
                        );
                    }
                }
            }
        }

        let values: Vec<_> = (0..513).map(|_| rng.rand(16)).collect();
        for backend in backends() {
            let mut actual = DaryPrefixSumTreeU64::build(&values, backend);
            let mut expected = values.clone();
            for _ in 0..1000 {
                let index = rng.rand(expected.len() as u64) as usize;
                let value = rng.rand(16);
                actual.update(index, value);
                expected[index] += value;
                let end = rng.rand(expected.len() as u64 + 1) as usize;
                assert_eq!(actual.accumulate0(end), expected[..end].iter().sum());
            }
            assert_eq!(actual.fold_all(), expected.iter().sum());
        }
    }
}
