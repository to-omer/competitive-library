#[cfg(target_arch = "x86_64")]
use super::prefetch_block;
use super::{Invertible, SimdMonoid, simd_monoid};
use super::{RangeBoundsExt, SimdBackend, simd_backend};

#[codesnip::entry("DarySegmentTree")]
pub use self::dary_tree::{DarySegmentGroup, DarySegmentKernel, DarySegmentTree};

#[codesnip::entry("DarySegmentTree", include("avx_helper", "discrete_steps"))]
mod dary_tree {
    #![allow(unsafe_op_in_unsafe_fn)] // Construction selects a supported ISA for block operations.
    #[cfg(target_arch = "x86_64")]
    use super::prefetch_block;
    use super::{RangeBoundsExt, SimdBackend, simd_backend};
    use std::{
        fmt::{self, Debug, Formatter},
        ops::{Range, RangeBounds},
    };

    /// Storage and ordered monoid operations for a d-ary segment tree.
    ///
    /// B is a power of two greater than one. `block` receives at most B values and
    /// pads the remaining lanes with `unit`. `combine(left, right)` preserves order.
    /// A Change describes how a whole block's aggregate changed: it may be a delta
    /// or its replacement value. Returning None from replace/update/repair stops propagation
    /// because that aggregate did not change; the block itself must still be updated.
    ///
    /// # Safety
    /// Unsafe operations receive in-bounds lanes/ranges (including empty ranges).
    /// ISA is 0 (portable), 1 (AVX2), or 2 (AVX-512F/DQ/CD/BW/VL), supported by the CPU.
    /// Implementations must provide the portable path on other architectures and
    /// check any additional instruction-set requirements themselves.
    pub trait DarySegmentKernel {
        const B: usize;
        type Key;
        type Output;
        type Value: Copy;
        type Change: Copy;
        type Block;

        fn encode(key: Self::Key) -> Self::Value;
        fn decode(value: Self::Value) -> Self::Output;
        fn unit() -> Self::Value;
        fn combine(left: Self::Value, right: Self::Value) -> Self::Value;
        fn block(values: impl Iterator<Item = Self::Value>) -> Self::Block;
        fn get(block: &Self::Block, lane: usize) -> Self::Value;
        fn repair_value(value: Self::Value, change: Self::Change) -> Self::Value;
        /// # Safety
        /// ISA and range must satisfy the block contract.
        unsafe fn fold<const ISA: u8>(block: &Self::Block, range: Range<usize>) -> Self::Value;
        /// # Safety
        /// ISA and lane must satisfy the block contract.
        unsafe fn replace<const ISA: u8>(
            block: &mut Self::Block,
            lane: usize,
            value: Self::Value,
        ) -> Option<Self::Change>;
        /// Appends value to the point in monoid order.
        /// # Safety
        /// ISA and lane must satisfy the block contract.
        unsafe fn update<const ISA: u8>(
            block: &mut Self::Block,
            lane: usize,
            value: Self::Value,
        ) -> Option<Self::Change> {
            Self::replace::<ISA>(block, lane, Self::combine(Self::get(block, lane), value))
        }
        /// # Safety
        /// ISA and lane must satisfy the block contract; change refers to that child.
        unsafe fn repair<const ISA: u8>(
            block: &mut Self::Block,
            lane: usize,
            change: Self::Change,
        ) -> Option<Self::Change>;
    }

    /// Inverses for computing a range fold from its two prefixes, in monoid order.
    pub trait DarySegmentGroup: DarySegmentKernel {
        fn inverse(value: Self::Value) -> Self::Value;
    }

    pub struct DarySegmentTree<K>
    where
        K: DarySegmentKernel,
    {
        levels: Vec<Vec<K::Block>>,
        total: K::Value,
        len: usize,
        backend: SimdBackend,
    }

    impl<K> Clone for DarySegmentTree<K>
    where
        K: DarySegmentKernel<Block: Clone>,
    {
        fn clone(&self) -> Self {
            Self {
                levels: self.levels.clone(),
                total: self.total,
                len: self.len,
                backend: self.backend,
            }
        }
    }
    impl<K> Debug for DarySegmentTree<K>
    where
        K: DarySegmentKernel<Block: Debug, Value: Debug>,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.debug_struct("DarySegmentTree")
                .field("levels", &self.levels)
                .field("total", &self.total)
                .field("len", &self.len)
                .field("backend", &self.backend)
                .finish()
        }
    }

    impl<K> DarySegmentTree<K>
    where
        K: DarySegmentKernel,
    {
        pub fn new(len: usize) -> Self {
            Self::build(std::iter::repeat_n(K::unit(), len), simd_backend())
        }

        pub fn from_keys(keys: impl ExactSizeIterator<Item = K::Key>) -> Self {
            Self::build(keys.map(K::encode), simd_backend())
        }

        pub fn len(&self) -> usize {
            self.len
        }
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        pub fn get(&self, index: usize) -> K::Output {
            assert!(index < self.len);
            K::decode(K::get(&self.levels[0][index / K::B], index % K::B))
        }

        pub fn set(&mut self, index: usize, key: K::Key) {
            assert!(index < self.len);
            self.set_value::<false>(index, K::encode(key));
        }

        pub fn clear(&mut self, index: usize) {
            assert!(index < self.len);
            self.set_value::<false>(index, K::unit());
        }

        /// Appends the key to the current value at index, in monoid order.
        pub fn update(&mut self, index: usize, key: K::Key) {
            assert!(index < self.len);
            self.set_value::<true>(index, K::encode(key));
        }

        pub fn fold<R>(&self, range: R) -> K::Output
        where
            R: RangeBounds<usize>,
        {
            let range = range.to_range_bounded(0, self.len).expect("invalid range");
            // SAFETY: the selected ISA is supported; all ranges are within a block.
            let value = unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.fold_range::<0>(range.start, range.end),
                    SimdBackend::Avx2 => self.fold_range::<1>(range.start, range.end),
                    SimdBackend::Avx512 => self.fold_range::<2>(range.start, range.end),
                }
            };
            K::decode(value)
        }

        pub fn fold_all(&self) -> K::Output {
            K::decode(self.total)
        }

        /// Uses two prefix folds and an inverse; useful for kernels storing prefix aggregates.
        pub fn fold_by_prefix<R>(&self, range: R) -> K::Output
        where
            R: RangeBounds<usize>,
            K: DarySegmentGroup,
        {
            let range = range.to_range_bounded(0, self.len).expect("invalid range");
            if range.is_empty() {
                return K::decode(K::unit());
            }
            // SAFETY: the range is bounded and the selected ISA is supported.
            let value = unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.prefix_difference::<0>(range.start, range.end),
                    SimdBackend::Avx2 => self.prefix_difference::<1>(range.start, range.end),
                    SimdBackend::Avx512 => self.prefix_difference::<2>(range.start, range.end),
                }
            };
            K::decode(value)
        }

        /// Prefetches the lower blocks containing index for a later operation.
        pub fn prefetch(&self, index: usize) {
            #[cfg(target_arch = "x86_64")]
            {
                let mut index = index;
                for level in self.levels.iter().take(2) {
                    index /= K::B;
                    if let Some(block) = level.get(index) {
                        prefetch_block(block);
                    }
                }
            }
            let _ = index;
        }

        fn build(keys: impl ExactSizeIterator<Item = K::Value>, backend: SimdBackend) -> Self {
            assert!(K::B >= 2 && K::B.is_power_of_two());
            // SAFETY: callers select a supported backend.
            unsafe {
                match backend {
                    SimdBackend::Scalar => Self::build_with::<0>(keys, backend),
                    SimdBackend::Avx2 => Self::build_with::<1>(keys, backend),
                    SimdBackend::Avx512 => Self::build_with::<2>(keys, backend),
                }
            }
        }

        unsafe fn build_with<const ISA: u8>(
            keys: impl ExactSizeIterator<Item = K::Value>,
            backend: SimdBackend,
        ) -> Self {
            let len = keys.len();
            let mut keys = keys.fuse();
            let mut consumed = 0;
            let mut blocks: Vec<_> = (0..len.div_ceil(K::B))
                .map(|_| K::block(keys.by_ref().take(K::B).inspect(|_| consumed += 1)))
                .collect();
            assert!(
                consumed == len && keys.next().is_none(),
                "iterator length mismatch"
            );
            let mut levels = Vec::new();
            let total = loop {
                if blocks.len() <= 1 {
                    let total = blocks
                        .first()
                        .map_or(K::unit(), |b| K::fold::<ISA>(b, 0..K::B));
                    levels.push(blocks);
                    break total;
                }
                let count = blocks.len();
                let mut summaries = blocks.iter().map(|b| K::fold::<ISA>(b, 0..K::B));
                let next: Vec<_> = (0..count.div_ceil(K::B))
                    .map(|_| K::block(summaries.by_ref().take(K::B)))
                    .collect();
                levels.push(blocks);
                blocks = next;
            };
            Self {
                levels,
                total,
                len,
                backend,
            }
        }

        fn set_value<const APPEND: bool>(&mut self, index: usize, value: K::Value) {
            // SAFETY: public callers check index and construction selects the ISA.
            unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.replace::<0, APPEND>(index, value),
                    SimdBackend::Avx2 => self.replace::<1, APPEND>(index, value),
                    SimdBackend::Avx512 => self.replace::<2, APPEND>(index, value),
                }
            }
        }

        unsafe fn replace<const ISA: u8, const APPEND: bool>(
            &mut self,
            mut index: usize,
            value: K::Value,
        ) {
            let block = &mut self.levels[0][index / K::B];
            let change = if APPEND {
                K::update::<ISA>(block, index % K::B, value)
            } else {
                K::replace::<ISA>(block, index % K::B, value)
            };
            let Some(mut change) = change else {
                return;
            };
            index /= K::B;
            for level in &mut self.levels[1..] {
                let Some(next) = K::repair::<ISA>(&mut level[index / K::B], index % K::B, change)
                else {
                    return;
                };
                change = next;
                index /= K::B;
            }
            self.total = K::repair_value(self.total, change);
        }

        unsafe fn fold_range<const ISA: u8>(&self, mut left: usize, mut right: usize) -> K::Value {
            let mut prefix = K::unit();
            let mut suffix = K::unit();
            for level in &self.levels {
                if left >= right {
                    break;
                }
                let first = left / K::B;
                let last = (right - 1) / K::B;
                if first == last {
                    prefix = K::combine(
                        prefix,
                        K::fold::<ISA>(&level[first], left % K::B..(right - 1) % K::B + 1),
                    );
                    break;
                }
                if !left.is_multiple_of(K::B) {
                    prefix = K::combine(prefix, K::fold::<ISA>(&level[first], left % K::B..K::B));
                    left = (first + 1) * K::B;
                }
                if !right.is_multiple_of(K::B) {
                    suffix = K::combine(K::fold::<ISA>(&level[last], 0..right % K::B), suffix);
                    right = last * K::B;
                }
                left /= K::B;
                right /= K::B;
            }
            K::combine(prefix, suffix)
        }
        unsafe fn prefix_difference<const ISA: u8>(&self, left: usize, right: usize) -> K::Value
        where
            K: DarySegmentGroup,
        {
            if right == self.len {
                let mut index = left;
                let mut value = K::unit();
                for level in &self.levels {
                    value = K::combine(
                        K::fold::<ISA>(level.get_unchecked(index / K::B), 0..index % K::B),
                        value,
                    );
                    index /= K::B;
                }
                return K::combine(K::inverse(value), self.total);
            }
            match self.levels.len() {
                1 => self.prefix_pair::<ISA, 1>(left, right),
                2 => self.prefix_pair::<ISA, 2>(left, right),
                3 => self.prefix_pair::<ISA, 3>(left, right),
                4 => self.prefix_pair::<ISA, 4>(left, right),
                5 => self.prefix_pair::<ISA, 5>(left, right),
                6 => self.prefix_pair::<ISA, 6>(left, right),
                7 => self.prefix_pair::<ISA, 7>(left, right),
                8 => self.prefix_pair::<ISA, 8>(left, right),
                _ => self.prefix_pair::<ISA, 0>(left, right),
            }
        }
        unsafe fn prefix_pair<const ISA: u8, const D: usize>(
            &self,
            left: usize,
            right: usize,
        ) -> K::Value
        where
            K: DarySegmentGroup,
        {
            let mut l = K::unit();
            let mut r = K::unit();
            for at in 0..if D == 0 { self.levels.len() } else { D } {
                let index_l = left >> (at * K::B.trailing_zeros() as usize);
                let index_r = right >> (at * K::B.trailing_zeros() as usize);
                let level = self.levels.get_unchecked(at);
                l = K::combine(
                    K::fold::<ISA>(level.get_unchecked(index_l / K::B), 0..index_l % K::B),
                    l,
                );
                r = K::combine(
                    K::fold::<ISA>(level.get_unchecked(index_r / K::B), 0..index_r % K::B),
                    r,
                );
            }
            K::combine(K::inverse(l), r)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[cfg(target_arch = "x86_64")]
        use crate::tools::avx512_supported;
        use crate::{
            data_structure::{
                DaryAddKernelI32, DaryAddKernelI64, DaryMaxKernelI32, DaryMaxKernelI64,
                DaryMinKernelI32, DaryMinKernelI64,
            },
            tools::Xorshift,
        };

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
        fn test_ordered_composite() {
            use crate::algebra::LinearOperation;
            use std::num::Wrapping as W;
            let mut rng = Xorshift::default();
            for backend in backends() {
                let mut crossing = vec![(W(1), W(0)); 48];
                crossing[16] = (W(1), W(1));
                crossing[32] = (W(2), W(0));
                let mut tree =
                    DarySegmentTree::<LinearOperation<W<i32>>>::from_keys(crossing.into_iter());
                tree.backend = backend;
                assert_eq!(tree.fold(1..33), (W(2), W(2)));
                for len in [0, 1, 15, 16, 17, 48, 255, 256, 257, 4097] {
                    let mut values: Vec<_> = (0..len)
                        .map(|_| (W(rng.rand64() as i32), W(rng.rand64() as i32)))
                        .collect();
                    let mut tree = DarySegmentTree::<LinearOperation<W<i32>>>::from_keys(
                        values.iter().copied(),
                    );
                    tree.backend = backend;
                    for turn in 0..1000 {
                        if len != 0 {
                            let index = rng.rand(len as u64) as usize;
                            let value = match turn % 8 {
                                0 => (W(0), W(rng.rand64() as i32)),
                                1 => (W(1), W(0)),
                                2 => (W(0), W(0)),
                                _ => (W(rng.rand64() as i32), W(rng.rand64() as i32)),
                            };
                            if turn % 2 == 0 {
                                tree.set(index, value);
                                values[index] = value;
                            } else {
                                tree.update(index, value);
                                values[index] = (
                                    values[index].0 * value.0,
                                    values[index].1 * value.0 + value.1,
                                );
                            }
                        }
                        let left = rng.rand(len as u64 + 1) as usize;
                        let right = left + rng.rand((len - left + 1) as u64) as usize;
                        let expected = values[left..right]
                            .iter()
                            .fold((W(1), W(0)), |(a, b), &(c, d)| (a * c, b * c + d));
                        assert_eq!(tree.fold(left..right), expected);
                    }
                }
            }
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
                        for (backend, from_values) in backends()
                            .into_iter()
                            .flat_map(|backend| [(backend, false), (backend, true)])
                        {
                            let mut minimum = if from_values {
                                <$minimum>::build(values.clone().into_iter(), backend)
                            } else {
                                <$minimum>::new(len)
                            };
                            let mut maximum = if from_values {
                                <$maximum>::build(values.clone().into_iter(), backend)
                            } else {
                                <$maximum>::new(len)
                            };
                            let mut sum = if from_values {
                                <$sum>::build(values.clone().into_iter(), backend)
                            } else {
                                <$sum>::new(len)
                            };
                            minimum.backend = backend;
                            maximum.backend = backend;
                            sum.backend = backend;
                            let mut expected_minimum = if from_values {
                                values.clone()
                            } else {
                                vec![<$value>::MAX; len]
                            };
                            let mut expected_maximum = if from_values {
                                values.clone()
                            } else {
                                vec![<$value>::MIN; len]
                            };
                            let mut expected_sum = if from_values {
                                values.clone()
                            } else {
                                vec![0; len]
                            };
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
                                            assert_eq!(sum.fold_by_prefix(left..right), expected_sum[left..right].iter().copied().fold(0, <$value>::wrapping_add));
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
                                assert_eq!(sum.fold_by_prefix(..),expected_sum.iter().copied().fold(0, <$value>::wrapping_add));
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
                DarySegmentTree<DaryMinKernelI32>,
                DarySegmentTree<DaryMaxKernelI32>,
                DarySegmentTree<DaryAddKernelI32>
            );
            check!(
                i64,
                DarySegmentTree<DaryMinKernelI64>,
                DarySegmentTree<DaryMaxKernelI64>,
                DarySegmentTree<DaryAddKernelI64>
            );
        }
        #[test]
        fn test_group_fold() {
            struct Permutation<const B: usize>;
            impl<const B: usize> DarySegmentKernel for Permutation<B> {
                const B: usize = B;
                type Key = [usize; 3];
                type Output = Self::Key;
                type Value = Self::Key;
                type Change = Self::Key;
                type Block = [Self::Value; B];
                fn encode(key: Self::Key) -> Self::Value {
                    key
                }
                fn decode(value: Self::Value) -> Self::Output {
                    value
                }
                fn unit() -> Self::Value {
                    [0, 1, 2]
                }
                fn combine(left: Self::Value, right: Self::Value) -> Self::Value {
                    left.map(|x| right[x])
                }
                fn block(values: impl Iterator<Item = Self::Value>) -> Self::Block {
                    let mut block = [Self::unit(); B];
                    for (lane, value) in values.enumerate() {
                        block[lane] = value;
                    }
                    block
                }
                fn get(block: &Self::Block, lane: usize) -> Self::Value {
                    block[lane]
                }
                fn repair_value(_value: Self::Value, change: Self::Change) -> Self::Value {
                    change
                }
                unsafe fn fold<const ISA: u8>(
                    block: &Self::Block,
                    range: Range<usize>,
                ) -> Self::Value {
                    block[range]
                        .iter()
                        .copied()
                        .fold(Self::unit(), Self::combine)
                }
                unsafe fn replace<const ISA: u8>(
                    block: &mut Self::Block,
                    lane: usize,
                    value: Self::Value,
                ) -> Option<Self::Change> {
                    block[lane] = value;
                    Some(Self::fold::<ISA>(block, 0..B))
                }
                unsafe fn repair<const ISA: u8>(
                    block: &mut Self::Block,
                    lane: usize,
                    change: Self::Change,
                ) -> Option<Self::Change> {
                    Self::replace::<ISA>(block, lane, change)
                }
            }
            impl<const B: usize> DarySegmentGroup for Permutation<B> {
                fn inverse(value: Self::Value) -> Self::Value {
                    let mut inverse = [0; 3];
                    for (i, x) in value.into_iter().enumerate() {
                        inverse[x] = i;
                    }
                    inverse
                }
            }
            fn check<const B: usize>() {
                let permutations = [
                    [0, 1, 2],
                    [0, 2, 1],
                    [1, 0, 2],
                    [1, 2, 0],
                    [2, 0, 1],
                    [2, 1, 0],
                ];
                let mut rng = Xorshift::default();
                for len in [0, 1, B - 1, B, B + 1, B * B - 1, B * B, B * B + 1, 1025] {
                    let mut values: Vec<_> = (0..len).map(|i| permutations[i % 6]).collect();
                    let mut tree =
                        DarySegmentTree::<Permutation<B>>::from_keys(values.iter().copied());
                    for step in 0..1000 {
                        if len != 0 {
                            let index = match step % 4 {
                                0 => 0,
                                1 => len - 1,
                                _ => rng.rand(len as u64) as usize,
                            };
                            let value = permutations[rng.rand(6) as usize];
                            tree.set(index, value);
                            values[index] = value;
                        }
                        let (left, right) = match step % 4 {
                            0 => (0, len),
                            1 => {
                                let end = rng.rand(len as u64 + 1) as usize;
                                (end, end)
                            }
                            _ => {
                                let a = rng.rand(len as u64 + 1) as usize;
                                let b = rng.rand(len as u64 + 1) as usize;
                                (a.min(b), a.max(b))
                            }
                        };
                        let actual = tree.fold_by_prefix(left..right);
                        for (x, actual) in actual.into_iter().enumerate() {
                            assert_eq!(
                                actual,
                                values[left..right]
                                    .iter()
                                    .fold(x, |x, permutation| permutation[x])
                            );
                        }
                    }
                }
            }
            check::<2>();
            check::<4>();
            check::<16>();
            check::<64>();
        }
    }
}

#[codesnip::entry("_SimdSegmentAdapter", when("SimdMonoid", "DarySegmentTree"))]
mod segment_monoid {
    use super::{DarySegmentGroup, DarySegmentKernel};
    use super::{Invertible, SimdMonoid, simd_monoid::LANES};
    use std::ops::Range;

    impl<M> DarySegmentKernel for M
    where
        M: SimdMonoid,
    {
        const B: usize = LANES;
        type Key = M::T;
        type Output = M::T;
        type Value = M::T;
        type Change = M::T;
        type Block = [M::T; LANES];

        fn encode(key: M::T) -> M::T {
            key
        }
        fn decode(value: M::T) -> M::T {
            value
        }
        fn unit() -> M::T {
            M::unit()
        }
        fn combine(left: M::T, right: M::T) -> M::T {
            M::operate(&left, &right)
        }
        fn block(values: impl Iterator<Item = M::T>) -> Self::Block {
            let mut values = values.fuse();
            std::array::from_fn(|_| values.next().unwrap_or_else(M::unit))
        }
        fn get(block: &Self::Block, lane: usize) -> M::T {
            block[lane]
        }
        fn repair_value(_value: M::T, change: M::T) -> M::T {
            change
        }
        unsafe fn fold<const ISA: u8>(block: &Self::Block, range: Range<usize>) -> M::T {
            unsafe { M::reduce_lanes::<ISA>(&block[range]) }
        }
        unsafe fn replace<const ISA: u8>(
            block: &mut Self::Block,
            lane: usize,
            value: M::T,
        ) -> Option<M::T> {
            block[lane] = value;
            Some(unsafe { M::reduce_lanes::<ISA>(block) })
        }
        unsafe fn repair<const ISA: u8>(
            block: &mut Self::Block,
            lane: usize,
            change: M::T,
        ) -> Option<M::T> {
            unsafe { Self::replace::<ISA>(block, lane, change) }
        }
    }

    impl<M> DarySegmentGroup for M
    where
        M: SimdMonoid + Invertible,
    {
        fn inverse(value: M::T) -> M::T {
            M::inverse(&value)
        }
    }
}
