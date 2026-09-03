#[cfg(target_arch = "x86_64")]
use super::simd;
use super::{SimdBackend, simd_backend};

#[repr(C, align(64))]
#[derive(Clone, Debug)]
struct HeapBlock<T, const D: usize>([T; D]);

impl<T: Copy, const D: usize> HeapBlock<T, D> {
    #[inline(always)]
    fn filled(value: T) -> Self {
        Self([value; D])
    }

    #[inline(always)]
    fn get(&self, index: usize) -> T {
        unsafe { *self.0.get_unchecked(index) }
    }

    #[inline(always)]
    fn set(&mut self, index: usize, value: T) {
        unsafe {
            *self.0.get_unchecked_mut(index) = value;
        }
    }
}

#[repr(C, align(64))]
#[derive(Clone, Debug)]
struct U128HeapBlock {
    low: [u64; 4],
    high: [u64; 4],
}

impl U128HeapBlock {
    #[inline(always)]
    fn filled(value: u128) -> Self {
        Self {
            low: [value as u64; 4],
            high: [(value >> 64) as u64; 4],
        }
    }

    #[inline(always)]
    fn get(&self, index: usize) -> u128 {
        unsafe {
            (*self.high.get_unchecked(index) as u128) << 64 | *self.low.get_unchecked(index) as u128
        }
    }

    #[inline(always)]
    fn set(&mut self, index: usize, value: u128) {
        unsafe {
            *self.low.get_unchecked_mut(index) = value as u64;
            *self.high.get_unchecked_mut(index) = (value >> 64) as u64;
        }
    }
}

#[inline(always)]
fn max_index<T: Ord, const D: usize>(values: &[T; D]) -> usize {
    let mut result = 0;
    for index in 1..D {
        if values[index] > values[result] {
            result = index;
        }
    }
    result
}

#[inline(always)]
fn max_index_u128(values: &U128HeapBlock) -> usize {
    let mut result = 0;
    for index in 1..4 {
        if values.high[index] > values.high[result]
            || (values.high[index] == values.high[result] && values.low[index] > values.low[result])
        {
            result = index;
        }
    }
    result
}

macro_rules! define_dary_heap {
    (
        $name:ident,
        $doc:literal,
        $value:ty,
        $storage:ty,
        $branch:expr,
        $block:ty
        , encode = $encode:expr
        , decode = $decode:expr
        $(, $field:ident: $field_type:ty = $field_value:expr)*
        $(,)?
    ) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name {
            root: $storage,
            blocks: Vec<$block>,
            len: usize,
            $(#[cfg(target_arch = "x86_64")] $field: $field_type,)*
        }

        impl $name {
            pub fn new() -> Self {
                Self::with_capacity(0)
            }

            pub fn with_capacity(capacity: usize) -> Self {
                Self::empty(capacity $(, $field_value)*)
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
            pub fn peek(&self) -> Option<$value> {
                (self.len != 0).then(|| Self::decode(self.root))
            }

            pub fn push(&mut self, value: $value) {
                let value = Self::encode(value);
                if self.len == 0 {
                    self.root = value;
                    self.len = 1;
                    return;
                }
                let mut hole = self.len;
                let block = (hole - 1) / $branch;
                if block == self.blocks.len() {
                    self.blocks.push(<$block>::filled(<$storage>::MIN));
                }
                self.len += 1;
                while hole != 0 {
                    let parent = (hole - 1) / $branch;
                    let parent_key = self.key(parent);
                    if parent_key >= value {
                        break;
                    }
                    self.set_key(hole, parent_key);
                    hole = parent;
                }
                self.set_key(hole, value);
            }

            pub fn pop(&mut self) -> Option<$value> {
                if self.len == 0 {
                    return None;
                }
                let result = self.root;
                if self.len == 1 {
                    self.root = <$storage>::MIN;
                    self.len = 0;
                    return Some(Self::decode(result));
                }
                let last = self.len - 1;
                let value = self.key(last);
                self.set_key(last, <$storage>::MIN);
                self.len = last;
                self.sift_down_after_pop(0, value);
                Some(Self::decode(result))
            }

            /// Unconditionally replaces the greatest value, or inserts into an empty heap.
            pub fn replace(&mut self, value: $value) -> Option<$value> {
                if self.len == 0 {
                    self.push(value);
                    return None;
                }
                let result = self.root;
                self.sift_down(0, Self::encode(value));
                Some(Self::decode(result))
            }

            pub fn clear(&mut self) {
                self.root = <$storage>::MIN;
                self.blocks.clear();
                self.len = 0;
            }

            pub fn into_sorted_vec(mut self) -> Vec<$value> {
                let mut values = Vec::with_capacity(self.len);
                while let Some(value) = self.pop() {
                    values.push(value);
                }
                values.reverse();
                values
            }

            fn empty(capacity: usize $(, $field: $field_type)*) -> Self {
                $({ let _ = &$field; })*
                Self {
                    root: <$storage>::MIN,
                    blocks: Vec::with_capacity(capacity.saturating_sub(1).div_ceil($branch)),
                    len: 0,
                    $(#[cfg(target_arch = "x86_64")] $field,)*
                }
            }

            fn build(values: Vec<$value> $(, $field: $field_type)*) -> Self {
                let len = values.len();
                let mut heap = Self::empty(len $(, $field)*);
                heap.len = len;
                if let Some((&root, values)) = values.split_first() {
                    heap.root = Self::encode(root);
                    heap.blocks.resize(
                        len.saturating_sub(1).div_ceil($branch),
                        <$block>::filled(<$storage>::MIN),
                    );
                    for (index, &value) in values.iter().enumerate() {
                        heap.blocks[index / $branch].set(index % $branch, Self::encode(value));
                    }
                    heap.heapify();
                }
                heap
            }

            #[inline(always)]
            fn encode(value: $value) -> $storage {
                ($encode)(value)
            }

            #[inline(always)]
            fn decode(value: $storage) -> $value {
                ($decode)(value)
            }

            #[inline(always)]
            fn key(&self, index: usize) -> $storage {
                if index == 0 {
                    self.root
                } else {
                    // SAFETY: callers only pass occupied heap indices.
                    unsafe { self.blocks.get_unchecked((index - 1) / $branch) }
                        .get((index - 1) % $branch)
                }
            }

            #[inline(always)]
            fn set_key(&mut self, index: usize, value: $storage) {
                if index == 0 {
                    self.root = value;
                } else {
                    // SAFETY: callers only pass occupied heap indices. `push` allocates the
                    // destination block before increasing `len`.
                    unsafe { self.blocks.get_unchecked_mut((index - 1) / $branch) }
                        .set((index - 1) % $branch, value);
                }
            }

            #[inline(always)]
            fn sift_down_by<F>(&mut self, mut hole: usize, value: $storage, mut max_index: F)
            where
                F: FnMut(&$block) -> usize,
            {
                if self.len <= 1 {
                    self.set_key(hole, value);
                    return;
                }
                let last_parent = (self.len - 2) / $branch;
                while hole <= last_parent {
                    // SAFETY: the loop condition guarantees a child block. Real children occupy
                    // its prefix and padding is the type minimum, so the first maximum is always
                    // a real child.
                    let block = unsafe { self.blocks.get_unchecked(hole) };
                    let lane = max_index(block);
                    let child_key = block.get(lane);
                    if child_key <= value {
                        break;
                    }
                    self.set_key(hole, child_key);
                    hole = hole * $branch + lane + 1;
                }
                self.set_key(hole, value);
            }

            fn heapify(&mut self) {
                if self.len <= 1 {
                    return;
                }
                for parent in (0..=(self.len - 2) / $branch).rev() {
                    let value = self.key(parent);
                    self.sift_down(parent, value);
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<Vec<$value>> for $name {
            fn from(values: Vec<$value>) -> Self {
                Self::build(values $(, $field_value)*)
            }
        }

        impl Extend<$value> for $name {
            fn extend<I>(&mut self, iter: I)
            where
                I: IntoIterator<Item = $value>,
            {
                for value in iter {
                    self.push(value);
                }
            }
        }

        impl FromIterator<$value> for $name {
            fn from_iter<I>(iter: I) -> Self
            where
                I: IntoIterator<Item = $value>,
            {
                let values: Vec<_> = iter.into_iter().collect();
                Self::from(values)
            }
        }
    };
}

define_dary_heap!(
    DaryHeapU32,
    "A cache-line-oriented 16-ary max-heap for medium-to-large 32-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",
    u32,
    u32,
    16,
    HeapBlock<u32, 16>,
    encode = |value| value,
    decode = |value| value,
    backend: SimdBackend = simd_backend(),
);
define_dary_heap!(
    DaryHeapI32,
    "A cache-line-oriented 16-ary max-heap for medium-to-large 32-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",
    i32,
    u32,
    16,
    HeapBlock<u32, 16>,
    encode = |value: i32| value as u32 ^ (1 << 31),
    decode = |value: u32| (value ^ (1 << 31)) as i32,
    backend: SimdBackend = simd_backend(),
);
define_dary_heap!(
    DaryHeapU64,
    "A cache-line-oriented 8-ary max-heap for large 64-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",
    u64,
    u64,
    8,
    HeapBlock<u64, 8>,
    encode = |value| value,
    decode = |value| value,
    backend: SimdBackend = simd_backend(),
);
define_dary_heap!(
    DaryHeapI64,
    "A cache-line-oriented 8-ary max-heap for large 64-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",
    i64,
    u64,
    8,
    HeapBlock<u64, 8>,
    encode = |value: i64| value as u64 ^ (1 << 63),
    decode = |value: u64| (value ^ (1 << 63)) as i64,
    backend: SimdBackend = simd_backend(),
);
define_dary_heap!(
    DaryHeapU128,
    "A cache-line-oriented 4-ary max-heap for large full-width 128-bit heaps. `BinaryHeap` can be faster for small heaps, monotone replacements, and heavily repeated keys.",
    u128,
    u128,
    4,
    U128HeapBlock,
    encode = |value| value,
    decode = |value| value,
    backend: SimdBackend = simd_backend(),
);
define_dary_heap!(
    DaryHeapI128,
    "A cache-line-oriented 4-ary max-heap for large full-width 128-bit heaps. `BinaryHeap` can be faster for small heaps, monotone replacements, and heavily repeated keys.",
    i128,
    u128,
    4,
    U128HeapBlock,
    encode = |value: i128| value as u128 ^ (1 << 127),
    decode = |value: u128| (value ^ (1 << 127)) as i128,
    backend: SimdBackend = simd_backend(),
);

macro_rules! impl_simd_heap {
    (
        $name:ident,
        $value:ty,
        $branch:expr,
        $max_avx2:ident,
        $max_avx512:ident
    ) => {
        impl $name {
            #[inline(always)]
            fn sift_down_scalar(&mut self, hole: usize, value: $value) {
                self.sift_down_by(hole, value, |block| max_index(&block.0))
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn sift_down_avx2(&mut self, hole: usize, value: $value) {
                self.sift_down_by(hole, value, |block| unsafe { simd::$max_avx2(&block.0) })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx512f")]
            unsafe fn sift_down_avx512(&mut self, hole: usize, value: $value) {
                self.sift_down_by(hole, value, |block| unsafe { simd::$max_avx512(&block.0) })
            }

            #[inline]
            fn sift_down(&mut self, hole: usize, value: $value) {
                #[cfg(target_arch = "x86_64")]
                match self.backend {
                    SimdBackend::Scalar => self.sift_down_scalar(hole, value),
                    // SAFETY: automatic construction only selects supported instruction sets;
                    // explicit construction is private and restricted to tests and benchmarks.
                    SimdBackend::Avx2 => unsafe { self.sift_down_avx2(hole, value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.sift_down_avx512(hole, value) },
                }
                #[cfg(not(target_arch = "x86_64"))]
                self.sift_down_scalar(hole, value);
            }

            #[inline(always)]
            fn sift_down_after_pop(&mut self, hole: usize, value: $value) {
                self.sift_down(hole, value);
            }
        }
    };
}

impl_simd_heap!(
    DaryHeapU32,
    u32,
    16,
    max_index_u32x16_avx2,
    max_index_u32x16_avx512
);
impl_simd_heap!(
    DaryHeapI32,
    u32,
    16,
    max_index_u32x16_avx2,
    max_index_u32x16_avx512
);
impl_simd_heap!(
    DaryHeapU64,
    u64,
    8,
    max_index_u64x8_avx2,
    max_index_u64x8_avx512
);
impl_simd_heap!(
    DaryHeapI64,
    u64,
    8,
    max_index_u64x8_avx2,
    max_index_u64x8_avx512
);

macro_rules! impl_u128_heap {
    ($name:ident) => {
        impl $name {
            #[inline(always)]
            fn sift_down_scalar(&mut self, hole: usize, value: u128) {
                self.sift_down_by(hole, value, max_index_u128)
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn sift_down_avx2(&mut self, hole: usize, value: u128) {
                self.sift_down_by(hole, value, |block| unsafe {
                    simd::max_index_u128x4_avx2(&block.low, &block.high)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2,avx512f,avx512vl")]
            unsafe fn sift_down_avx512(&mut self, hole: usize, value: u128) {
                self.sift_down_by(hole, value, |block| unsafe {
                    simd::max_index_u128x4_avx512(&block.low, &block.high)
                })
            }

            #[inline]
            fn sift_down(&mut self, hole: usize, value: u128) {
                #[cfg(target_arch = "x86_64")]
                match self.backend {
                    SimdBackend::Scalar => self.sift_down_scalar(hole, value),
                    // SAFETY: automatic construction only selects supported instruction sets;
                    // explicit construction is private and restricted to tests and benchmarks.
                    SimdBackend::Avx2 => unsafe { self.sift_down_avx2(hole, value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.sift_down_avx512(hole, value) },
                }
                #[cfg(not(target_arch = "x86_64"))]
                self.sift_down_scalar(hole, value);
            }

            #[inline]
            fn sift_down_after_pop(&mut self, hole: usize, value: u128) {
                #[cfg(target_arch = "x86_64")]
                match self.backend {
                    SimdBackend::Scalar => self.sift_down_scalar(hole, value),
                    // Scalar selection avoids SIMD setup overhead below this crossover.
                    SimdBackend::Avx2 if self.len < 1 << 18 => self.sift_down_scalar(hole, value),
                    // SAFETY: automatic construction only selects supported instruction sets;
                    // explicit construction is private and restricted to tests and benchmarks.
                    SimdBackend::Avx2 => unsafe { self.sift_down_avx2(hole, value) },
                    // Scalar selection avoids SIMD setup overhead below this crossover.
                    SimdBackend::Avx512 if self.len < 1 << 15 => self.sift_down_scalar(hole, value),
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.sift_down_avx512(hole, value) },
                }
                #[cfg(not(target_arch = "x86_64"))]
                self.sift_down_scalar(hole, value);
            }
        }
    };
}

impl_u128_heap!(DaryHeapU128);
impl_u128_heap!(DaryHeapI128);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    #[cfg(target_arch = "x86_64")]
    use crate::tools::avx512_supported;
    use std::collections::BinaryHeap;

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
    fn test_dary_heap() {
        let mut rng = Xorshift::default();
        for len in [0, 1, 7, 8, 15, 16, 17, 255, 256, 257, 4095, 4096, 4097] {
            let mut values: Vec<_> = (0..len).map(|_| rng.rand64() as u32).collect();
            values.extend([0, u32::MAX, u32::MAX]);
            for backend in backends() {
                let mut actual = DaryHeapU32::build(values.clone(), backend);
                let mut expected = BinaryHeap::from(values.clone());
                while !expected.is_empty() {
                    assert_eq!(actual.pop(), expected.pop());
                }
                assert_eq!(actual.pop(), None);
            }
        }

        for backend in backends() {
            let mut actual = DaryHeapU32::empty(10_000, backend);
            let mut expected = BinaryHeap::new();
            for _ in 0..20_000 {
                match rng.rand(3) {
                    0 => {
                        let value = rng.rand64() as u32;
                        actual.push(value);
                        expected.push(value);
                    }
                    1 => assert_eq!(actual.pop(), expected.pop()),
                    _ => {
                        let value = rng.rand64() as u32;
                        let old = expected.pop();
                        expected.push(value);
                        assert_eq!(actual.replace(value), old);
                    }
                }
                assert_eq!(actual.peek(), expected.peek().copied());
                assert_eq!(actual.len(), expected.len());
            }
        }

        for backend in backends() {
            let values: Vec<_> = (0..4097)
                .map(|_| rng.rand64() as i32)
                .chain([i32::MIN, 0, i32::MAX, i32::MAX])
                .collect();
            let mut actual = DaryHeapI32::build(values.clone(), backend);
            let mut expected = BinaryHeap::from(values);
            while !expected.is_empty() {
                assert_eq!(actual.pop(), expected.pop());
            }
        }

        macro_rules! check_heap {
            ($heap:ty, $value:ty, $values:expr $(, $backend:expr)?) => {{
                let values: Vec<$value> = ($values).collect();
                let mut actual = <$heap>::build(values.clone() $(, $backend)?);
                let mut expected = BinaryHeap::from(values);
                while !expected.is_empty() {
                    assert_eq!(actual.pop(), expected.pop());
                }
            }};
        }

        for backend in backends() {
            check_heap!(
                DaryHeapU64,
                u64,
                (0..1025).map(|_| rng.rand64()).chain([0, u64::MAX]),
                backend
            );
            check_heap!(
                DaryHeapI64,
                i64,
                (0..1025)
                    .map(|_| rng.rand64() as i64)
                    .chain([i64::MIN, i64::MAX]),
                backend
            );
        }
        for backend in backends() {
            check_heap!(
                DaryHeapU128,
                u128,
                (0..1025)
                    .map(|_| (rng.rand64() as u128) << 64 | rng.rand64() as u128)
                    .chain([0, u128::MAX]),
                backend
            );
            check_heap!(
                DaryHeapI128,
                i128,
                (0..1025)
                    .map(|_| ((rng.rand64() as u128) << 64 | rng.rand64() as u128) as i128)
                    .chain([i128::MIN, i128::MAX]),
                backend
            );
        }

        let mut actual = DaryHeapU32::from(vec![1, 2, 3]);
        actual.clear();
        assert!(actual.is_empty());
        assert_eq!(actual.replace(4), None);
        assert_eq!(actual.peek(), Some(4));
    }
}
