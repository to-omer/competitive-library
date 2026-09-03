use super::SimdBackend;
#[cfg(target_arch = "x86_64")]
use super::{avx512_enabled, simd};
use std::{marker::PhantomData, ops::Range};

#[inline]
fn static_search_backend(bits: u32) -> SimdBackend {
    #[cfg(target_arch = "x86_64")]
    {
        if avx512_enabled()
            && is_x86_feature_detected!("avx512f")
            && (bits != 16 || is_x86_feature_detected!("avx512bw"))
        {
            return SimdBackend::Avx512;
        }
        if is_x86_feature_detected!("avx2") {
            return SimdBackend::Avx2;
        }
    }
    let _ = bits;
    SimdBackend::Scalar
}

/// Maps a key to an unsigned integer with exactly the same ordering.
///
/// `BITS` must be 8, 16, 32, 64, or 128. `encode` must be deterministic, fit in
/// `BITS`, and satisfy `a.cmp(&b) == a.encode().cmp(&b.encode())`.
pub trait SimdKey: Copy + Ord {
    const BITS: u32;

    fn encode(self) -> u128;
}

macro_rules! impl_unsigned_simd_key {
    ($($value:ty),* $(,)?) => {
        $(
            impl SimdKey for $value {
                const BITS: u32 = <$value>::BITS;

                #[inline(always)]
                fn encode(self) -> u128 {
                    self as u128
                }
            }
        )*
    };
}

macro_rules! impl_signed_simd_key {
    ($(($signed:ty, $unsigned:ty)),* $(,)?) => {
        $(
            impl SimdKey for $signed {
                const BITS: u32 = <$signed>::BITS;

                #[inline(always)]
                fn encode(self) -> u128 {
                    ((self as $unsigned) ^ (1 as $unsigned << (<$signed>::BITS - 1))) as u128
                }
            }
        )*
    };
}

impl_unsigned_simd_key!(u8, u16, u32, u64, u128, usize);
impl_signed_simd_key!(
    (i8, u8),
    (i16, u16),
    (i32, u32),
    (i64, u64),
    (i128, u128),
    (isize, usize),
);

#[derive(Clone, Debug)]
enum DirectStaticSearch {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl DirectStaticSearch {
    fn build<K: SimdKey>(values: &[K], bits: u32) -> Self {
        assert!(values.len() <= u32::MAX as usize);
        if values.len() <= u16::MAX as usize {
            Self::U16(build_direct_positions(values, bits, |position| {
                position as u16
            }))
        } else {
            Self::U32(build_direct_positions(values, bits, |position| {
                position as u32
            }))
        }
    }

    #[inline(always)]
    fn lower_bound(&self, value: u128) -> usize {
        let value = usize::try_from(value).expect("SimdKey::encode exceeds usize");
        match self {
            Self::U16(positions) => positions[value] as usize,
            Self::U32(positions) => positions[value] as usize,
        }
    }

    #[inline(always)]
    fn upper_bound(&self, value: u128) -> usize {
        let value = usize::try_from(value)
            .ok()
            .and_then(|value| value.checked_add(1))
            .expect("SimdKey::encode exceeds usize");
        match self {
            Self::U16(positions) => positions[value] as usize,
            Self::U32(positions) => positions[value] as usize,
        }
    }

    #[inline(always)]
    fn contains(&self, value: u128) -> bool {
        let value = usize::try_from(value)
            .ok()
            .and_then(|value| value.checked_add(1).map(|next| (value, next)))
            .expect("SimdKey::encode exceeds usize");
        match self {
            Self::U16(positions) => positions[value.0] != positions[value.1],
            Self::U32(positions) => positions[value.0] != positions[value.1],
        }
    }
}

fn build_direct_positions<K, P>(values: &[K], bits: u32, position: impl Fn(usize) -> P) -> Vec<P>
where
    K: SimdKey,
    P: Copy,
{
    let len = (1 << bits) + 1;
    let mut positions = Vec::with_capacity(len);
    let maximum = (1u128 << bits) - 1;
    let mut previous: Option<(K, u128)> = None;
    for (index, &value) in values.iter().enumerate() {
        let encoded = value.encode();
        assert!(
            encoded <= maximum,
            "SimdKey::encode exceeds its declared width"
        );
        if let Some((previous_value, previous_encoded)) = previous {
            assert_eq!(
                previous_value.cmp(&value),
                previous_encoded.cmp(&encoded),
                "SimdKey::encode does not preserve order"
            );
        }
        if previous.is_none_or(|(_, previous)| previous != encoded) {
            positions.resize(encoded as usize + 1, position(index));
        }
        previous = Some((value, encoded));
    }
    positions.resize(len, position(values.len()));
    positions
}

#[repr(C, align(64))]
#[derive(Clone, Debug)]
struct SearchBlock<T, const B: usize>([T; B]);

#[derive(Clone, Debug)]
struct StaticSearchTree<T, const B: usize> {
    values: Vec<SearchBlock<T, B>>,
    len: usize,
    maximum: T,
    levels: Vec<Vec<SearchBlock<T, B>>>,
    #[cfg(target_arch = "x86_64")]
    backend: SimdBackend,
}

impl<T: Copy + Ord, const B: usize> StaticSearchTree<T, B> {
    fn build<K>(
        values: &[K],
        sentinel: T,
        maximum_encoded: u128,
        convert: impl Fn(u128) -> T,
        backend: SimdBackend,
    ) -> Self
    where
        K: SimdKey,
    {
        let _ = &backend;
        let len = values.len();
        let mut previous: Option<(K, T)> = None;
        let mut separators = Vec::with_capacity(values.len().div_ceil(B));
        let mut blocks = Vec::with_capacity(separators.capacity());
        for chunk in values.chunks(B) {
            let mut block = [sentinel; B];
            for (index, &value) in chunk.iter().enumerate() {
                let encoded = value.encode();
                assert!(
                    encoded <= maximum_encoded,
                    "SimdKey::encode exceeds its declared width"
                );
                let encoded = convert(encoded);
                if let Some((previous_value, previous_encoded)) = previous {
                    assert!(
                        previous_value.cmp(&value) == previous_encoded.cmp(&encoded),
                        "SimdKey::encode does not preserve order"
                    );
                }
                previous = Some((value, encoded));
                block[index] = encoded;
            }
            separators.push(block[chunk.len() - 1]);
            blocks.push(SearchBlock(block));
        }
        let maximum = separators.last().copied().unwrap_or(sentinel);
        let mut levels = Vec::new();
        while separators.len() > 1 {
            let mut blocks = Vec::with_capacity(separators.len().div_ceil(B));
            let mut next = Vec::with_capacity(blocks.capacity());
            for chunk in separators.chunks(B) {
                let mut block = [sentinel; B];
                block[..chunk.len()].copy_from_slice(chunk);
                blocks.push(SearchBlock(block));
                next.push(chunk[chunk.len() - 1]);
            }
            levels.push(blocks);
            separators = next;
        }
        Self {
            values: blocks,
            len,
            maximum,
            levels,
            #[cfg(target_arch = "x86_64")]
            backend,
        }
    }

    #[inline(always)]
    fn descend<F>(&self, value: T, mut position: F) -> usize
    where
        F: FnMut(&[T; B], T) -> usize,
    {
        let mut block = 0;
        for level in self.levels.iter().rev() {
            // SAFETY: each separator is the maximum of one real child group. The public entry
            // point excludes queries beyond the global maximum, so the first matching separator
            // selects a real group at every level.
            let values = &unsafe { level.get_unchecked(block) }.0;
            block = block * B + position(values, value);
        }
        let values = &unsafe { self.values.get_unchecked(block) }.0;
        (block * B + position(values, value)).min(self.len)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> T {
        unsafe {
            *self
                .values
                .get_unchecked(index / B)
                .0
                .get_unchecked(index % B)
        }
    }

    #[inline(always)]
    fn descend_batch<F>(&self, values: &[T; 16], mut position: F) -> [usize; 16]
    where
        F: FnMut(&[T; B], T) -> usize,
    {
        let mut blocks = [0; 16];
        for level in self.levels.iter().rev() {
            for index in 0..16 {
                // SAFETY: each query is capped at the global maximum before descent. As in
                // `descend`, every selected separator therefore names a real child group.
                let block_values = &unsafe { level.get_unchecked(blocks[index]) }.0;
                blocks[index] = blocks[index] * B + position(block_values, values[index]);
            }
        }
        for index in 0..16 {
            let block = blocks[index];
            let block_values = &unsafe { self.values.get_unchecked(block) }.0;
            blocks[index] = (block * B + position(block_values, values[index])).min(self.len);
        }
        blocks
    }

    #[inline(always)]
    fn lower_bound_scalar(&self, value: T) -> usize {
        self.descend(value, |values, value| {
            values.partition_point(|&current| current < value)
        })
    }

    #[inline(always)]
    fn upper_bound_scalar(&self, value: T) -> usize {
        self.descend(value, |values, value| {
            values.partition_point(|&current| current <= value)
        })
    }

    #[inline(always)]
    fn lower_bound_batch_scalar(&self, values: &[T; 16]) -> [usize; 16] {
        self.descend_batch(values, |values, value| {
            values.partition_point(|&current| current < value)
        })
    }

    #[inline(always)]
    fn upper_bound_batch_scalar(&self, values: &[T; 16]) -> [usize; 16] {
        self.descend_batch(values, |values, value| {
            values.partition_point(|&current| current <= value)
        })
    }
}

macro_rules! impl_static_search_tree {
    (
        $value:ty,
        $branch:expr,
        $first_ge_avx2:ident,
        $first_gt_avx2:ident,
        $first_ge_avx512:ident,
        $first_gt_avx512:ident,
        $avx512_features:literal
    ) => {
        impl StaticSearchTree<$value, $branch> {
            #[inline]
            fn lower_bound(&self, value: $value) -> usize {
                if self.len == 0 || value > self.maximum {
                    return self.len;
                }
                #[cfg(target_arch = "x86_64")]
                return match self.backend {
                    SimdBackend::Scalar => self.lower_bound_scalar(value),
                    // SAFETY: `simd_backend` only selects supported instruction sets. Tests and
                    // standalone benchmarks pass supported backends to the private constructor.
                    SimdBackend::Avx2 => unsafe { self.lower_bound_avx2(value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.lower_bound_avx512(value) },
                };
                #[cfg(not(target_arch = "x86_64"))]
                self.lower_bound_scalar(value)
            }

            #[inline]
            fn upper_bound(&self, value: $value) -> usize {
                if self.len == 0 {
                    return 0;
                }
                if value >= self.maximum {
                    return self.len;
                }
                #[cfg(target_arch = "x86_64")]
                return match self.backend {
                    SimdBackend::Scalar => self.upper_bound_scalar(value),
                    // SAFETY: `simd_backend` only selects supported instruction sets. Tests and
                    // standalone benchmarks pass supported backends to the private constructor.
                    SimdBackend::Avx2 => unsafe { self.upper_bound_avx2(value) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.upper_bound_avx512(value) },
                };
                #[cfg(not(target_arch = "x86_64"))]
                self.upper_bound_scalar(value)
            }

            #[inline]
            fn contains(&self, value: $value) -> bool {
                let index = self.lower_bound(value);
                index < self.len && self.get(index) == value
            }

            #[inline]
            fn lower_bound_batch(&self, values: &[$value; 16]) -> [usize; 16] {
                if self.len == 0 {
                    return [0; 16];
                }
                let mut values = *values;
                let mut beyond = [false; 16];
                for index in 0..16 {
                    beyond[index] = values[index] > self.maximum;
                    values[index] = values[index].min(self.maximum);
                }
                #[cfg(target_arch = "x86_64")]
                let mut result = match self.backend {
                    SimdBackend::Scalar => self.lower_bound_batch_scalar(&values),
                    // SAFETY: `simd_backend` only selects supported instruction sets.
                    SimdBackend::Avx2 => unsafe { self.lower_bound_batch_avx2(&values) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.lower_bound_batch_avx512(&values) },
                };
                #[cfg(not(target_arch = "x86_64"))]
                let mut result = self.lower_bound_batch_scalar(&values);
                for index in 0..16 {
                    if beyond[index] {
                        result[index] = self.len;
                    }
                }
                result
            }

            #[inline]
            fn upper_bound_batch(&self, values: &[$value; 16]) -> [usize; 16] {
                if self.len == 0 {
                    return [0; 16];
                }
                let mut values = *values;
                let mut beyond = [false; 16];
                for index in 0..16 {
                    beyond[index] = values[index] >= self.maximum;
                }
                let Some(value) = values.iter().copied().find(|&value| value < self.maximum) else {
                    return [self.len; 16];
                };
                for index in 0..16 {
                    if beyond[index] {
                        values[index] = value;
                    }
                }
                #[cfg(target_arch = "x86_64")]
                let mut result = match self.backend {
                    SimdBackend::Scalar => self.upper_bound_batch_scalar(&values),
                    // SAFETY: `simd_backend` only selects supported instruction sets.
                    SimdBackend::Avx2 => unsafe { self.upper_bound_batch_avx2(&values) },
                    // SAFETY: same as above.
                    SimdBackend::Avx512 => unsafe { self.upper_bound_batch_avx512(&values) },
                };
                #[cfg(not(target_arch = "x86_64"))]
                let mut result = self.upper_bound_batch_scalar(&values);
                for index in 0..16 {
                    if beyond[index] {
                        result[index] = self.len;
                    }
                }
                result
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn lower_bound_avx2(&self, value: $value) -> usize {
                self.descend(value, |values, value| unsafe {
                    simd::$first_ge_avx2(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn upper_bound_avx2(&self, value: $value) -> usize {
                self.descend(value, |values, value| unsafe {
                    simd::$first_gt_avx2(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn lower_bound_batch_avx2(&self, values: &[$value; 16]) -> [usize; 16] {
                self.descend_batch(values, |values, value| unsafe {
                    simd::$first_ge_avx2(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = "avx2")]
            unsafe fn upper_bound_batch_avx2(&self, values: &[$value; 16]) -> [usize; 16] {
                self.descend_batch(values, |values, value| unsafe {
                    simd::$first_gt_avx2(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = $avx512_features)]
            unsafe fn lower_bound_avx512(&self, value: $value) -> usize {
                self.descend(value, |values, value| unsafe {
                    simd::$first_ge_avx512(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = $avx512_features)]
            unsafe fn upper_bound_avx512(&self, value: $value) -> usize {
                self.descend(value, |values, value| unsafe {
                    simd::$first_gt_avx512(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = $avx512_features)]
            unsafe fn lower_bound_batch_avx512(&self, values: &[$value; 16]) -> [usize; 16] {
                self.descend_batch(values, |values, value| unsafe {
                    simd::$first_ge_avx512(values, value)
                })
            }

            #[cfg(target_arch = "x86_64")]
            #[target_feature(enable = $avx512_features)]
            unsafe fn upper_bound_batch_avx512(&self, values: &[$value; 16]) -> [usize; 16] {
                self.descend_batch(values, |values, value| unsafe {
                    simd::$first_gt_avx512(values, value)
                })
            }
        }
    };
}

impl_static_search_tree!(
    u16,
    32,
    first_ge_u16x32_avx2,
    first_gt_u16x32_avx2,
    first_ge_u16x32_avx512,
    first_gt_u16x32_avx512,
    "avx512f,avx512bw"
);
impl_static_search_tree!(
    u32,
    16,
    first_ge_u32x16_avx2,
    first_gt_u32x16_avx2,
    first_ge_u32x16_avx512,
    first_gt_u32x16_avx512,
    "avx512f"
);
impl_static_search_tree!(
    u64,
    8,
    first_ge_u64x8_avx2,
    first_gt_u64x8_avx2,
    first_ge_u64x8_avx512,
    first_gt_u64x8_avx512,
    "avx512f"
);

impl StaticSearchTree<u128, 4> {
    #[inline]
    fn lower_bound(&self, value: u128) -> usize {
        if self.len == 0 || value > self.maximum {
            self.len
        } else {
            self.descend(value, |values, value| {
                (values[0] < value) as usize
                    + (values[1] < value) as usize
                    + (values[2] < value) as usize
                    + (values[3] < value) as usize
            })
        }
    }

    #[inline]
    fn upper_bound(&self, value: u128) -> usize {
        if self.len == 0 || value >= self.maximum {
            self.len
        } else {
            self.descend(value, |values, value| {
                (values[0] <= value) as usize
                    + (values[1] <= value) as usize
                    + (values[2] <= value) as usize
                    + (values[3] <= value) as usize
            })
        }
    }

    #[inline]
    fn contains(&self, value: u128) -> bool {
        let index = self.lower_bound(value);
        index < self.len && self.get(index) == value
    }

    #[inline]
    fn lower_bound_batch(&self, values: &[u128; 16]) -> [usize; 16] {
        if self.len == 0 {
            return [0; 16];
        }
        let mut values = *values;
        let mut beyond = [false; 16];
        for index in 0..16 {
            beyond[index] = values[index] > self.maximum;
            values[index] = values[index].min(self.maximum);
        }
        let mut result = self.descend_batch(&values, |values, value| {
            (values[0] < value) as usize
                + (values[1] < value) as usize
                + (values[2] < value) as usize
                + (values[3] < value) as usize
        });
        for index in 0..16 {
            if beyond[index] {
                result[index] = self.len;
            }
        }
        result
    }

    #[inline]
    fn upper_bound_batch(&self, values: &[u128; 16]) -> [usize; 16] {
        if self.len == 0 {
            return [0; 16];
        }
        let mut values = *values;
        let mut beyond = [false; 16];
        for index in 0..16 {
            beyond[index] = values[index] >= self.maximum;
        }
        let Some(value) = values.iter().copied().find(|&value| value < self.maximum) else {
            return [self.len; 16];
        };
        for index in 0..16 {
            if beyond[index] {
                values[index] = value;
            }
        }
        let mut result = self.descend_batch(&values, |values, value| {
            (values[0] <= value) as usize
                + (values[1] <= value) as usize
                + (values[2] <= value) as usize
                + (values[3] <= value) as usize
        });
        for index in 0..16 {
            if beyond[index] {
                result[index] = self.len;
            }
        }
        result
    }
}

fn search_batch<K, T>(
    values: &[K],
    output: &mut [usize],
    convert: impl Fn(u128) -> T,
    single: impl Fn(T) -> usize,
    batch: impl Fn(&[T; 16]) -> [usize; 16],
) where
    K: SimdKey,
    T: Copy,
{
    let mut offset = 0;
    while offset + 16 <= values.len() {
        let values = std::array::from_fn(|index| convert(values[offset + index].encode()));
        output[offset..offset + 16].copy_from_slice(&batch(&values));
        offset += 16;
    }
    let remaining = values.len() - offset;
    if remaining >= 8 {
        let mut encoded = [convert(values[offset].encode()); 16];
        for index in 1..remaining {
            encoded[index] = convert(values[offset + index].encode());
        }
        let positions = batch(&encoded);
        output[offset..].copy_from_slice(&positions[..remaining]);
    } else {
        for (&value, position) in values[offset..].iter().zip(&mut output[offset..]) {
            *position = single(convert(value.encode()));
        }
    }
}

#[derive(Clone, Debug)]
enum StaticSearchStorage {
    Direct(DirectStaticSearch),
    U16(StaticSearchTree<u16, 32>),
    U32(StaticSearchTree<u32, 16>),
    U64(StaticSearchTree<u64, 8>),
    U128(StaticSearchTree<u128, 4>),
}

impl StaticSearchStorage {
    #[inline(always)]
    fn lower_bound(&self, value: u128) -> usize {
        match self {
            Self::Direct(search) => search.lower_bound(value),
            Self::U16(search) => search.lower_bound(
                u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U32(search) => search.lower_bound(
                u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U64(search) => search.lower_bound(
                u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U128(search) => search.lower_bound(value),
        }
    }

    #[inline(always)]
    fn upper_bound(&self, value: u128) -> usize {
        match self {
            Self::Direct(search) => search.upper_bound(value),
            Self::U16(search) => search.upper_bound(
                u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U32(search) => search.upper_bound(
                u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U64(search) => search.upper_bound(
                u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U128(search) => search.upper_bound(value),
        }
    }

    #[inline(always)]
    fn contains(&self, value: u128) -> bool {
        match self {
            Self::Direct(search) => search.contains(value),
            Self::U16(search) => search.contains(
                u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U32(search) => search.contains(
                u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U64(search) => search.contains(
                u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),
            ),
            Self::U128(search) => search.contains(value),
        }
    }

    fn lower_bound_batch<K: SimdKey>(&self, values: &[K], output: &mut [usize]) {
        match self {
            Self::Direct(search) => {
                for (&value, position) in values.iter().zip(output) {
                    *position = search.lower_bound(value.encode());
                }
            }
            Self::U16(search) => search_batch(
                values,
                output,
                |value| u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),
                |value| search.lower_bound(value),
                |values| search.lower_bound_batch(values),
            ),
            Self::U32(search) => search_batch(
                values,
                output,
                |value| u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),
                |value| search.lower_bound(value),
                |values| search.lower_bound_batch(values),
            ),
            Self::U64(search) => search_batch(
                values,
                output,
                |value| u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),
                |value| search.lower_bound(value),
                |values| search.lower_bound_batch(values),
            ),
            Self::U128(search) => search_batch(
                values,
                output,
                |value| value,
                |value| search.lower_bound(value),
                |values| search.lower_bound_batch(values),
            ),
        }
    }

    fn upper_bound_batch<K: SimdKey>(&self, values: &[K], output: &mut [usize]) {
        match self {
            Self::Direct(search) => {
                for (&value, position) in values.iter().zip(output) {
                    *position = search.upper_bound(value.encode());
                }
            }
            Self::U16(search) => search_batch(
                values,
                output,
                |value| u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),
                |value| search.upper_bound(value),
                |values| search.upper_bound_batch(values),
            ),
            Self::U32(search) => search_batch(
                values,
                output,
                |value| u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),
                |value| search.upper_bound(value),
                |values| search.upper_bound_batch(values),
            ),
            Self::U64(search) => search_batch(
                values,
                output,
                |value| u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),
                |value| search.upper_bound(value),
                |values| search.upper_bound_batch(values),
            ),
            Self::U128(search) => search_batch(
                values,
                output,
                |value| value,
                |value| search.upper_bound(value),
                |values| search.upper_bound_batch(values),
            ),
        }
    }
}

/// A static search index over sorted integer or integer-encoded keys.
///
/// The index adds build time and storage. For a small number of searches, search the sorted slice
/// directly instead.
#[derive(Clone, Debug)]
pub struct StaticSearch<K> {
    storage: StaticSearchStorage,
    len: usize,
    marker: PhantomData<fn() -> K>,
}

impl<K: SimdKey> StaticSearch<K> {
    /// Builds an index over sorted `values`.
    ///
    /// # Panics
    ///
    /// Panics if `values` is not sorted or `SimdKey` does not satisfy its contract.
    pub fn from_sorted(values: &[K]) -> Self {
        Self::build(values, static_search_backend(K::BITS), false)
    }

    /// Builds a direct lookup table over sorted 8-bit or 16-bit `values`.
    ///
    /// This layout uses a fixed table of 257 or 65,537 positions and is intended
    /// for query-heavy workloads.
    ///
    /// # Panics
    ///
    /// Panics if `K::BITS` is neither 8 nor 16, or if `values` is not sorted.
    pub fn from_sorted_direct(values: &[K]) -> Self {
        assert!(matches!(K::BITS, 8 | 16));
        Self::build(values, static_search_backend(K::BITS), true)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the first index whose value is greater than or equal to `value`.
    #[inline]
    pub fn lower_bound(&self, value: K) -> usize {
        self.storage.lower_bound(value.encode())
    }

    /// Returns one past the last index whose value is less than or equal to `value`.
    #[inline]
    pub fn upper_bound(&self, value: K) -> usize {
        self.storage.upper_bound(value.encode())
    }

    /// Writes the first index greater than or equal to each value into `output`.
    ///
    /// # Panics
    ///
    /// Panics if `values` and `output` have different lengths.
    pub fn lower_bound_batch(&self, values: &[K], output: &mut [usize]) {
        assert_eq!(values.len(), output.len());
        self.storage.lower_bound_batch(values, output);
    }

    /// Writes one past the last index less than or equal to each value into `output`.
    ///
    /// # Panics
    ///
    /// Panics if `values` and `output` have different lengths.
    pub fn upper_bound_batch(&self, values: &[K], output: &mut [usize]) {
        assert_eq!(values.len(), output.len());
        self.storage.upper_bound_batch(values, output);
    }

    #[inline]
    pub fn range(&self, value: K) -> Range<usize> {
        let value = value.encode();
        self.storage.lower_bound(value)..self.storage.upper_bound(value)
    }

    #[inline]
    pub fn contains(&self, value: K) -> bool {
        self.storage.contains(value.encode())
    }

    fn build(values: &[K], backend: SimdBackend, direct: bool) -> Self {
        assert!(matches!(K::BITS, 8 | 16 | 32 | 64 | 128));
        assert!(values.windows(2).all(|pair| pair[0] <= pair[1]));
        let len = values.len();
        let storage = match K::BITS {
            8 => StaticSearchStorage::Direct(DirectStaticSearch::build(values, K::BITS)),
            16 => {
                if direct {
                    StaticSearchStorage::Direct(DirectStaticSearch::build(values, K::BITS))
                } else {
                    StaticSearchStorage::U16(StaticSearchTree::build(
                        values,
                        u16::MAX,
                        u16::MAX as u128,
                        |value| value as u16,
                        backend,
                    ))
                }
            }
            32 => StaticSearchStorage::U32(StaticSearchTree::build(
                values,
                u32::MAX,
                u32::MAX as u128,
                |value| value as u32,
                backend,
            )),
            64 => StaticSearchStorage::U64(StaticSearchTree::build(
                values,
                u64::MAX,
                u64::MAX as u128,
                |value| value as u64,
                backend,
            )),
            128 => StaticSearchStorage::U128(StaticSearchTree::build(
                values,
                u128::MAX,
                u128::MAX,
                |value| value,
                backend,
            )),
            _ => unreachable!(),
        };
        Self {
            storage,
            len,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    #[cfg(target_arch = "x86_64")]
    use crate::tools::avx512_supported;
    use std::fmt::Debug;

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

    fn check<K>(values: Vec<K>, queries: &[K])
    where
        K: SimdKey + Debug,
    {
        let verify = |search: StaticSearch<K>| {
            assert_eq!(search.len(), values.len());
            assert_eq!(search.is_empty(), values.is_empty());
            for &query in queries {
                let left = values.partition_point(|&value| value < query);
                let right = values.partition_point(|&value| value <= query);
                assert_eq!(search.lower_bound(query), left);
                assert_eq!(search.upper_bound(query), right);
                assert_eq!(search.range(query), left..right);
                assert_eq!(search.contains(query), left != right);
            }
            for len in [0, 1, 7, 8, 15, 16, 17, queries.len()] {
                let queries = &queries[..len.min(queries.len())];
                let mut left = vec![0; queries.len()];
                let mut right = vec![0; queries.len()];
                search.lower_bound_batch(queries, &mut left);
                search.upper_bound_batch(queries, &mut right);
                assert_eq!(
                    left,
                    queries
                        .iter()
                        .map(|query| values.partition_point(|value| value < query))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    right,
                    queries
                        .iter()
                        .map(|query| values.partition_point(|value| value <= query))
                        .collect::<Vec<_>>()
                );
            }
        };
        if K::BITS == 8 {
            verify(StaticSearch::build(&values, SimdBackend::Scalar, false));
        } else {
            for backend in backends() {
                verify(StaticSearch::build(&values, backend, false));
            }
        }
        if K::BITS == 16 {
            verify(StaticSearch::build(&values, SimdBackend::Scalar, true));
        }
    }

    fn check_random<K>(
        rng: &mut Xorshift,
        mut random: impl FnMut(&mut Xorshift) -> K,
        boundaries: &[K],
    ) where
        K: SimdKey + Debug,
    {
        for len in [0, 1, 7, 8, 15, 16, 17, 31, 32, 33, 255, 256, 257, 4097] {
            let mut values: Vec<_> = (0..len).map(|_| random(rng)).collect();
            for (value, &boundary) in values.iter_mut().zip(boundaries) {
                *value = boundary;
            }
            if values.len() > boundaries.len() {
                values[boundaries.len()] = boundaries[0];
            }
            values.sort_unstable();
            let mut queries: Vec<_> = (0..263).map(|_| random(rng)).collect();
            queries.extend_from_slice(boundaries);
            check(values, &queries);
        }
    }

    #[test]
    fn test_static_search() {
        let mut rng = Xorshift::default();
        check_random(&mut rng, |rng| rng.rand64() as u8, &[u8::MIN, 1, u8::MAX]);
        check_random(
            &mut rng,
            |rng| rng.rand64() as i8,
            &[i8::MIN, -1, 0, 1, i8::MAX],
        );
        check_random(
            &mut rng,
            |rng| rng.rand64() as u16,
            &[u16::MIN, 1, u16::MAX],
        );
        check_random(
            &mut rng,
            |rng| rng.rand64() as i16,
            &[i16::MIN, -1, 0, 1, i16::MAX],
        );
        check_random(
            &mut rng,
            |rng| rng.rand64() as u32,
            &[u32::MIN, 1, u32::MAX],
        );
        check_random(
            &mut rng,
            |rng| rng.rand64() as i32,
            &[i32::MIN, -1, 0, 1, i32::MAX],
        );
        check_random(&mut rng, |rng| rng.rand64(), &[u64::MIN, 1, u64::MAX]);
        check_random(
            &mut rng,
            |rng| rng.rand64() as i64,
            &[i64::MIN, -1, 0, 1, i64::MAX],
        );
        check_random(
            &mut rng,
            |rng| (rng.rand64() as u128) << 64 | rng.rand64() as u128,
            &[u128::MIN, 1, u128::MAX],
        );
        check_random(
            &mut rng,
            |rng| ((rng.rand64() as u128) << 64 | rng.rand64() as u128) as i128,
            &[i128::MIN, -1, 0, 1, i128::MAX],
        );
        check_random(
            &mut rng,
            |rng| rng.rand64() as usize,
            &[usize::MIN, 1, usize::MAX],
        );
        check_random(
            &mut rng,
            |rng| rng.rand64() as isize,
            &[isize::MIN, -1, 0, 1, isize::MAX],
        );

        let mut values: Vec<_> = (0..=u16::MAX).map(|_| rng.rand64() as u16).collect();
        values.sort_unstable();
        let queries: Vec<_> = (0..263).map(|_| rng.rand64() as u16).collect();
        check(values, &queries);

        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        struct Pair(u32, u32);

        impl SimdKey for Pair {
            const BITS: u32 = 64;

            fn encode(self) -> u128 {
                ((self.0 as u128) << 32) | self.1 as u128
            }
        }

        check_random(
            &mut rng,
            |rng| Pair(rng.rand64() as u32, rng.rand64() as u32),
            &[Pair(0, 0), Pair(u32::MAX, u32::MAX)],
        );
    }
}
