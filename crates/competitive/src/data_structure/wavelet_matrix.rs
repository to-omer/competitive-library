use super::{
    AbelianGroup, BinaryIndexedTree, BitVector, Compressor, OrderedCompressor,
    RankSelectDictionaries, VecCompress,
};
use std::{
    mem::{self, MaybeUninit},
    ops::Range,
};

const ACCESS: u8 = 0;
const RANK: u8 = 1;
const QUANTILE: u8 = 2;
const RANK_LESSTHAN: u8 = 3;

#[derive(Debug, Clone)]
#[repr(C)]
struct WaveletMatrixQuadBlock {
    lo: u64,
    hi: u64,
    rank: [u32; 4],
}

#[derive(Debug, Clone)]
struct WaveletMatrixQuadVector {
    blocks: Vec<WaveletMatrixQuadBlock>,
    starts: [usize; 4],
    select_samples: [Vec<usize>; 4],
}

impl WaveletMatrixQuadVector {
    fn from_words(low: &[u64], high: Option<&[u64]>, len: usize) -> Self {
        let mut blocks = Vec::with_capacity(len / 64 + 1);
        let mut rank = [0; 4];
        for (word, &lo) in low.iter().enumerate() {
            let count = (len - word * 64).min(64);
            let hi = high.map_or(0, |high| high[word]);
            blocks.push(WaveletMatrixQuadBlock { lo, hi, rank });
            let low = lo.count_ones();
            let high = hi.count_ones();
            let both = (lo & hi).count_ones();
            rank[0] += count as u32 - low - (high - both);
            rank[1] += low - both;
            rank[2] += high - both;
            rank[3] += both;
        }
        if len.is_multiple_of(64) {
            blocks.push(WaveletMatrixQuadBlock { lo: 0, hi: 0, rank });
        }
        // Two stable binary partitions order the children as 0, 2, 1, 3.
        let starts = [
            0,
            (rank[0] as usize + rank[2] as usize),
            rank[0] as usize,
            rank[0] as usize + rank[2] as usize + rank[1] as usize,
        ];
        let mut select_samples: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
        for (i, block) in blocks.iter().enumerate().take(len.div_ceil(64)) {
            for digit in 0..4 {
                let start = block.rank[digit] as usize;
                let end = blocks
                    .get(i + 1)
                    .map_or(rank[digit], |next| next.rank[digit])
                    as usize;
                if start.div_ceil(128) != end.div_ceil(128) {
                    select_samples[digit].push(i);
                }
            }
        }
        Self {
            blocks,
            starts,
            select_samples,
        }
    }

    #[inline]
    fn ranks(&self, position: usize) -> [usize; 4] {
        let block = &self.blocks[position / 64];
        let mask = !(u64::MAX << (position % 64));
        let low = (block.lo & mask).count_ones() as usize;
        let high = (block.hi & mask).count_ones() as usize;
        let both = (block.lo & block.hi & mask).count_ones() as usize;
        [
            block.rank[0] as usize + position % 64 - low - (high - both),
            block.rank[1] as usize + low - both,
            block.rank[2] as usize + high - both,
            block.rank[3] as usize + both,
        ]
    }

    #[inline]
    fn rank(&self, digit: usize, position: usize) -> usize {
        let block = &self.blocks[position / 64];
        let low = if digit & 1 != 0 { block.lo } else { !block.lo };
        let high = if digit & 2 != 0 { block.hi } else { !block.hi };
        block.rank[digit] as usize
            + (low & high & !(u64::MAX << (position % 64))).count_ones() as usize
    }

    fn select(&self, digit: usize, k: usize) -> usize {
        let sample = k / 128;
        let start = self.select_samples[digit][sample];
        let end = self.select_samples[digit]
            .get(sample + 1)
            .map_or(self.blocks.len(), |&word| word + 1);
        let word = start
            + self.blocks[start..end].partition_point(|block| block.rank[digit] as usize <= k)
            - 1;
        let block = &self.blocks[word];
        let lo = if digit & 1 != 0 { block.lo } else { !block.lo };
        let hi = if digit & 2 != 0 { block.hi } else { !block.hi };
        word * 64 + BitVector::select_word(lo & hi, k - block.rank[digit] as usize)
    }

    #[inline]
    fn access_rank(&self, position: usize) -> (usize, usize) {
        let block = &self.blocks[position / 64];
        let offset = position % 64;
        let digit =
            ((block.lo >> offset) & 1) as usize | (((block.hi >> offset) & 1) as usize) << 1;
        (digit, self.rank(digit, position))
    }
}

#[cfg(target_arch = "x86_64")]
mod simd {
    #![allow(unsafe_op_in_unsafe_fn)] // All entry points check the required CPU features.
    use super::BitVector;
    use std::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    pub unsafe fn pack_words(indices: &[u32], bit: usize) -> Vec<u64> {
        let mut result = Vec::with_capacity(indices.len().div_ceil(64));
        let shift = _mm_cvtsi32_si128((31 - bit) as i32);
        for chunk in indices.chunks(64) {
            let mut word = 0u64;
            let mut i = 0;
            while i + 8 <= chunk.len() {
                let v = _mm256_loadu_si256(chunk.as_ptr().add(i).cast());
                word |= (_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_sll_epi32(v, shift)))
                    as u64)
                    << i;
                i += 8;
            }
            for (j, &value) in chunk[i..].iter().enumerate() {
                word |= (((value >> bit) & 1) as u64) << (i + j);
            }
            result.push(word);
        }
        result
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn partition_avx2(indices: &[u32], words: &[u64], mut one: usize, next: &mut [u32]) {
        const PERM: [[i32; 8]; 256] = {
            let mut table = [[0; 8]; 256];
            let mut mask = 0;
            while mask < 256 {
                let mut pos = 0;
                let mut bit = 0;
                while bit < 2 {
                    let mut i = 0;
                    while i < 8 {
                        if (mask >> i) & 1 == bit {
                            table[mask][pos] = i;
                            pos += 1;
                        }
                        i += 1;
                    }
                    bit += 1;
                }
                mask += 1;
            }
            table
        };
        let order = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
        let mut zero = 0;
        for (chunk, &word) in indices.chunks(64).zip(words) {
            if word == 0 || word == u64::MAX {
                let at = if word == 0 { &mut zero } else { &mut one };
                next[*at..*at + chunk.len()].copy_from_slice(chunk);
                *at += chunk.len();
                continue;
            }
            let mut i = 0;
            while i + 8 <= chunk.len() {
                let value = _mm256_loadu_si256(chunk.as_ptr().add(i).cast());
                let mask = ((word >> i) & 255) as usize;
                let ones = mask.count_ones() as usize;
                let zeros = 8 - ones;
                let packed = _mm256_permutevar8x32_epi32(
                    value,
                    _mm256_loadu_si256(PERM[mask].as_ptr().cast()),
                );
                let rotated = _mm256_permutevar8x32_epi32(
                    packed,
                    _mm256_add_epi32(order, _mm256_set1_epi32(zeros as i32)),
                );
                _mm256_maskstore_epi32(
                    next.as_mut_ptr().add(zero).cast(),
                    _mm256_cmpgt_epi32(_mm256_set1_epi32(zeros as i32), order),
                    packed,
                );
                _mm256_maskstore_epi32(
                    next.as_mut_ptr().add(one).cast(),
                    _mm256_cmpgt_epi32(_mm256_set1_epi32(ones as i32), order),
                    rotated,
                );
                zero += zeros;
                one += ones;
                i += 8;
            }
            for (j, &value) in chunk[i..].iter().enumerate() {
                let bit = (word >> (i + j)) & 1 != 0;
                next[if bit { one } else { zero }] = value;
                zero += !bit as usize;
                one += bit as usize;
            }
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn popcount_avx2(value: __m256i) -> __m256i {
        let lookup = _mm256_setr_epi8(
            0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2,
            3, 3, 4,
        );
        let mask = _mm256_set1_epi8(15);
        let low = _mm256_shuffle_epi8(lookup, _mm256_and_si256(value, mask));
        let high = _mm256_shuffle_epi8(
            lookup,
            _mm256_and_si256(_mm256_srli_epi16::<4>(value), mask),
        );
        _mm256_sad_epu8(_mm256_add_epi8(low, high), _mm256_setzero_si256())
    }

    #[target_feature(enable = "avx512f")]
    #[inline]
    unsafe fn greater_avx512(a: __m512i, b: __m512i) -> __m512i {
        _mm512_maskz_set1_epi64(_mm512_cmpgt_epi64_mask(a, b), -1)
    }

    #[target_feature(enable = "avx512f")]
    #[inline]
    unsafe fn gather_avx512<const SCALE: i32>(base: *const i64, index: __m512i) -> __m512i {
        _mm512_i64gather_epi64::<SCALE>(index, base)
    }

    macro_rules! rank_lessthan {
        ($name:ident, $feature:literal, $lanes:literal,
         $load:ident, $store:ident, $set:ident, $gather:ident,
         $add:ident, $sub:ident, $and:ident, $andnot:ident, $or:ident,
         $left:ident, $right:ident, $gt:ident, $popcount:ident) => {
            #[target_feature(enable = $feature)]
            pub unsafe fn $name(vectors: &[BitVector], zeros: &[usize], states: &mut [[usize; 4]]) {
                for chunk in states.chunks_exact_mut($lanes) {
                    let mut starts = [0u64; $lanes];
                    let mut ends = [0u64; $lanes];
                    let mut keys = [0u64; $lanes];
                    let mut values = [0u64; $lanes];
                    for (i, state) in chunk.iter().enumerate() {
                        starts[i] = state[0] as u64;
                        ends[i] = state[1] as u64;
                        keys[i] = state[2] as u64;
                    }
                    let mut start = $load(starts.as_ptr().cast());
                    let mut end = $load(ends.as_ptr().cast());
                    let key = $load(keys.as_ptr().cast());
                    let mut value = $set(0);
                    let one = $set(1);
                    for (level, vector) in vectors.iter().enumerate() {
                        let d = vectors.len() - 1 - level;
                        let base = vector.blocks().as_ptr().cast::<i64>();
                        let rank1 = |position| {
                            let offset = $and(position, $set(63));
                            let index = $left($right(position, $set(6)), one);
                            let bits = $gather::<8>(base, index);
                            let prefix = $gather::<8>(base, $add(index, one));
                            $add(prefix, $popcount($and(bits, $sub($left(one, offset), one))))
                        };
                        let rank = rank1(start);
                        let end_rank = rank1(end);
                        let start0 = $sub(start, rank);
                        let end0 = $sub(end, end_rank);
                        let count = $sub(end0, start0);
                        let mask = $gt($and($right(key, $set(d as i64)), one), $set(0));
                        let zero = $set(zeros[level] as i64);
                        start = $or($andnot(mask, start0), $and(mask, $add(zero, rank)));
                        end = $or($andnot(mask, end0), $and(mask, $add(zero, end_rank)));
                        value = $add(value, $and(mask, count));
                    }
                    $store(values.as_mut_ptr().cast(), value);
                    for (i, state) in chunk.iter_mut().enumerate() {
                        state[3] = values[i] as usize;
                    }
                }
            }
        };
    }

    rank_lessthan!(
        rank_lessthan_avx2,
        "avx2",
        4,
        _mm256_loadu_si256,
        _mm256_storeu_si256,
        _mm256_set1_epi64x,
        _mm256_i64gather_epi64,
        _mm256_add_epi64,
        _mm256_sub_epi64,
        _mm256_and_si256,
        _mm256_andnot_si256,
        _mm256_or_si256,
        _mm256_sllv_epi64,
        _mm256_srlv_epi64,
        _mm256_cmpgt_epi64,
        popcount_avx2
    );
    rank_lessthan!(
        rank_lessthan_avx512,
        "avx512f,avx512vpopcntdq",
        8,
        _mm512_loadu_si512,
        _mm512_storeu_si512,
        _mm512_set1_epi64,
        gather_avx512,
        _mm512_add_epi64,
        _mm512_sub_epi64,
        _mm512_and_si512,
        _mm512_andnot_si512,
        _mm512_or_si512,
        _mm512_sllv_epi64,
        _mm512_srlv_epi64,
        greater_avx512,
        _mm512_popcnt_epi64
    );

    #[target_feature(enable = "avx512f,avx512vpopcntdq")]
    pub unsafe fn quad_avx512(
        layers: &[super::WaveletMatrixQuadVector],
        states: &mut [[usize; 4]],
    ) {
        for chunk in states.as_chunks_mut::<8>().0 {
            let mut starts = [0u64; 8];
            let mut ends = [0u64; 8];
            let mut keys = [0u64; 8];
            let mut result = [0u64; 8];
            for (i, s) in chunk.iter().enumerate() {
                starts[i] = s[0] as u64;
                ends[i] = s[1] as u64;
                keys[i] = s[2] as u64;
            }
            let mut start = _mm512_loadu_si512(starts.as_ptr().cast());
            let mut end = _mm512_loadu_si512(ends.as_ptr().cast());
            let mut key = _mm512_loadu_si512(keys.as_ptr().cast());
            let mut code = _mm512_set1_epi64(0);
            let one = _mm512_set1_epi64(1);
            macro_rules! blend {
                ($m:expr,$x:expr,$y:expr) => {
                    _mm512_or_si512(_mm512_andnot_si512($m, $x), _mm512_and_si512($m, $y))
                };
            }
            for layer in layers {
                let base = layer.blocks.as_ptr().cast::<i64>();
                let ranks = |pos| {
                    let offset = _mm512_and_si512(pos, _mm512_set1_epi64(63));
                    let mask = _mm512_sub_epi64(_mm512_sllv_epi64(one, offset), one);
                    let i = _mm512_sllv_epi64(
                        _mm512_srlv_epi64(pos, _mm512_set1_epi64(6)),
                        _mm512_set1_epi64(2),
                    );
                    let lo = _mm512_and_si512(gather_avx512::<8>(base, i), mask);
                    let hi =
                        _mm512_and_si512(gather_avx512::<8>(base, _mm512_add_epi64(i, one)), mask);
                    let a = _mm512_popcnt_epi64(lo);
                    let b = _mm512_popcnt_epi64(hi);
                    let c = _mm512_popcnt_epi64(_mm512_and_si512(lo, hi));
                    let p01 = gather_avx512::<8>(base, _mm512_add_epi64(i, _mm512_set1_epi64(2)));
                    let p23 = gather_avx512::<8>(base, _mm512_add_epi64(i, _mm512_set1_epi64(3)));
                    let r0 = _mm512_add_epi64(
                        _mm512_and_si512(p01, _mm512_set1_epi64(u32::MAX as i64)),
                        _mm512_add_epi64(_mm512_sub_epi64(_mm512_sub_epi64(offset, a), b), c),
                    );
                    let r1 = _mm512_add_epi64(
                        _mm512_srlv_epi64(p01, _mm512_set1_epi64(32)),
                        _mm512_sub_epi64(a, c),
                    );
                    let r2 = _mm512_add_epi64(
                        _mm512_and_si512(p23, _mm512_set1_epi64(u32::MAX as i64)),
                        _mm512_sub_epi64(b, c),
                    );
                    let r3 = _mm512_sub_epi64(_mm512_sub_epi64(_mm512_sub_epi64(pos, r0), r1), r2);
                    [r0, r1, r2, r3]
                };
                let l = ranks(start);
                let r = ranks(end);
                let low_count =
                    _mm512_sub_epi64(_mm512_add_epi64(r[0], r[1]), _mm512_add_epi64(l[0], l[1]));
                let high = greater_avx512(key, _mm512_sub_epi64(low_count, one));
                key = _mm512_sub_epi64(key, _mm512_and_si512(high, low_count));
                let l0 = blend!(high, l[0], l[2]);
                let r0 = blend!(high, r[0], r[2]);
                let l1 = blend!(high, l[1], l[3]);
                let r1 = blend!(high, r[1], r[3]);
                let count0 = _mm512_sub_epi64(r0, l0);
                let low = greater_avx512(key, _mm512_sub_epi64(count0, one));
                key = _mm512_sub_epi64(key, _mm512_and_si512(low, count0));
                let base0 = blend!(
                    high,
                    _mm512_set1_epi64(layer.starts[0] as i64),
                    _mm512_set1_epi64(layer.starts[2] as i64)
                );
                let base1 = blend!(
                    high,
                    _mm512_set1_epi64(layer.starts[1] as i64),
                    _mm512_set1_epi64(layer.starts[3] as i64)
                );
                let offset = blend!(low, base0, base1);
                start = _mm512_add_epi64(offset, blend!(low, l0, l1));
                end = _mm512_add_epi64(offset, blend!(low, r0, r1));
                code = _mm512_or_si512(
                    _mm512_sllv_epi64(code, _mm512_set1_epi64(2)),
                    _mm512_or_si512(
                        _mm512_and_si512(high, _mm512_set1_epi64(2)),
                        _mm512_and_si512(low, one),
                    ),
                );
            }
            _mm512_storeu_si512(result.as_mut_ptr().cast(), code);
            for (i, s) in chunk.iter_mut().enumerate() {
                s[3] = result[i] as usize;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaveletMatrix<T> {
    len: usize,
    bit_length: usize,
    zeros: Vec<usize>,
    bit_vectors: Vec<BitVector>,
    // Binary layers preserve callback coordinates and support weighted queries.
    quad_vectors: Vec<WaveletMatrixQuadVector>,
    compress: VecCompress<T>,
    #[cfg(target_arch = "x86_64")]
    backend: super::SimdBackend,
}

impl<T> WaveletMatrix<T>
where
    T: Ord + Clone,
{
    pub fn new(v: Vec<T>) -> Self {
        if v.len() <= u32::MAX as usize {
            #[cfg(target_arch = "x86_64")]
            let backend = super::simd_backend();
            Self::from_values(
                v,
                |i| i as u32,
                |i| i as usize,
                |indices, d| {
                    #[cfg(target_arch = "x86_64")]
                    if backend != super::SimdBackend::Scalar && is_x86_feature_detected!("avx2") {
                        // SAFETY: AVX2 is available.
                        return unsafe { simd::pack_words(indices, d) };
                    }
                    Self::pack_words(indices, |i| (i >> d) & 1 != 0)
                },
                |indices, words, zeros, next| {
                    #[cfg(target_arch = "x86_64")]
                    if backend != super::SimdBackend::Scalar && is_x86_feature_detected!("avx2") {
                        // SAFETY: AVX2 is available, and the partition buffers have equal length.
                        unsafe { simd::partition_avx2(indices, words, zeros, next) };
                        return;
                    }
                    Self::partition(indices, words, zeros, next);
                },
            )
        } else {
            Self::from_values(
                v,
                |i| i,
                |i| i,
                |indices, d| Self::pack_words(indices, |i| (i >> d) & 1 != 0),
                Self::partition,
            )
        }
    }

    fn pack_words<I: Copy>(indices: &[I], bit: impl Fn(I) -> bool) -> Vec<u64> {
        indices
            .chunks(64)
            .map(|chunk| {
                chunk
                    .iter()
                    .enumerate()
                    .fold(0, |word, (i, &index)| word | ((bit(index) as u64) << i))
            })
            .collect()
    }

    fn partition<I: Copy>(indices: &[I], words: &[u64], mut one: usize, next: &mut [I]) {
        let mut zero = 0;
        for (chunk, &word) in indices.chunks(64).zip(words) {
            if word == 0 {
                next[zero..zero + chunk.len()].copy_from_slice(chunk);
                zero += chunk.len();
            } else if word == u64::MAX {
                next[one..one + chunk.len()].copy_from_slice(chunk);
                one += chunk.len();
            } else {
                for (i, &index) in chunk.iter().enumerate() {
                    let bit = (word >> i) & 1 != 0;
                    next[if bit { one } else { zero }] = index;
                    zero += !bit as usize;
                    one += bit as usize;
                }
            }
        }
    }

    fn from_values<I: Copy>(
        v: Vec<T>,
        code: impl Fn(usize) -> I,
        index: impl Fn(I) -> usize,
        pack: impl Fn(&[I], usize) -> Vec<u64>,
        partition: impl Fn(&[I], &[u64], usize, &mut [I]),
    ) -> Self {
        let len = v.len();
        let mut sorted: Vec<_> = v
            .into_iter()
            .enumerate()
            .map(|(i, value)| (value, code(i)))
            .collect();
        sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut values = Vec::with_capacity(len);
        let mut indices = vec![code(0); len];
        for (value, i) in sorted {
            if values.last().is_none_or(|last| last != &value) {
                values.push(value);
            }
            indices[index(i)] = code(values.len() - 1);
        }
        let compress = VecCompress::from_sorted_unique(values);
        let bit_length = usize::BITS as usize - compress.size().leading_zeros() as usize;
        let mut bit_vectors = Vec::with_capacity(bit_length);
        let mut zeros = Vec::with_capacity(bit_length);
        let quad_bits =
            usize::BITS as usize - compress.size().saturating_sub(1).leading_zeros() as usize;
        let mut quad_vectors = Vec::with_capacity(quad_bits.div_ceil(2));
        let mut next = indices.clone();
        for d in (0..bit_length).rev() {
            let words = pack(&indices, d);
            if len <= u32::MAX as usize && d < quad_bits && (d % 2 == 1 || d + 1 == quad_bits) {
                if d % 2 == 1 {
                    let low = pack(&indices, d - 1);
                    quad_vectors.push(WaveletMatrixQuadVector::from_words(&low, Some(&words), len));
                } else {
                    quad_vectors.push(WaveletMatrixQuadVector::from_words(&words, None, len));
                }
            }
            let bits = BitVector::from_words(&words, len);
            let zero_count = bits.rank0(len);
            if d == 0 {
                zeros.push(zero_count);
                bit_vectors.push(bits);
                break;
            }
            partition(&indices, &words, zero_count, &mut next);
            zeros.push(zero_count);
            bit_vectors.push(bits);
            mem::swap(&mut indices, &mut next);
        }
        Self {
            len,
            bit_length,
            zeros,
            bit_vectors,
            quad_vectors,
            compress,
            #[cfg(target_arch = "x86_64")]
            backend: match super::simd_backend() {
                super::SimdBackend::Avx512 if !is_x86_feature_detected!("avx512vpopcntdq") => {
                    super::SimdBackend::Avx2
                }
                backend => backend,
            },
        }
    }

    pub fn new_with_init<F>(v: Vec<T>, mut f: F) -> Self
    where
        F: FnMut(usize, usize, T),
    {
        let this = Self::new(v.clone());
        if !this.quad_vectors.is_empty() {
            let bits = usize::BITS as usize
                - this.compress.size().saturating_sub(1).leading_zeros() as usize;
            for (mut k, value) in v.into_iter().enumerate() {
                if this.bit_length > bits {
                    f(this.bit_length - 1, k, value.clone());
                }
                for (level, vector) in this.quad_vectors.iter().enumerate() {
                    let d = (this.quad_vectors.len() - level - 1) * 2;
                    let (digit, rank) = vector.access_rank(k);
                    if d + 1 < bits {
                        let block = &vector.blocks[k / 64];
                        let high = block.rank[2] as usize
                            + block.rank[3] as usize
                            + (block.hi & !(u64::MAX << (k % 64))).count_ones() as usize;
                        let middle = if digit & 2 == 0 {
                            k - high
                        } else {
                            this.zeros[this.level(d + 1)] + high
                        };
                        f(d + 1, middle, value.clone());
                    }
                    k = vector.starts[digit] + rank;
                    f(d, k, value.clone());
                }
            }
            return this;
        }
        for (mut k, value) in v.into_iter().enumerate() {
            for d in (0..this.bit_length).rev() {
                let level = this.level(d);
                let (bit, rank1) = this.bit_vectors[level].access_rank1(k);
                k = if bit {
                    this.zeros[level] + rank1
                } else {
                    k - rank1
                };
                f(d, k, value.clone());
            }
        }
        this
    }

    fn level(&self, d: usize) -> usize {
        self.bit_length - 1 - d
    }

    fn rank1(&self, level: usize, k: usize) -> usize {
        self.bit_vectors[level].rank1(k)
    }

    fn reorder<U>(&self, level: usize, current: Vec<U>) -> Vec<U> {
        assert_eq!(current.len(), self.len);
        let mut next = Vec::with_capacity(self.len);
        next.resize_with(self.len, MaybeUninit::uninit);
        let mut zero = 0;
        let mut one = self.zeros[level];
        let mut current = current.into_iter();
        for block in self.bit_vectors[level].blocks() {
            let count = current.len().min(64);
            if block.bits == 0 || block.bits == u64::MAX {
                let offset = if block.bits == 0 { &mut zero } else { &mut one };
                for (slot, value) in next[*offset..*offset + count]
                    .iter_mut()
                    .zip(current.by_ref().take(count))
                {
                    slot.write(value);
                }
                *offset += count;
            } else {
                for (i, value) in current.by_ref().take(count).enumerate() {
                    let bit = (block.bits >> i) & 1 != 0;
                    next[if bit { one } else { zero }].write(value);
                    zero += !bit as usize;
                    one += bit as usize;
                }
            }
        }
        // SAFETY: the partition counts fill every slot once, and `MaybeUninit<U>` has `U`'s layout.
        unsafe {
            let mut next = mem::ManuallyDrop::new(next);
            Vec::from_raw_parts(next.as_mut_ptr().cast(), next.len(), next.capacity())
        }
    }

    fn range_by_index(&self, idx: usize, mut range: Range<usize>) -> Range<usize> {
        if !self.quad_vectors.is_empty() {
            for (level, vector) in self.quad_vectors.iter().enumerate() {
                if range.is_empty() {
                    break;
                }
                let digit = (idx >> ((self.quad_vectors.len() - level - 1) * 2)) & 3;
                range = vector.starts[digit] + vector.rank(digit, range.start)
                    ..vector.starts[digit] + vector.rank(digit, range.end);
            }
            return range;
        }
        for d in (0..self.bit_length - self.compress.size().is_power_of_two() as usize).rev() {
            if range.is_empty() {
                break;
            }
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            if ((idx >> d) & 1) != 0 {
                range.start = self.zeros[level] + start1;
                range.end = self.zeros[level] + end1;
            } else {
                range.start -= start1;
                range.end -= end1;
            }
        }
        range
    }

    fn batch<const OP: u8>(&self, states: &mut [[usize; 4]; 16], count: usize) {
        #[cfg(target_arch = "x86_64")]
        if !cfg!(target_feature = "popcnt") && is_x86_feature_detected!("popcnt") {
            // SAFETY: POPCNT is checked above.
            unsafe {
                self.batch_popcnt::<OP>(states, count);
            }
            return;
        }
        self.batch_inner::<OP>(states, count);
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "popcnt")]
    unsafe fn batch_popcnt<const OP: u8>(&self, states: &mut [[usize; 4]; 16], count: usize) {
        self.batch_inner::<OP>(states, count);
    }

    #[inline(always)]
    fn batch_inner<const OP: u8>(&self, states: &mut [[usize; 4]; 16], count: usize) {
        if OP == RANK_LESSTHAN && self.compress.size() <= 1 {
            for state in &mut states[..count] {
                state[3] = if state[2] == 0 {
                    0
                } else {
                    state[1] - state[0]
                };
            }
            return;
        }
        if !self.quad_vectors.is_empty() && matches!(OP, ACCESS | RANK | QUANTILE) {
            #[cfg(target_arch = "x86_64")]
            if OP == QUANTILE && count >= 8 && self.backend == super::SimdBackend::Avx512 {
                // SAFETY: checked batch ranges stay within the quad blocks after each
                // stable partition. Construction restricts quad counters to u32, and
                // the cached backend includes AVX512F and VPOPCNTDQ.
                unsafe {
                    simd::quad_avx512(&self.quad_vectors, &mut states[..count.next_multiple_of(8)]);
                }
                return;
            }

            for (level, vector) in self.quad_vectors.iter().enumerate() {
                let d = (self.quad_vectors.len() - level - 1) * 2;
                #[cfg(target_arch = "x86_64")]
                let next = self
                    .quad_vectors
                    .get(level + 1)
                    .filter(|_| self.len >= 1048576 || (OP == ACCESS && self.len >= 262144));
                for state in &mut states[..count] {
                    if OP == ACCESS {
                        let (digit, rank) = vector.access_rank(state[0]);
                        state[0] = vector.starts[digit] + rank;
                        state[3] = state[3] * 4 + digit;
                    } else if OP == RANK {
                        let digit = (state[2] >> d) & 3;
                        state[0] = vector.starts[digit] + vector.rank(digit, state[0]);
                        state[1] = vector.starts[digit] + vector.rank(digit, state[1]);
                    } else {
                        let start = vector.ranks(state[0]);
                        let end = vector.ranks(state[1]);
                        let prefix = [
                            0,
                            end[0] - start[0],
                            end[0] + end[1] - start[0] - start[1],
                            state[1] - state[0] - (end[3] - start[3]),
                        ];
                        let digit = (state[2] >= prefix[1]) as usize
                            + (state[2] >= prefix[2]) as usize
                            + (state[2] >= prefix[3]) as usize;
                        state[2] -= prefix[digit];
                        state[0] = vector.starts[digit] + start[digit];
                        state[1] = vector.starts[digit] + end[digit];
                        state[3] = state[3] * 4 + digit;
                    }
                    #[cfg(target_arch = "x86_64")]
                    if let Some(next) = next {
                        // SAFETY: stable partitions keep both endpoints within the next vector.
                        unsafe {
                            std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                                next.blocks.as_ptr().add(state[0] / 64).cast(),
                            );
                            if OP != ACCESS {
                                std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                                    next.blocks.as_ptr().add(state[1] / 64).cast(),
                                );
                            }
                        }
                    }
                }
            }
            return;
        }
        let first = (matches!(OP, ACCESS | RANK | QUANTILE)
            && self.compress.size().is_power_of_two()) as usize;
        #[cfg(target_arch = "x86_64")]
        if OP == RANK_LESSTHAN && count >= 8 && self.len <= u32::MAX as usize {
            // SAFETY: the caller checked each initial position. Rank transitions remain in
            // 0..=len, including the sentinel block. Padding uses position zero. The
            // backend was detected at construction, and x86-64 blocks contain two u64s.
            unsafe {
                match self.backend {
                    super::SimdBackend::Avx512 => {
                        simd::rank_lessthan_avx512(
                            &self.bit_vectors[first..],
                            &self.zeros[first..],
                            &mut states[..count.next_multiple_of(8)],
                        );
                        return;
                    }
                    super::SimdBackend::Avx2 if self.len < 1048576 => {
                        simd::rank_lessthan_avx2(
                            &self.bit_vectors[first..],
                            &self.zeros[first..],
                            &mut states[..count.next_multiple_of(4)],
                        );
                        return;
                    }
                    _ => {}
                }
            }
        }
        for d in (0..self.bit_length - first).rev() {
            let level = self.level(d);
            for state in &mut states[..count] {
                let (bit, start1) = self.bit_vectors[level].access_rank1(state[0]);
                let start0 = state[0] - start1;
                let end1 = if OP == ACCESS {
                    0
                } else {
                    self.rank1(level, state[1])
                };
                let end0 = if OP == ACCESS { 0 } else { state[1] - end1 };
                let count0 = if OP == ACCESS { 0 } else { end0 - start0 };
                let bit = match OP {
                    ACCESS => bit,
                    RANK | RANK_LESSTHAN => (state[2] >> d) & 1 != 0,
                    _ => state[2] >= count0,
                };
                state[0] = if bit {
                    self.zeros[level] + start1
                } else {
                    start0
                };
                if OP != ACCESS {
                    state[1] = if bit { self.zeros[level] + end1 } else { end0 };
                }
                if OP == ACCESS || OP == QUANTILE {
                    state[3] |= (bit as usize) << d;
                }
                if OP == QUANTILE {
                    state[2] -= if bit { count0 } else { 0 };
                }
                if OP == RANK_LESSTHAN {
                    state[3] += if bit { count0 } else { 0 };
                }
            }
        }
    }

    /// get k-th value
    pub fn access(&self, mut k: usize) -> T {
        if !self.quad_vectors.is_empty() {
            let mut index = 0;
            for vector in &self.quad_vectors {
                let (digit, rank) = vector.access_rank(k);
                index = index * 4 + digit;
                k = vector.starts[digit] + rank;
            }
            return self.compress.values()[index].clone();
        }
        let mut idx = 0;
        for d in (0..self.bit_length - self.compress.size().is_power_of_two() as usize).rev() {
            let level = self.level(d);
            let (bit, rank1) = self.bit_vectors[level].access_rank1(k);
            idx |= (bit as usize) << d;
            k = if bit {
                self.zeros[level] + rank1
            } else {
                k - rank1
            };
        }
        self.compress.values()[idx].clone()
    }

    /// Returns the values at `indices` in input order.
    pub fn access_batch(&self, indices: impl IntoIterator<Item = usize>) -> Vec<T> {
        let indices: Vec<_> = indices.into_iter().collect();
        let mut result = Vec::with_capacity(indices.len());
        for indices in indices.chunks(16) {
            let mut states = [[0; 4]; 16];
            for (state, &index) in states.iter_mut().zip(indices) {
                assert!(index < self.len);
                state[0] = index;
            }
            self.batch::<ACCESS>(&mut states, indices.len());
            result.extend(
                states[..indices.len()]
                    .iter()
                    .map(|state| self.compress.values()[state[3]].clone()),
            );
        }
        result
    }

    /// the number of val in range
    pub fn rank(&self, val: T, range: Range<usize>) -> usize {
        match self.compress.index_exact(&val) {
            Some(idx) => self.range_by_index(idx, range).len(),
            None => 0,
        }
    }

    /// Returns the number of exact matches for each `(value, range)` query.
    pub fn rank_batch(&self, queries: impl IntoIterator<Item = (T, Range<usize>)>) -> Vec<usize> {
        let queries: Vec<_> = queries.into_iter().collect();
        let mut result = Vec::with_capacity(queries.len());
        for queries in queries.chunks(16) {
            let mut states = [[0; 4]; 16];
            for (state, (value, range)) in states.iter_mut().zip(queries) {
                assert!(range.start <= range.end && range.end <= self.len);
                if let Some(index) = self.compress.index_exact(value) {
                    *state = [range.start, range.end, index, 0];
                }
            }
            self.batch::<RANK>(&mut states, queries.len());
            result.extend(
                states[..queries.len()]
                    .iter()
                    .map(|state| state[1] - state[0]),
            );
        }
        result
    }

    /// index of k-th val
    pub fn select(&self, val: T, k: usize) -> Option<usize> {
        let idx = self.compress.index_exact(&val)?;
        let range = self.range_by_index(idx, 0..self.len);
        if range.len() <= k {
            return None;
        }
        let mut i = range.start + k;
        if !self.quad_vectors.is_empty() {
            for (level, vector) in self.quad_vectors.iter().enumerate().rev() {
                let digit = (idx >> ((self.quad_vectors.len() - level - 1) * 2)) & 3;
                i = vector.select(digit, i - vector.starts[digit]);
            }
            return Some(i);
        }
        for level in (self.compress.size().is_power_of_two() as usize..self.bit_length).rev() {
            if i >= self.zeros[level] {
                i = self.bit_vectors[level]
                    .select1(i - self.zeros[level])
                    .unwrap();
            } else {
                i = self.bit_vectors[level].select0(i).unwrap();
            }
        }
        Some(i)
    }

    /// get k-th smallest value in range
    pub fn quantile(&self, mut range: Range<usize>, mut k: usize) -> T {
        if !self.quad_vectors.is_empty() {
            return self.quad_quantile(range, k, 0, 0);
        }
        let mut idx = 0;
        for d in (0..self.bit_length - self.compress.size().is_power_of_two() as usize).rev() {
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            let z = end0 - start0;
            let bit = z <= k;
            k -= if bit { z } else { 0 };
            idx |= (bit as usize) << d;
            range.start = if bit {
                self.zeros[level] + start1
            } else {
                start0
            };
            range.end = if bit { self.zeros[level] + end1 } else { end0 };
        }
        self.compress.values()[idx].clone()
    }

    #[inline(always)]
    fn quad_quantile(
        &self,
        mut range: Range<usize>,
        mut k: usize,
        level: usize,
        mut index: usize,
    ) -> T {
        for vector in &self.quad_vectors[level..] {
            let start = vector.ranks(range.start);
            let end = vector.ranks(range.end);
            let mut digit = 0;
            while digit < 3 && k >= end[digit] - start[digit] {
                k -= end[digit] - start[digit];
                digit += 1;
            }
            index = index * 4 + digit;
            range = vector.starts[digit] + start[digit]..vector.starts[digit] + end[digit];
        }
        self.compress.values()[index].clone()
    }

    pub fn quantile_batch(
        &self,
        queries: impl IntoIterator<Item = (Range<usize>, usize)>,
    ) -> Vec<T> {
        let queries: Vec<_> = queries.into_iter().collect();
        let mut result = Vec::with_capacity(queries.len());
        for queries in queries.chunks(16) {
            let mut states = [[0; 4]; 16];
            for (state, (range, k)) in states.iter_mut().zip(queries) {
                assert!(range.start <= range.end && range.end <= self.len && *k < range.len());
                *state = [range.start, range.end, *k, 0];
            }
            if let [(range, k)] = queries {
                result.push(self.quantile(range.clone(), *k));
                continue;
            }
            self.batch::<QUANTILE>(&mut states, queries.len());
            result.extend(
                states[..queries.len()]
                    .iter()
                    .map(|state| self.compress.values()[state[3]].clone()),
            );
        }
        result
    }

    /// Returns the requested order statistics of one range. `ranks` must be sorted
    /// in nondecreasing order and every rank must be less than the range length.
    pub fn quantiles_sorted(&self, range: Range<usize>, ranks: &[usize]) -> Vec<T> {
        assert!(range.start <= range.end && range.end <= self.len);
        assert!(ranks.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(ranks.last().is_none_or(|&k| k < range.len()));
        if ranks.is_empty() {
            return Vec::new();
        }
        if ranks[0] == ranks[ranks.len() - 1] {
            return vec![self.quantile(range, ranks[0]); ranks.len()];
        }
        let use_batch = ranks.len() < 1024;
        #[cfg(target_arch = "x86_64")]
        let use_batch = use_batch || self.backend == super::SimdBackend::Avx512;
        // Sparse requests amortize less of the shared traversal's branching work.
        if use_batch && ranks.len() < self.compress.size().div_ceil(16) {
            return self.quantile_batch(ranks.iter().map(|&k| (range.clone(), k)));
        }
        let mut result = Vec::with_capacity(ranks.len());
        if !self.quad_vectors.is_empty() {
            let mut stack = vec![(range, 0, 0, 0, ranks)];
            while let Some((range, base, index, level, ranks)) = stack.pop() {
                if level == self.quad_vectors.len() {
                    result.resize(
                        result.len() + ranks.len(),
                        self.compress.values()[index].clone(),
                    );
                    continue;
                }
                if ranks.len() == 1 {
                    result.push(self.quad_quantile(range, ranks[0] - base, level, index));
                    continue;
                }
                let vector = &self.quad_vectors[level];
                let start = vector.ranks(range.start);
                let end = vector.ranks(range.end);
                let mut split = base + range.len();
                let mut requested = ranks.len();
                for digit in (0..4).rev() {
                    split -= end[digit] - start[digit];
                    let boundary = ranks[..requested].partition_point(|&k| k < split);
                    if boundary < requested {
                        stack.push((
                            vector.starts[digit] + start[digit]..vector.starts[digit] + end[digit],
                            split,
                            index * 4 + digit,
                            level + 1,
                            &ranks[boundary..requested],
                        ));
                    }
                    requested = boundary;
                }
            }
            return result;
        }
        let mut stack = vec![(range, 0, 0, self.bit_length, ranks)];
        while let Some((range, base, index, bits, ranks)) = stack.pop() {
            if bits == 0 {
                result.resize(
                    result.len() + ranks.len(),
                    self.compress.values()[index].clone(),
                );
                continue;
            }
            let d = bits - 1;
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            let split = base + end0 - start0;
            let boundary = ranks.partition_point(|&k| k < split);
            if boundary < ranks.len() {
                stack.push((
                    self.zeros[level] + start1..self.zeros[level] + end1,
                    split,
                    index | (1 << d),
                    d,
                    &ranks[boundary..],
                ));
            }
            if boundary != 0 {
                stack.push((start0..end0, base, index, d, &ranks[..boundary]));
            }
        }
        result
    }

    /// get k-th smallest value out of range
    pub fn quantile_outer(&self, mut range: Range<usize>, mut k: usize) -> T {
        if !self.quad_vectors.is_empty() {
            let mut outer = 0..self.len;
            let mut index = 0;
            for vector in &self.quad_vectors {
                let start = vector.ranks(range.start);
                let end = vector.ranks(range.end);
                let outer_start = vector.ranks(outer.start);
                let outer_end = vector.ranks(outer.end);
                let mut digit = 0;
                while digit < 3 {
                    let count = outer_end[digit] - outer_start[digit] - (end[digit] - start[digit]);
                    if k < count {
                        break;
                    }
                    k -= count;
                    digit += 1;
                }
                index = index * 4 + digit;
                range = vector.starts[digit] + start[digit]..vector.starts[digit] + end[digit];
                outer = vector.starts[digit] + outer_start[digit]
                    ..vector.starts[digit] + outer_end[digit];
            }
            return self.compress.values()[index].clone();
        }
        let mut idx = 0;
        let mut orange = 0..self.len;
        for d in (0..self.bit_length - self.compress.size().is_power_of_two() as usize).rev() {
            let level = self.level(d);
            let range_start1 = self.rank1(level, range.start);
            let range_end1 = self.rank1(level, range.end);
            let outer_start1 = self.rank1(level, orange.start);
            let outer_end1 = self.rank1(level, orange.end);
            let range_start0 = range.start - range_start1;
            let range_end0 = range.end - range_end1;
            let outer_start0 = orange.start - outer_start1;
            let outer_end0 = orange.end - outer_end1;
            let z = (outer_end0 - outer_start0) - (range_end0 - range_start0);
            if z <= k {
                k -= z;
                idx |= 1 << d;
                range.start = self.zeros[level] + range_start1;
                range.end = self.zeros[level] + range_end1;
                orange.start = self.zeros[level] + outer_start1;
                orange.end = self.zeros[level] + outer_end1;
            } else {
                range.start = range_start0;
                range.end = range_end0;
                orange.start = outer_start0;
                orange.end = outer_end0;
            }
        }
        self.compress.values()[idx].clone()
    }

    /// the number of value less than val in range
    pub fn rank_lessthan(&self, val: T, range: Range<usize>) -> usize {
        let index = self.compress.index_lower_bound(&val);
        if index == self.compress.size() {
            return range.len();
        }
        if self.len >= 262144 && !self.quad_vectors.is_empty() {
            self.quad_rank_lessthan_index(index, range, 0)
        } else {
            let bits = usize::BITS as usize
                - self.compress.size().saturating_sub(1).leading_zeros() as usize;
            self.rank_lessthan_index(index, range, bits)
        }
    }

    /// Returns the number of values below each query's threshold.
    pub fn rank_lessthan_batch(
        &self,
        queries: impl IntoIterator<Item = (T, Range<usize>)>,
    ) -> Vec<usize> {
        let queries: Vec<_> = queries.into_iter().collect();
        let mut result = Vec::with_capacity(queries.len());
        for queries in queries.chunks(16) {
            let mut states = [[0; 4]; 16];
            for (state, (value, range)) in states.iter_mut().zip(queries) {
                assert!(range.start <= range.end && range.end <= self.len);
                *state = [
                    range.start,
                    range.end,
                    self.compress.index_lower_bound(value),
                    0,
                ];
            }
            self.batch::<RANK_LESSTHAN>(&mut states, queries.len());
            result.extend(states[..queries.len()].iter().map(|state| state[3]));
        }
        result
    }

    fn quad_rank_lessthan_index(
        &self,
        index: usize,
        mut range: Range<usize>,
        start_level: usize,
    ) -> usize {
        let mut result = 0;
        for (level, vector) in self.quad_vectors.iter().enumerate().skip(start_level) {
            let d = (self.quad_vectors.len() - level - 1) * 2;
            if d + 2 <= index.trailing_zeros() as usize {
                break;
            }
            let digit = (index >> d) & 3;
            let ranks = |position: usize| {
                let block = &vector.blocks[position / 64];
                let mask = !(u64::MAX << (position % 64));
                let low = if digit & 1 == 0 { !block.lo } else { block.lo };
                let high = if digit & 2 == 0 { !block.hi } else { block.hi };
                let exact = block.rank[digit] as usize + (low & high & mask).count_ones() as usize;
                let less = match digit {
                    0 => 0,
                    1 => {
                        block.rank[0] as usize
                            + (!(block.lo | block.hi) & mask).count_ones() as usize
                    }
                    2 => {
                        block.rank[0] as usize
                            + block.rank[1] as usize
                            + (!block.hi & mask).count_ones() as usize
                    }
                    _ => position - exact,
                };
                (exact, less)
            };
            let (start, start_less) = ranks(range.start);
            let (end, end_less) = ranks(range.end);
            result += end_less - start_less;
            range = vector.starts[digit] + start..vector.starts[digit] + end;
        }
        result
    }

    fn rank_lessthan_index(&self, idx: usize, mut range: Range<usize>, bits: usize) -> usize {
        let mut res = 0;
        for d in (idx.trailing_zeros() as usize..bits).rev() {
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let bit = (idx >> d) & 1 != 0;
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            res += if bit { end0 - start0 } else { 0 };
            range.start = if bit {
                self.zeros[level] + start1
            } else {
                start0
            };
            range.end = if bit { self.zeros[level] + end1 } else { end0 };
        }
        res
    }

    /// the number of valrange in range
    pub fn rank_range(&self, valrange: Range<T>, mut range: Range<usize>) -> usize {
        let lower = self.compress.index_lower_bound(&valrange.start);
        let upper = self.compress.index_lower_bound(&valrange.end);
        if lower >= upper {
            return 0;
        }
        if !self.quad_vectors.is_empty() {
            if upper == self.compress.size() {
                return range.len() - self.quad_rank_lessthan_index(lower, range, 0);
            }
            for (level, vector) in self.quad_vectors.iter().enumerate() {
                let d = (self.quad_vectors.len() - level - 1) * 2;
                let low = (lower >> d) & 3;
                let high = (upper >> d) & 3;
                if low == high {
                    range = vector.starts[low] + vector.rank(low, range.start)
                        ..vector.starts[low] + vector.rank(low, range.end);
                    continue;
                }
                let start = vector.ranks(range.start);
                let end = vector.ranks(range.end);
                let count: usize = (low..high).map(|digit| end[digit] - start[digit]).sum();
                return count
                    - self.quad_rank_lessthan_index(
                        lower,
                        vector.starts[low] + start[low]..vector.starts[low] + end[low],
                        level + 1,
                    )
                    + self.quad_rank_lessthan_index(
                        upper,
                        vector.starts[high] + start[high]..vector.starts[high] + end[high],
                        level + 1,
                    );
            }
            return 0;
        }
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let zero_range = range.start - start1..range.end - end1;
            let one_range = self.zeros[level] + start1..self.zeros[level] + end1;
            if ((lower ^ upper) >> d) & 1 != 0 {
                return zero_range.len() - self.rank_lessthan_index(lower, zero_range, d)
                    + self.rank_lessthan_index(upper, one_range, d);
            }
            range = if (lower >> d) & 1 != 0 {
                one_range
            } else {
                zero_range
            };
        }
        0
    }

    /// Returns each query's count in a half-open value range.
    pub fn rank_range_batch(
        &self,
        queries: impl IntoIterator<Item = (Range<T>, Range<usize>)>,
    ) -> Vec<usize> {
        let queries: Vec<_> = queries.into_iter().collect();
        let mut result = Vec::with_capacity(queries.len());
        for queries in queries.chunks(8) {
            let mut states = [[0; 4]; 16];
            for (i, (values, range)) in queries.iter().enumerate() {
                assert!(range.start <= range.end && range.end <= self.len);
                let lower = self.compress.index_lower_bound(&values.start);
                let upper = self.compress.index_lower_bound(&values.end);
                if lower < upper {
                    states[i * 2] = [range.start, range.end, lower, 0];
                    states[i * 2 + 1] = [range.start, range.end, upper, 0];
                }
            }
            self.batch::<RANK_LESSTHAN>(&mut states, queries.len() * 2);
            result.extend(
                states[..queries.len() * 2]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| pair[1][3] - pair[0][3]),
            );
        }
        result
    }

    pub fn query_less_than<F>(&self, val: T, mut range: Range<usize>, mut f: F)
    where
        F: FnMut(usize, Range<usize>),
    {
        let idx = self.compress.index_lower_bound(&val);
        if !self.quad_vectors.is_empty() {
            if idx == self.compress.size() && idx.is_power_of_two() {
                f(self.bit_length - 1, range);
                return;
            }
            for (level, vector) in self.quad_vectors.iter().enumerate().take(
                self.quad_vectors
                    .len()
                    .saturating_sub(idx.trailing_zeros() as usize / 2),
            ) {
                let d = (self.quad_vectors.len() - level - 1) * 2;
                let digit = (idx >> d) & 3;
                let start = vector.ranks(range.start);
                let end = vector.ranks(range.end);
                if digit & 2 != 0 {
                    f(d + 1, start[0] + start[1]..end[0] + end[1]);
                }
                if digit & 1 != 0 {
                    let zero = digit & 2;
                    f(
                        d,
                        vector.starts[zero] + start[zero]..vector.starts[zero] + end[zero],
                    );
                }
                range = vector.starts[digit] + start[digit]..vector.starts[digit] + end[digit];
            }
            return;
        }
        for d in (idx.trailing_zeros() as usize..self.bit_length).rev() {
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((idx >> d) & 1) != 0 {
                f(d, start0..end0);
                range.start = self.zeros[level] + start1;
                range.end = self.zeros[level] + end1;
            } else {
                range.start = start0;
                range.end = end0;
            }
        }
    }

    pub fn build_fold<M>(&self, weights: &[M::T]) -> WaveletMatrixFold<'_, T, M>
    where
        M: AbelianGroup,
    {
        assert_eq!(weights.len(), self.len);
        let mut offsets = Vec::with_capacity(self.bit_length);
        let mut prefix = Vec::with_capacity(self.zeros.iter().map(|&zero| zero + 1).sum());
        let mut current: Vec<M::T> = weights.to_vec();
        for level in 0..self.bit_length {
            current = self.reorder(level, current);
            offsets.push(prefix.len());
            let mut acc = M::unit();
            prefix.push(acc.clone());
            for w in &current[..self.zeros[level]] {
                acc = M::operate(&acc, w);
                prefix.push(acc.clone());
            }
        }
        WaveletMatrixFold {
            wavelet_matrix: self,
            prefix,
            offsets,
        }
    }

    pub fn build_point_add<M>(&self, weights: &[M::T]) -> WaveletMatrixPointAdd<'_, T, M>
    where
        M: AbelianGroup,
    {
        assert_eq!(weights.len(), self.len);
        let mut current = weights.to_vec();
        let mut bits = Vec::with_capacity(self.bit_length);
        for level in 0..self.bit_length {
            current = self.reorder(level, current);
            bits.push(BinaryIndexedTree::from_slice(&current[..self.zeros[level]]));
        }
        WaveletMatrixPointAdd {
            wavelet_matrix: self,
            bits,
        }
    }
}

pub struct WaveletMatrixPointAdd<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    wavelet_matrix: &'a WaveletMatrix<T>,
    bits: Vec<BinaryIndexedTree<M>>,
}

impl<'a, T, M> WaveletMatrixPointAdd<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    pub fn update(&mut self, mut index: usize, value: M::T) {
        debug_assert!(index < self.wavelet_matrix.len);
        for d in (0..self.wavelet_matrix.bit_length).rev() {
            let level = self.wavelet_matrix.level(d);
            let (bit, rank1) = self.wavelet_matrix.bit_vectors[level].access_rank1(index);
            if bit {
                index = self.wavelet_matrix.zeros[level] + rank1;
            } else {
                index -= rank1;
                self.bits[level].update(index, value.clone());
            }
        }
    }

    pub fn fold_lessthan(&self, value: T, range: Range<usize>) -> M::T {
        let mut result = M::unit();
        self.wavelet_matrix
            .query_less_than(value, range, |d, range| {
                M::operate_assign(
                    &mut result,
                    &self.bits[self.wavelet_matrix.level(d)].fold_abelian(range.start, range.end),
                );
            });
        result
    }

    pub fn fold_range(&self, values: Range<T>, range: Range<usize>) -> M::T {
        let lower = self
            .wavelet_matrix
            .compress
            .index_lower_bound(&values.start);
        let upper = self.wavelet_matrix.compress.index_lower_bound(&values.end);
        if lower >= upper {
            return M::unit();
        }
        let mut range = range;
        for d in (0..self.wavelet_matrix.bit_length).rev() {
            let level = self.wavelet_matrix.level(d);
            let start1 = self.wavelet_matrix.bit_vectors[level].rank1(range.start);
            let end1 = self.wavelet_matrix.bit_vectors[level].rank1(range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((lower >> d) & 1) == ((upper >> d) & 1) {
                if ((lower >> d) & 1) == 0 {
                    range = start0..end0;
                } else {
                    range = self.wavelet_matrix.zeros[level] + start1
                        ..self.wavelet_matrix.zeros[level] + end1;
                }
                continue;
            }
            let zero_range = start0..end0;
            let one_range =
                self.wavelet_matrix.zeros[level] + start1..self.wavelet_matrix.zeros[level] + end1;
            let lower_sum = self.fold_lessthan_index(lower, zero_range.clone(), d);
            let upper_sum = self.fold_lessthan_index(upper, one_range, d);
            let zero_sum = self.bits[level].fold_abelian(zero_range.start, zero_range.end);
            let mut result = M::rinv_operate(&zero_sum, &lower_sum);
            M::operate_assign(&mut result, &upper_sum);
            return result;
        }
        M::unit()
    }

    fn fold_lessthan_index(&self, idx: usize, mut range: Range<usize>, bits: usize) -> M::T {
        let mut result = M::unit();
        for d in (idx.trailing_zeros() as usize..bits).rev() {
            let level = self.wavelet_matrix.level(d);
            let start1 = self.wavelet_matrix.bit_vectors[level].rank1(range.start);
            let end1 = self.wavelet_matrix.bit_vectors[level].rank1(range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((idx >> d) & 1) != 0 {
                M::operate_assign(&mut result, &self.bits[level].fold_abelian(start0, end0));
                range.start = self.wavelet_matrix.zeros[level] + start1;
                range.end = self.wavelet_matrix.zeros[level] + end1;
            } else {
                range.start = start0;
                range.end = end0;
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct WaveletMatrixFold<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    wavelet_matrix: &'a WaveletMatrix<T>,
    prefix: Vec<M::T>,
    offsets: Vec<usize>,
}

impl<'a, T, M> WaveletMatrixFold<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    pub fn fold_lessthan(&self, val: T, range: Range<usize>) -> M::T {
        self.fold_lessthan_with_count(val, range).1
    }

    pub fn fold_lessthan_with_count(&self, val: T, range: Range<usize>) -> (usize, M::T) {
        debug_assert!(range.end <= self.wavelet_matrix.len);
        let [result] = self.fold_lessthan_indices_with_count(
            [self.wavelet_matrix.compress.index_lower_bound(&val)],
            [range],
            self.wavelet_matrix.bit_length,
        );
        result
    }

    pub fn fold_range(&self, valrange: Range<T>, range: Range<usize>) -> M::T {
        self.fold_range_with_count(valrange, range).1
    }

    pub fn fold_range_with_count(
        &self,
        valrange: Range<T>,
        mut range: Range<usize>,
    ) -> (usize, M::T) {
        debug_assert!(range.end <= self.wavelet_matrix.len);
        let lower = self
            .wavelet_matrix
            .compress
            .index_lower_bound(&valrange.start);
        let upper = self
            .wavelet_matrix
            .compress
            .index_lower_bound(&valrange.end);
        if lower >= upper {
            return (0, M::unit());
        }
        for d in (0..self.wavelet_matrix.bit_length).rev() {
            let level = self.wavelet_matrix.level(d);
            let start1 = self.wavelet_matrix.bit_vectors[level].rank1(range.start);
            let end1 = self.wavelet_matrix.bit_vectors[level].rank1(range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((lower >> d) & 1) == ((upper >> d) & 1) {
                if ((lower >> d) & 1) == 0 {
                    range = start0..end0;
                } else {
                    range = self.wavelet_matrix.zeros[level] + start1
                        ..self.wavelet_matrix.zeros[level] + end1;
                }
                continue;
            }
            let zero_range = start0..end0;
            let one_range =
                self.wavelet_matrix.zeros[level] + start1..self.wavelet_matrix.zeros[level] + end1;
            let [(lower_count, lower_sum), (upper_count, upper_sum)] = self
                .fold_lessthan_indices_with_count(
                    [lower, upper],
                    [zero_range.clone(), one_range],
                    d,
                );
            let zero_sum = self.range_sum(level, zero_range.clone());
            return (
                zero_range.len() - lower_count + upper_count,
                M::operate(&M::rinv_operate(&zero_sum, &lower_sum), &upper_sum),
            );
        }
        (0, M::unit())
    }

    #[inline]
    fn range_sum(&self, level: usize, range: Range<usize>) -> M::T {
        let offset = self.offsets[level];
        M::rinv_operate(
            &self.prefix[offset + range.end],
            &self.prefix[offset + range.start],
        )
    }

    fn fold_lessthan_indices_with_count<const N: usize>(
        &self,
        indices: [usize; N],
        mut ranges: [Range<usize>; N],
        bits: usize,
    ) -> [(usize, M::T); N] {
        let mut results = std::array::from_fn(|_| (0, M::unit()));
        let last = indices
            .iter()
            .map(|index| index.trailing_zeros() as usize)
            .min()
            .unwrap_or(bits);
        for d in (last..bits).rev() {
            let level = self.wavelet_matrix.level(d);
            for ((&index, range), (count, sum)) in indices.iter().zip(&mut ranges).zip(&mut results)
            {
                let start1 = self.wavelet_matrix.bit_vectors[level].rank1(range.start);
                let end1 = self.wavelet_matrix.bit_vectors[level].rank1(range.end);
                let start0 = range.start - start1;
                let end0 = range.end - end1;
                if ((index >> d) & 1) != 0 {
                    *count += end0 - start0;
                    M::operate_assign(sum, &self.range_sum(level, start0..end0));
                    range.start = self.wavelet_matrix.zeros[level] + start1;
                    range.end = self.wavelet_matrix.zeros[level] + end1;
                } else {
                    range.start = start0;
                    range.end = end0;
                }
            }
        }
        results
    }

    /// Folds the weights below each query's threshold, traversing the queries together.
    pub fn fold_lessthan_batch(
        &self,
        queries: impl IntoIterator<Item = (T, Range<usize>)>,
    ) -> Vec<M::T> {
        let queries: Vec<_> = queries.into_iter().collect();
        let mut result = Vec::with_capacity(queries.len());
        for queries in queries.chunks(16) {
            let mut indices = [0; 16];
            let mut ranges = std::array::from_fn(|_| 0..0);
            for (i, (value, range)) in queries.iter().enumerate() {
                assert!(range.start <= range.end && range.end <= self.wavelet_matrix.len);
                indices[i] = self.wavelet_matrix.compress.index_lower_bound(value);
                ranges[i] = range.clone();
            }
            result.extend(
                self.fold_lessthan_indices_with_count(
                    indices,
                    ranges,
                    self.wavelet_matrix.bit_length,
                )
                .into_iter()
                .take(queries.len())
                .map(|(_, sum)| sum),
            );
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_wavelet_matrix() {
        let mut rng = Xorshift::default();
        let mut lengths: Vec<usize> = vec![
            0, 1, 2, 3, 7, 8, 15, 16, 17, 63, 64, 65, 127, 128, 129, 255, 256, 257, 1000, 262144,
            1048577,
        ];
        lengths.extend((0..32).map(|_| rng.random(0usize..1024)));
        for n in lengths {
            let sigma: i64 = if n > 1024 { 4095 } else { rng.random(1..65) };
            let values: Vec<_> = (0..n).map(|_| rng.random(0..sigma) * 2).collect();
            let ranks: Vec<_> = (0..33)
                .map(|i| {
                    let left: usize = rng.random(0..=n);
                    let right = rng.random(left..=(left + 257).min(n));
                    let range = if i == 0 { 0..n } else { left..right };
                    (rng.random(-1..=sigma * 2), range)
                })
                .collect();
            let counts: Vec<_> = ranks
                .iter()
                .map(|(v, r)| values[r.clone()].iter().filter(|&x| x == v).count())
                .collect();
            let less: Vec<_> = ranks
                .iter()
                .map(|(v, r)| values[r.clone()].iter().filter(|&x| x < v).count())
                .collect();
            let positions: Vec<Vec<_>> = ranks
                .iter()
                .map(|(v, _)| (0..n).filter(|&i| values[i] == *v).collect())
                .collect();
            let ranges: Vec<_> = ranks
                .iter()
                .map(|(v, r)| (*v..*v + rng.random(0..=sigma), r.clone()))
                .collect();
            let range_counts: Vec<_> = ranges
                .iter()
                .map(|(v, r)| values[r.clone()].iter().filter(|x| v.contains(x)).count())
                .collect();
            let indices: Vec<_> = (0..33)
                .filter(|_| n != 0)
                .map(|_| rng.random(0..n))
                .collect();
            let queries: Vec<_> = indices
                .iter()
                .enumerate()
                .map(|(i, &left)| {
                    let right = rng.random(left + 1..=(left + 257).min(n));
                    let range = if i == 0 { 0..n } else { left..right };
                    let k = rng.random(0..range.len());
                    (range, k)
                })
                .collect();
            let expected: Vec<_> = queries
                .iter()
                .map(|(r, k)| {
                    let mut sorted = values[r.clone()].to_vec();
                    sorted.sort_unstable();
                    sorted[*k]
                })
                .collect();
            let mut sorted = values.clone();
            sorted.sort_unstable();
            let mut sorted_ranks: Vec<_> = indices.iter().map(|_| rng.random(0..n)).collect();
            sorted_ranks.sort_unstable();
            let original = WaveletMatrix::new(values.clone());
            for binary_only in [false, true] {
                let mut wm = original.clone();
                if binary_only {
                    wm.quad_vectors.clear();
                }
                for_each_backend(&mut wm, |wm| {
                    for (i, ((value, range), positions)) in ranks.iter().zip(&positions).enumerate()
                    {
                        assert_eq!(wm.rank(*value, range.clone()), counts[i]);
                        assert_eq!(wm.rank_lessthan(*value, range.clone()), less[i]);
                        assert_eq!(
                            wm.rank_range(ranges[i].0.clone(), range.clone()),
                            range_counts[i]
                        );
                        let k = rng.random(0..=positions.len());
                        assert_eq!(wm.select(*value, k), positions.get(k).copied());
                        assert_eq!(wm.select(*value, positions.len()), None);
                    }
                    for (&index, ((range, k), &value)) in
                        indices.iter().zip(queries.iter().zip(&expected))
                    {
                        assert_eq!(wm.access(index), values[index]);
                        assert_eq!(wm.quantile(range.clone(), *k), value);
                    }
                    for q in 0..=ranks.len() {
                        assert_eq!(wm.rank_batch(ranks[..q].iter().cloned()), counts[..q]);
                        assert_eq!(
                            wm.rank_lessthan_batch(ranks[..q].iter().cloned()),
                            less[..q]
                        );
                        assert_eq!(
                            wm.rank_range_batch(ranges[..q].iter().cloned()),
                            range_counts[..q]
                        );
                    }
                    for q in 0..=queries.len() {
                        assert_eq!(
                            wm.access_batch(indices[..q].iter().copied()),
                            indices[..q].iter().map(|&i| values[i]).collect::<Vec<_>>()
                        );
                        assert_eq!(
                            wm.quantile_batch(queries[..q].iter().cloned()),
                            expected[..q]
                        );
                        assert_eq!(
                            wm.quantiles_sorted(0..n, &sorted_ranks[..q]),
                            sorted_ranks[..q]
                                .iter()
                                .map(|&k| sorted[k])
                                .collect::<Vec<_>>()
                        );
                    }
                    if let Some((range, _)) = queries.last() {
                        let mut sorted = values[range.clone()].to_vec();
                        sorted.sort_unstable();
                        let mut ranks: Vec<_> =
                            (0..33).map(|_| rng.random(0..sorted.len())).collect();
                        ranks.sort_unstable();
                        assert_eq!(
                            wm.quantiles_sorted(range.clone(), &ranks),
                            ranks.iter().map(|&k| sorted[k]).collect::<Vec<_>>()
                        );
                        let mut outside = values.clone();
                        outside.drain(range.clone());
                        outside.sort_unstable();
                        if !outside.is_empty() {
                            let k = rng.random(0..outside.len());
                            assert_eq!(wm.quantile_outer(range.clone(), k), outside[k]);
                        }
                    }
                });
            }
        }
    }

    #[test]
    fn test_wavelet_matrix_fold() {
        use crate::algebra::{Associative, Commutative, Invertible, Magma, Unital};
        use std::cmp::Reverse;

        enum Sum {}
        impl Magma for Sum {
            type T = Box<i64>;
            fn operate(a: &Self::T, b: &Self::T) -> Self::T {
                Box::new(**a + **b)
            }
        }
        impl Unital for Sum {
            fn unit() -> Self::T {
                Box::new(0)
            }
        }
        impl Associative for Sum {}
        impl Commutative for Sum {}
        impl Invertible for Sum {
            fn inverse(a: &Self::T) -> Self::T {
                Box::new(-**a)
            }
        }

        let mut rng = Xorshift::default();
        for n in 0..=65 {
            let sigma: i64 = rng.random(1..=16);
            let values: Vec<_> = (0..n)
                .map(|_| Box::new(rng.random(-sigma..=sigma)))
                .collect();
            let weights: Vec<_> = (0..n).map(|_| Box::new(rng.random(-100..=100))).collect();
            let mut dictionary = values.clone();
            dictionary.sort_unstable();
            dictionary.dedup();
            let height = usize::BITS as usize - dictionary.len().leading_zeros() as usize;
            let levels: Vec<Vec<_>> = (0..height)
                .map(|d| {
                    let mut order: Vec<_> = (0..n).collect();
                    // Each lower bit takes precedence over the previously partitioned higher bits.
                    order.sort_by_key(|&i| {
                        (dictionary.binary_search(&values[i]).unwrap() >> d).reverse_bits()
                    });
                    order
                })
                .collect();
            let mut expected = Vec::new();
            for (d, order) in levels.iter().enumerate() {
                for (position, &i) in order.iter().enumerate() {
                    expected.push((i, Reverse(d), position, values[i].clone()));
                }
            }
            expected.sort_by_key(|&(i, d, _, _)| (i, d));
            let mut callbacks = Vec::new();
            let original = WaveletMatrix::new_with_init(values.clone(), |d, i, value| {
                callbacks.push((d, i, value))
            });
            assert_eq!(
                callbacks,
                expected
                    .into_iter()
                    .map(|(_, Reverse(d), i, value)| (d, i, value))
                    .collect::<Vec<_>>()
            );
            let queries: Vec<_> = (0..33)
                .map(|i| {
                    if i == 0 {
                        return (Box::new(-sigma - 1)..Box::new(sigma + 1), 0..n);
                    }
                    let left = rng.random(0..=n);
                    let right = rng.random(left..=n);
                    let lower = rng.random(-sigma - 1..=sigma + 1);
                    let upper = rng.random(lower..=sigma + 1);
                    (Box::new(lower)..Box::new(upper), left..right)
                })
                .collect();
            for binary_only in [false, true] {
                let mut wm = original.clone();
                if binary_only {
                    wm.quad_vectors.clear();
                }
                for_each_backend(&mut wm, |wm| {
                    let fold = wm.build_fold::<Sum>(&weights);
                    let mut dynamic = wm.build_point_add::<Sum>(&weights);
                    let mut updated = weights.clone();
                    let mut less_sums = Vec::new();
                    for (step, (bounds, range)) in queries.iter().enumerate() {
                        if n != 0 {
                            let i = if step == 0 { n - 1 } else { rng.random(0..n) };
                            let delta = rng.random(-100..=100);
                            dynamic.update(i, Box::new(delta));
                            *updated[i] += delta;
                        }
                        let selected: Vec<_> = range
                            .clone()
                            .filter(|&i| bounds.contains(&values[i]))
                            .collect();
                        let sum: i64 = selected.iter().map(|&i| *weights[i]).sum();
                        let updated_sum: i64 = selected.iter().map(|&i| *updated[i]).sum();
                        assert_eq!(*fold.fold_range(bounds.clone(), range.clone()), sum);
                        assert_eq!(
                            fold.fold_range_with_count(bounds.clone(), range.clone()),
                            (selected.len(), Box::new(sum))
                        );
                        assert_eq!(
                            *dynamic.fold_range(bounds.clone(), range.clone()),
                            updated_sum
                        );
                        let selected: Vec<_> =
                            range.clone().filter(|&i| values[i] < bounds.end).collect();
                        let sum: i64 = selected.iter().map(|&i| *weights[i]).sum();
                        let updated_sum: i64 = selected.iter().map(|&i| *updated[i]).sum();
                        assert_eq!(*fold.fold_lessthan(bounds.end.clone(), range.clone()), sum);
                        assert_eq!(
                            fold.fold_lessthan_with_count(bounds.end.clone(), range.clone()),
                            (selected.len(), Box::new(sum))
                        );
                        assert_eq!(
                            *dynamic.fold_lessthan(bounds.end.clone(), range.clone()),
                            updated_sum
                        );
                        less_sums.push(Box::new(sum));
                        let mut actual = Vec::new();
                        let mut dimensions = Vec::new();
                        wm.query_less_than(bounds.end.clone(), range.clone(), |d, r| {
                            dimensions.push(d);
                            actual.extend_from_slice(&levels[d][r]);
                        });
                        actual.sort_unstable();
                        assert_eq!(actual, selected);
                        let bound = dictionary.partition_point(|v| v < &bounds.end);
                        assert_eq!(
                            dimensions,
                            (0..height)
                                .rev()
                                .filter(|&d| (bound >> d) & 1 != 0)
                                .collect::<Vec<_>>()
                        );
                    }
                    for q in 0..=queries.len() {
                        assert_eq!(
                            fold.fold_lessthan_batch(
                                queries[..q].iter().map(|(v, r)| (v.end.clone(), r.clone()))
                            ),
                            less_sums[..q]
                        );
                    }
                });
            }
        }
    }

    fn for_each_backend<T>(wm: &mut WaveletMatrix<T>, mut test: impl FnMut(&WaveletMatrix<T>)) {
        #[cfg(target_arch = "x86_64")]
        for backend in [
            crate::tools::SimdBackend::Scalar,
            crate::tools::SimdBackend::Avx2,
            crate::tools::SimdBackend::Avx512,
        ] {
            if (backend == crate::tools::SimdBackend::Avx2 && !is_x86_feature_detected!("avx2"))
                || (backend == crate::tools::SimdBackend::Avx512
                    && !(is_x86_feature_detected!("avx512f")
                        && is_x86_feature_detected!("avx512vpopcntdq")))
            {
                continue;
            }
            wm.backend = backend;
            test(wm);
        }
        #[cfg(not(target_arch = "x86_64"))]
        test(wm);
    }
}
