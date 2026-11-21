#[cfg(target_arch = "x86_64")]
use super::prefetch_block;
#[cfg(target_arch = "x86_64")]
use super::simd;
use super::{Commutative, DaryAddKernelI32, DaryAddKernelI64, Invertible, SimdMonoid, simd_monoid};
use super::{SimdBackend, simd_backend};

#[codesnip::entry("SimdBinaryIndexedTree")]
pub use self::simd_fenwick_tree::{SimdBinaryIndexedTree, SimdFenwickGroup, SimdFenwickKernel};

#[codesnip::entry("SimdBinaryIndexedTree", include("avx_helper"))]
mod simd_fenwick_tree {
    #![allow(unsafe_op_in_unsafe_fn)] // Backend dispatch guards SIMD block operations.
    #[cfg(target_arch = "x86_64")]
    use super::prefetch_block;
    use super::{SimdBackend, simd_backend};
    use std::fmt::{self, Debug, Formatter};

    /// Block operations for a binary indexed tree over a commutative monoid.
    ///
    /// B is a power of two greater than one. At each level, the B-th Fenwick cells
    /// move to the next level. `block` receives at most B - 1 cells, not original keys,
    /// and pads unused lanes with unit. A kernel may cache the local prefix values. `add` applies a value
    /// to local Fenwick ancestors (lane, lane | (lane + 1), ...), including padding.
    /// `prefix` combines cells at end - 1, (end & (end - 1)) - 1, ... in that block;
    /// prefix(0) is unit. The tree never queries the last padding lane.
    ///
    /// # Safety
    /// ISA is 0 (portable), 1 (AVX2), or 2 (AVX-512F/DQ/CD/BW/VL), supported by the CPU.
    /// Additional instructions require the kernel's own check. Unsafe methods receive
    /// lane < B or end < B. Portable implementations must work on non-x86 targets.
    pub trait SimdFenwickKernel {
        const B: usize;
        type Key;
        type Output;
        type Value: Copy;
        type Block;
        fn unit() -> Self::Value;
        fn encode(key: Self::Key) -> Self::Value;
        fn decode(value: Self::Value) -> Self::Output;
        fn combine(left: Self::Value, right: Self::Value) -> Self::Value;
        fn block(values: impl Iterator<Item = Self::Value>) -> Self::Block;
        /// # Safety
        /// ISA and lane must satisfy the block contract.
        unsafe fn add<const ISA: u8>(block: &mut Self::Block, lane: usize, value: Self::Value);
        /// # Safety
        /// ISA and end must satisfy the block contract.
        unsafe fn prefix<const ISA: u8>(block: &Self::Block, end: usize) -> Self::Value;
    }

    /// Inverses for range folds and replacement of individual values.
    pub trait SimdFenwickGroup: SimdFenwickKernel {
        fn inverse(value: Self::Value) -> Self::Value;
    }

    pub struct SimdBinaryIndexedTree<K>
    where
        K: SimdFenwickKernel,
    {
        levels: Vec<Vec<K::Block>>,
        len: usize,
        backend: SimdBackend,
    }
    impl<K> Clone for SimdBinaryIndexedTree<K>
    where
        K: SimdFenwickKernel<Block: Clone>,
    {
        fn clone(&self) -> Self {
            Self {
                levels: self.levels.clone(),
                len: self.len,
                backend: self.backend,
            }
        }
    }
    impl<K> Debug for SimdBinaryIndexedTree<K>
    where
        K: SimdFenwickKernel<Block: Debug>,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.debug_struct("SimdBinaryIndexedTree")
                .field("levels", &self.levels)
                .field("len", &self.len)
                .field("backend", &self.backend)
                .finish()
        }
    }
    impl<K> SimdBinaryIndexedTree<K>
    where
        K: SimdFenwickKernel,
    {
        pub fn new(len: usize) -> Self {
            Self::build(std::iter::repeat_n(K::unit(), len))
        }
        pub fn from_keys(keys: impl ExactSizeIterator<Item = K::Key>) -> Self {
            Self::build(keys.map(K::encode))
        }
        pub fn len(&self) -> usize {
            self.len
        }
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        pub fn update(&mut self, index: usize, value: K::Key) {
            assert!(index < self.len);
            let value = K::encode(value);
            // SAFETY: index is in bounds.
            unsafe { self.add_value(index, value) };
        }
        pub fn accumulate0(&self, end: usize) -> K::Output {
            assert!(end <= self.len);
            K::decode(self.prefix(end))
        }
        pub fn accumulate(&self, index: usize) -> K::Output {
            assert!(index < self.len);
            self.accumulate0(index + 1)
        }
        pub fn fold(&self, left: usize, right: usize) -> K::Output
        where
            K: SimdFenwickGroup,
        {
            assert!(left <= right && right <= self.len);
            K::decode(self.fold_value::<false>(left, right))
        }
        pub fn get(&self, index: usize) -> K::Output
        where
            K: SimdFenwickGroup,
        {
            assert!(index < self.len);
            K::decode(self.fold_value::<true>(index, index + 1))
        }
        pub fn set(&mut self, index: usize, value: K::Key)
        where
            K: SimdFenwickGroup,
        {
            assert!(index < self.len);
            let old = self.fold_value::<true>(index, index + 1);
            let delta = K::combine(K::inverse(old), K::encode(value));
            // SAFETY: index is in bounds.
            unsafe { self.add_value(index, delta) };
        }

        /// Hints the cache for a future access; index need not be in bounds.
        pub fn prefetch(&self, index: usize) {
            #[cfg(target_arch = "x86_64")]
            {
                let mut index = index;
                for level in self.levels.iter().take(2) {
                    if let Some(block) = level.get(index / K::B) {
                        prefetch_block(block);
                    }
                    index /= K::B;
                }
            }
            let _ = index;
        }

        fn build(keys: impl ExactSizeIterator<Item = K::Value>) -> Self {
            assert!(K::B >= 2 && K::B.is_power_of_two());
            let mut values: Vec<_> = keys.collect();
            let len = values.len();
            for index in 0..len {
                let parent = index | (index + 1);
                if parent < len {
                    values[parent] = K::combine(values[parent], values[index]);
                }
            }
            let mut levels = Vec::new();
            while !values.is_empty() {
                // The extra block makes an end boundary valid even at a multiple of B.
                let count = values.len() / K::B + 1;
                let blocks = (0..count)
                    .map(|i| {
                        let begin = i * K::B;
                        let end = begin + (values.len() - begin).min(K::B - 1);
                        K::block(values[begin..end].iter().copied())
                    })
                    .collect();
                levels.push(blocks);
                values = values.into_iter().skip(K::B - 1).step_by(K::B).collect();
            }
            Self {
                levels,
                len,
                backend: simd_backend(),
            }
        }
        unsafe fn add_value(&mut self, index: usize, value: K::Value) {
            // The backend was selected on this CPU.
            match self.backend {
                SimdBackend::Scalar => self.add::<0>(index, value),
                SimdBackend::Avx2 => self.add::<1>(index, value),
                SimdBackend::Avx512 => self.add::<2>(index, value),
            }
        }

        unsafe fn add<const ISA: u8>(&mut self, mut index: usize, value: K::Value) {
            for level in &mut self.levels {
                K::add::<ISA>(level.get_unchecked_mut(index / K::B), index % K::B, value);
                index /= K::B;
            }
        }
        fn prefix(&self, end: usize) -> K::Value {
            // SAFETY: public callers bound end; every level includes its end sentinel block.
            unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.prefix_with::<0>(end),
                    SimdBackend::Avx2 => self.prefix_with::<1>(end),
                    SimdBackend::Avx512 => self.prefix_with::<2>(end),
                }
            }
        }
        unsafe fn prefix_with<const ISA: u8>(&self, mut end: usize) -> K::Value {
            let mut value = K::unit();
            for level in &self.levels {
                value = K::combine(
                    value,
                    K::prefix::<ISA>(level.get_unchecked(end / K::B), end % K::B),
                );
                end /= K::B;
            }
            value
        }
        fn fold_value<const POINT: bool>(&self, left: usize, right: usize) -> K::Value
        where
            K: SimdFenwickGroup,
        {
            // SAFETY: callers bound both endpoints; each level includes its end sentinel block.
            unsafe {
                match self.backend {
                    SimdBackend::Scalar => self.difference::<0, POINT>(left, right),
                    SimdBackend::Avx2 => self.difference::<1, POINT>(left, right),
                    SimdBackend::Avx512 => self.difference::<2, POINT>(left, right),
                }
            }
        }
        unsafe fn difference<const ISA: u8, const POINT: bool>(
            &self,
            left: usize,
            right: usize,
        ) -> K::Value
        where
            K: SimdFenwickGroup,
        {
            if left == right {
                return K::unit();
            }
            // Point access cancels the shared ancestors; range folds keep a predictable depth.
            let depth = if POINT {
                ((left ^ right).ilog2() / K::B.trailing_zeros() + 1) as usize
            } else {
                self.levels.len()
            };
            match depth {
                1 => self.difference_depth::<ISA, 1>(left, right),
                2 => self.difference_depth::<ISA, 2>(left, right),
                3 => self.difference_depth::<ISA, 3>(left, right),
                4 => self.difference_depth::<ISA, 4>(left, right),
                5 => self.difference_depth::<ISA, 5>(left, right),
                6 => self.difference_depth::<ISA, 6>(left, right),
                7 => self.difference_depth::<ISA, 7>(left, right),
                8 => self.difference_depth::<ISA, 8>(left, right),
                _ => self.difference_depth::<ISA, 0>(left, right),
            }
        }
        unsafe fn difference_depth<const ISA: u8, const D: usize>(
            &self,
            left: usize,
            right: usize,
        ) -> K::Value
        where
            K: SimdFenwickGroup,
        {
            let mut l = K::unit();
            let mut r = K::unit();
            let depth = if D == 0 { self.levels.len() } else { D };
            for at in 0..depth {
                let a = left >> (at * K::B.trailing_zeros() as usize);
                let b = right >> (at * K::B.trailing_zeros() as usize);
                let level = self.levels.get_unchecked(at);
                l = K::combine(l, K::prefix::<ISA>(level.get_unchecked(a / K::B), a % K::B));
                r = K::combine(r, K::prefix::<ISA>(level.get_unchecked(b / K::B), b % K::B));
            }
            K::combine(K::inverse(l), r)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{
            data_structure::{DaryAddKernelI32, DaryAddKernelI64},
            tools::Xorshift,
        };
        #[test]
        fn test_fenwick() {
            let mut rng = Xorshift::default();
            let backends = vec![SimdBackend::Scalar];
            #[cfg(target_arch = "x86_64")]
            let backends = {
                let mut backends = backends;
                if is_x86_feature_detected!("avx2") {
                    backends.push(SimdBackend::Avx2);
                }
                if crate::tools::avx512_supported() {
                    backends.push(SimdBackend::Avx512);
                }
                backends
            };
            macro_rules! check {
                ($kernel:ty, $value:ty) => {
                    for len in [
                        0, 1, 2, 7, 8, 9, 15, 16, 17, 63, 64, 65, 255, 256, 257, 1023, 1024, 1025,
                        4095, 4096, 4097,
                    ] {
                        let initial: Vec<_> = (0..len).map(|_| rng.rand64() as $value).collect();
                        for &backend in &backends {
                            let mut bit = SimdBinaryIndexedTree::<$kernel>::from_keys(
                                initial.iter().copied(),
                            );
                            bit.backend = backend;
                            let mut expected = initial.clone();
                            for end in 0..=len {
                                assert_eq!(
                                    bit.accumulate0(end),
                                    expected[..end]
                                        .iter()
                                        .copied()
                                        .fold(0, <$value>::wrapping_add)
                                );
                            }
                            for step in 0..1000 {
                                if len != 0 {
                                    let width = <$kernel as SimdFenwickKernel>::B;
                                    let index = match step % 8 {
                                        0 => 0,
                                        1 => len - 1,
                                        2 => (width - 1).min(len - 1),
                                        3 => width.min(len - 1),
                                        4 => len / 2,
                                        _ => rng.rand(len as u64) as usize,
                                    };
                                    let value = match step % 7 {
                                        0 => <$value>::MIN,
                                        1 => <$value>::MAX,
                                        2 => 0,
                                        _ => rng.rand64() as $value,
                                    };
                                    if rng.rand(2) == 0 {
                                        bit.update(index, value);
                                        expected[index] = expected[index].wrapping_add(value);
                                    } else {
                                        bit.set(index, value);
                                        expected[index] = value;
                                    }
                                    assert_eq!(bit.get(index), expected[index]);
                                }
                                let left = rng.rand(len as u64 + 1) as usize;
                                let right = left + rng.rand((len - left + 1) as u64) as usize;
                                assert_eq!(
                                    bit.fold(left, right),
                                    expected[left..right]
                                        .iter()
                                        .copied()
                                        .fold(0, <$value>::wrapping_add)
                                );
                            }
                        }
                    }
                };
            }
            check!(DaryAddKernelI32, i32);
            check!(DaryAddKernelI64, i64);
        }
        #[test]
        fn test_commutative_monoid() {
            struct Minimum<const B: usize>;
            impl<const B: usize> SimdFenwickKernel for Minimum<B> {
                const B: usize = B;
                type Key = u32;
                type Output = u32;
                type Value = u32;
                type Block = [u32; B];
                fn unit() -> u32 {
                    u32::MAX
                }
                fn encode(key: u32) -> u32 {
                    key
                }
                fn decode(value: u32) -> u32 {
                    value
                }
                fn combine(left: u32, right: u32) -> u32 {
                    left.min(right)
                }
                fn block(values: impl Iterator<Item = u32>) -> Self::Block {
                    let mut block = [u32::MAX; B];
                    for (lane, value) in values.enumerate() {
                        block[lane] = value;
                    }
                    block
                }
                unsafe fn add<const ISA: u8>(block: &mut Self::Block, mut lane: usize, value: u32) {
                    while lane < B {
                        block[lane] = block[lane].min(value);
                        lane |= lane + 1;
                    }
                }
                unsafe fn prefix<const ISA: u8>(block: &Self::Block, mut end: usize) -> u32 {
                    let mut value = u32::MAX;
                    while end != 0 {
                        value = value.min(block[end - 1]);
                        end &= end - 1;
                    }
                    value
                }
            }
            fn check<const B: usize>() {
                let mut rng = Xorshift::default();
                for len in [0, 1, B - 1, B, B + 1, B * B - 1, B * B, B * B + 1, 1025] {
                    for from_keys in [false, true] {
                        let mut values: Vec<_> = (0..len)
                            .map(|_| {
                                if from_keys {
                                    rng.rand64() as u32
                                } else {
                                    u32::MAX
                                }
                            })
                            .collect();
                        let mut bit = if from_keys {
                            SimdBinaryIndexedTree::<Minimum<B>>::from_keys(values.iter().copied())
                        } else {
                            SimdBinaryIndexedTree::<Minimum<B>>::new(len)
                        };
                        for end in 0..=len {
                            assert_eq!(
                                bit.accumulate0(end),
                                values[..end].iter().copied().min().unwrap_or(u32::MAX)
                            );
                        }
                        for step in 0..1000 {
                            if len != 0 {
                                let index = match step % 5 {
                                    0 => 0,
                                    1 => len - 1,
                                    2 => (B - 1).min(len - 1),
                                    _ => rng.rand(len as u64) as usize,
                                };
                                let value = if step == 999 { 0 } else { rng.rand64() as u32 };
                                bit.update(index, value);
                                values[index] = values[index].min(value);
                            }
                            let end = match step % 4 {
                                0 => 0,
                                1 => len,
                                _ => rng.rand(len as u64 + 1) as usize,
                            };
                            assert_eq!(
                                bit.accumulate0(end),
                                values[..end].iter().copied().min().unwrap_or(u32::MAX)
                            );
                            if step % 127 == 0 {
                                bit = bit.clone();
                            }
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

#[codesnip::entry("_SimdFenwickAdapter", when("SimdMonoid", "SimdBinaryIndexedTree"))]
mod fenwick_monoid {
    use super::{Commutative, Invertible, SimdMonoid, simd_monoid::LANES};
    use super::{SimdFenwickGroup, SimdFenwickKernel};

    impl<M> SimdFenwickKernel for M
    where
        M: SimdMonoid + Commutative,
    {
        const B: usize = LANES;
        type Key = M::T;
        type Output = M::T;
        type Value = M::T;
        type Block = [M::T; LANES];

        fn unit() -> M::T {
            M::unit()
        }
        fn encode(key: M::T) -> M::T {
            key
        }
        fn decode(value: M::T) -> M::T {
            value
        }
        fn combine(left: M::T, right: M::T) -> M::T {
            M::operate(&left, &right)
        }
        fn block(values: impl Iterator<Item = M::T>) -> Self::Block {
            let mut block = [M::unit(); LANES];
            let mut values = values.fuse();
            // Each cell completes the prefix at end using the earlier cached prefix.
            for end in 1..LANES {
                let cell = values.next().unwrap_or_else(M::unit);
                let before = end & (end - 1);
                block[end] = M::operate(&block[before], &cell);
            }
            block
        }
        unsafe fn add<const ISA: u8>(block: &mut Self::Block, lane: usize, value: M::T) {
            unsafe { M::broadcast_suffix::<ISA>(block, lane + 1, value) };
        }
        unsafe fn prefix<const ISA: u8>(block: &Self::Block, end: usize) -> M::T {
            block[end]
        }
    }

    impl<M> SimdFenwickGroup for M
    where
        M: SimdMonoid + Commutative + Invertible,
    {
        fn inverse(value: M::T) -> M::T {
            M::inverse(&value)
        }
    }
}

#[codesnip::entry(
    "_DaryFenwickKernel",
    when("DaryNumericKernel", "SimdBinaryIndexedTree")
)]
mod fenwick_numeric {
    #![allow(unsafe_op_in_unsafe_fn)] // The tree selects an ISA supported by the CPU.
    #[cfg(target_arch = "x86_64")]
    use super::simd;
    use super::{DaryAddKernelI32, DaryAddKernelI64};
    use super::{SimdFenwickGroup, SimdFenwickKernel};

    #[repr(C, align(64))]
    #[derive(Clone, Debug)]
    pub struct FenwickNumericBlock<T, const B: usize>([T; B]);

    macro_rules! define_fenwick_kernel {
        ($kernel:ty, $value:ty, $branch:expr, $unsigned:ty, $avx2:ident, $avx512:ident) => {
            impl SimdFenwickKernel for $kernel {
                const B: usize = $branch;
                type Key = $value;
                type Output = $value;
                type Value = $value;
                type Block = FenwickNumericBlock<$value, $branch>;
                fn unit() -> $value {
                    0
                }
                fn encode(key: $value) -> $value {
                    key
                }
                fn decode(value: $value) -> $value {
                    value
                }
                fn combine(left: $value, right: $value) -> $value {
                    left.wrapping_add(right)
                }
                fn block(values: impl Iterator<Item = $value>) -> Self::Block {
                    let mut block = [0; $branch];
                    for (lane, value) in values.enumerate() {
                        block[lane] = value;
                    }
                    // Cache local prefixes while retaining the Fenwick cell contract.
                    for i in 0..$branch {
                        let before = i & (i + 1);
                        if before != 0 {
                            block[i] = block[i].wrapping_add(block[before - 1]);
                        }
                    }
                    FenwickNumericBlock(block)
                }
                unsafe fn add<const ISA: u8>(
                    block: &mut Self::Block,
                    mut lane: usize,
                    value: $value,
                ) {
                    #[cfg(target_arch = "x86_64")]
                    if ISA != 0 {
                        let values = &mut *block.0.as_mut_ptr().cast::<[$unsigned; $branch]>();
                        if ISA == 1 {
                            simd::$avx2(values, lane, value as $unsigned);
                        } else {
                            simd::$avx512(values, lane, value as $unsigned);
                        }
                        return;
                    }
                    while lane < $branch {
                        block.0[lane] = block.0[lane].wrapping_add(value);
                        lane += 1;
                    }
                }
                unsafe fn prefix<const ISA: u8>(block: &Self::Block, end: usize) -> $value {
                    *block.0.get_unchecked(end.wrapping_sub(1) & ($branch - 1))
                        & (0 as $value).wrapping_sub((end != 0) as $value)
                }
            }
            impl SimdFenwickGroup for $kernel {
                fn inverse(value: $value) -> $value {
                    value.wrapping_neg()
                }
            }
        };
    }
    define_fenwick_kernel!(
        DaryAddKernelI32,
        i32,
        16,
        u32,
        add_suffix_u32x16_avx2,
        add_suffix_u32x16_avx512
    );
    define_fenwick_kernel!(
        DaryAddKernelI64,
        i64,
        16,
        u64,
        add_suffix_u64x16_avx2,
        add_suffix_u64x16_avx512
    );
}
