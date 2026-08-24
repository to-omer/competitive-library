#[cfg(target_arch = "x86_64")]
use super::avx512_enabled;
use std::{
    cmp::Ordering,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
        ShrAssign,
    },
};

const BIT_AND: u8 = 0;
const BIT_OR: u8 = 1;
const BIT_XOR: u8 = 2;
#[cfg(target_arch = "x86_64")]
const SIMD_MIN_BLOCKS: usize = 8;

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Block([u64; 8]);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitSet {
    size: usize,
    bits: Vec<Block>,
}

impl BitSet {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            bits: vec![Block::default(); size.div_ceil(512)],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn ones(size: usize) -> Self {
        let mut self_ = Self {
            size,
            bits: vec![Block([u64::MAX; 8]); size.div_ceil(512)],
        };
        self_.trim();
        self_
    }

    pub fn get(&self, i: usize) -> bool {
        self.bits[i >> 9].0[i >> 6 & 7] & (1 << (i & 63)) != 0
    }

    pub fn set(&mut self, i: usize, b: bool) {
        let word = &mut self.bits[i >> 9].0[i >> 6 & 7];
        if b {
            *word |= 1 << (i & 63);
        } else {
            *word &= !(1 << (i & 63));
        }
    }

    /// Clears all bits.
    pub fn reset(&mut self) {
        self.bits.fill(Block::default());
    }

    /// Sets all bits to `value`.
    pub fn fill(&mut self, value: bool) {
        self.bits.fill(Block([if value { u64::MAX } else { 0 }; 8]));
        self.trim();
    }

    /// Tests whether any bit is set.
    #[inline]
    pub fn any(&self) -> bool {
        !self.none()
    }

    /// Tests whether all bits are unset.
    #[inline]
    pub fn none(&self) -> bool {
        #[cfg(target_arch = "x86_64")]
        if self.bits.len() >= SIMD_MIN_BLOCKS {
            if self.bits[0].0[0] != 0 {
                return false;
            }
            if avx512_enabled() && is_x86_feature_detected!("avx512f") {
                // SAFETY: blocks are 64-byte aligned and feature detection checked AVX-512F.
                return unsafe { simd::none_avx512(&self.bits) };
            }
            if is_x86_feature_detected!("avx2") {
                // SAFETY: 64-byte alignment also satisfies AVX2 and feature detection checked it.
                return unsafe { simd::none_avx2(&self.bits) };
            }
        }
        self.words().iter().all(|&word| word == 0)
    }

    /// Tests whether all bits are set.
    #[inline(always)]
    pub fn all(&self) -> bool {
        let words = self.words();
        let full_words = self.size >> 6;
        #[cfg(target_arch = "x86_64")]
        if self.size >> 9 >= SIMD_MIN_BLOCKS {
            let full_blocks = self.size >> 9;
            if words[0] != u64::MAX {
                return false;
            }
            let full_blocks_are_set = if avx512_enabled() && is_x86_feature_detected!("avx512f") {
                // SAFETY: blocks are 64-byte aligned and feature detection checked AVX-512F.
                Some(unsafe { simd::all_avx512(&self.bits[..full_blocks]) })
            } else if is_x86_feature_detected!("avx2") {
                // SAFETY: 64-byte alignment also satisfies AVX2 and feature detection checked it.
                Some(unsafe { simd::all_avx2(&self.bits[..full_blocks]) })
            } else {
                None
            };
            if let Some(full_blocks_are_set) = full_blocks_are_set {
                return full_blocks_are_set
                    && words[full_blocks * 8..full_words]
                        .iter()
                        .all(|&word| word == u64::MAX)
                    && (self.size & 63 == 0
                        || words[full_words] == u64::MAX >> (64 - (self.size & 63)));
            }
        }
        if self.size & 63 == 0 {
            return words.iter().all(|&word| word == u64::MAX);
        }
        words[..full_words].iter().all(|&word| word == u64::MAX)
            && words[full_words] == u64::MAX >> (64 - (self.size & 63))
    }

    /// Iterates over set-bit indices in ascending order.
    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.words()
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(word_index, mut word)| {
                std::iter::from_fn(move || {
                    if word == 0 {
                        None
                    } else {
                        let bit = word.trailing_zeros() as usize;
                        word &= word - 1;
                        Some((word_index << 6) | bit)
                    }
                })
            })
    }

    /// Counts set bits.
    #[inline]
    pub fn count_ones(&self) -> u64 {
        let words = self.words();
        #[cfg(target_arch = "x86_64")]
        if words.len() >= 8 {
            if avx512_enabled()
                && is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512vpopcntdq")
            {
                // SAFETY: blocks are aligned and feature detection checked both requirements.
                return unsafe { simd::count_ones_avx512(&self.bits) };
            }
            if is_x86_feature_detected!("avx2") {
                // SAFETY: 64-byte alignment also satisfies AVX2 and feature detection checked it.
                return unsafe { simd::count_ones_avx2(&self.bits) };
            }
        }
        words.iter().map(|word| word.count_ones() as u64).sum()
    }

    /// Counts unset bits.
    #[inline]
    pub fn count_zeros(&self) -> u64 {
        self.size as u64 - self.count_ones()
    }

    pub fn push(&mut self, b: bool) {
        if self.size & 511 == 0 {
            self.bits.push(Block::default());
        }
        if b {
            self.bits[self.size >> 9].0[self.size >> 6 & 7] |= 1 << (self.size & 63);
        }
        self.size += 1;
    }

    pub fn resize(&mut self, new_size: usize) {
        match self.size.cmp(&new_size) {
            Ordering::Less => self.bits.resize(new_size.div_ceil(512), Block::default()),
            Ordering::Equal => {}
            Ordering::Greater => self.bits.truncate(new_size.div_ceil(512)),
        }
        self.size = new_size;
        self.trim();
    }

    /// Assigns `self | (self << rhs)` to `self`.
    #[inline]
    pub fn shl_bitor_assign(&mut self, rhs: usize) {
        self.shift_left::<true>(rhs);
    }

    /// Assigns `self | (self >> rhs)` to `self`.
    #[inline]
    pub fn shr_bitor_assign(&mut self, rhs: usize) {
        self.shift_right::<true>(rhs);
    }

    fn words(&self) -> &[u64] {
        // SAFETY: `Block` is exactly eight contiguous `u64`s, with no trailing padding.
        unsafe { std::slice::from_raw_parts(self.bits.as_ptr().cast(), self.size.div_ceil(64)) }
    }

    fn words_mut(&mut self) -> &mut [u64] {
        let len = self.size.div_ceil(64);
        // SAFETY: `Block` is exactly eight contiguous `u64`s, with no trailing padding.
        unsafe { std::slice::from_raw_parts_mut(self.bits.as_mut_ptr().cast(), len) }
    }

    fn trim(&mut self) {
        let used_words = self.size.div_ceil(64) & 7;
        if let Some(last) = self.bits.last_mut() {
            if self.size & 63 != 0 {
                last.0[(self.size >> 6) & 7] &= u64::MAX >> (64 - (self.size & 63));
            }
            if used_words != 0 {
                last.0[used_words..].fill(0);
            }
        }
    }

    #[inline]
    fn bitop_assign<const OP: u8>(&mut self, rhs: &Self) {
        assert_eq!(
            self.size, rhs.size,
            "bitwise operations require equal lengths"
        );
        #[cfg(target_arch = "x86_64")]
        if self.bits.len() >= SIMD_MIN_BLOCKS {
            if avx512_enabled() && is_x86_feature_detected!("avx512f") {
                // SAFETY: blocks are 64-byte aligned and feature detection checked AVX-512F.
                unsafe { simd::bitop_avx512::<OP>(&mut self.bits, &rhs.bits) };
                return;
            }
            if is_x86_feature_detected!("avx2") {
                // SAFETY: 64-byte alignment also satisfies AVX2 and feature detection checked it.
                unsafe { simd::bitop_avx2::<OP>(&mut self.bits, &rhs.bits) };
                return;
            }
        }
        for (lhs, &rhs) in self.words_mut().iter_mut().zip(rhs.words()) {
            *lhs = match OP {
                BIT_AND => *lhs & rhs,
                BIT_OR => *lhs | rhs,
                BIT_XOR => *lhs ^ rhs,
                _ => unreachable!(),
            };
        }
    }

    #[inline]
    fn shift_left<const OR_ASSIGN: bool>(&mut self, rhs: usize) {
        if rhs == 0 {
            return;
        }
        if rhs >= self.size {
            if !OR_ASSIGN {
                self.reset();
            }
            return;
        }
        #[cfg(target_arch = "x86_64")]
        if self.bits.len() >= SIMD_MIN_BLOCKS {
            if avx512_enabled()
                && is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512vbmi2")
            {
                // SAFETY: blocks are aligned and feature detection checked AVX-512F and VBMI2.
                unsafe { simd::shift_left_avx512::<OR_ASSIGN>(&mut self.bits, rhs) };
                if self.size & 511 != 0 {
                    self.trim();
                }
                return;
            }
            if is_x86_feature_detected!("avx2") {
                // SAFETY: feature detection checked AVX2 support.
                unsafe { simd::shift_left_avx2::<OR_ASSIGN>(self.words_mut(), rhs) };
                if self.size & 63 != 0 {
                    self.trim();
                }
                return;
            }
        }

        let bits = self.words_mut();
        let word_shift = rhs >> 6;
        let bit_shift = rhs & 63;
        if bit_shift == 0 {
            for i in (0..bits.len() - word_shift).rev() {
                if OR_ASSIGN {
                    bits[i + word_shift] |= bits[i];
                } else {
                    bits[i + word_shift] = bits[i];
                }
            }
        } else {
            for i in (1..bits.len() - word_shift).rev() {
                let value = (bits[i] << bit_shift) | (bits[i - 1] >> (64 - bit_shift));
                if OR_ASSIGN {
                    bits[i + word_shift] |= value;
                } else {
                    bits[i + word_shift] = value;
                }
            }
            if OR_ASSIGN {
                bits[word_shift] |= bits[0] << bit_shift;
            } else {
                bits[word_shift] = bits[0] << bit_shift;
            }
        }
        if !OR_ASSIGN {
            bits[..word_shift].fill(0);
        }
        if self.size & 63 != 0 {
            self.trim();
        }
    }

    #[inline]
    fn shift_right<const OR_ASSIGN: bool>(&mut self, rhs: usize) {
        if rhs == 0 {
            return;
        }
        if rhs >= self.size {
            if !OR_ASSIGN {
                self.reset();
            }
            return;
        }
        #[cfg(target_arch = "x86_64")]
        if self.bits.len() >= SIMD_MIN_BLOCKS {
            if avx512_enabled()
                && is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512vbmi2")
            {
                // SAFETY: blocks are aligned and feature detection checked AVX-512F and VBMI2.
                unsafe { simd::shift_right_avx512::<OR_ASSIGN>(&mut self.bits, rhs) };
                return;
            }
            if is_x86_feature_detected!("avx2") {
                // SAFETY: feature detection checked AVX2 support.
                unsafe { simd::shift_right_avx2::<OR_ASSIGN>(self.words_mut(), rhs) };
                return;
            }
        }

        let bits = self.words_mut();
        let word_shift = rhs >> 6;
        let bit_shift = rhs & 63;
        if bit_shift == 0 {
            for i in word_shift..bits.len() {
                if OR_ASSIGN {
                    bits[i - word_shift] |= bits[i];
                } else {
                    bits[i - word_shift] = bits[i];
                }
            }
        } else {
            for i in word_shift..bits.len() - 1 {
                let value = (bits[i] >> bit_shift) | (bits[i + 1] << (64 - bit_shift));
                if OR_ASSIGN {
                    bits[i - word_shift] |= value;
                } else {
                    bits[i - word_shift] = value;
                }
            }
            if OR_ASSIGN {
                bits[bits.len() - word_shift - 1] |= bits[bits.len() - 1] >> bit_shift;
            } else {
                bits[bits.len() - word_shift - 1] = bits[bits.len() - 1] >> bit_shift;
            }
        }
        if !OR_ASSIGN {
            let end = bits.len() - word_shift;
            bits[end..].fill(0);
        }
    }
}

impl Extend<bool> for BitSet {
    fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
        for bit in iter {
            self.push(bit);
        }
    }
}

impl FromIterator<bool> for BitSet {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let mut set = BitSet::new(0);
        set.extend(iter);
        set
    }
}

impl ShlAssign<usize> for BitSet {
    #[inline]
    fn shl_assign(&mut self, rhs: usize) {
        self.shift_left::<false>(rhs);
    }
}

impl Shl<usize> for BitSet {
    type Output = Self;
    fn shl(mut self, rhs: usize) -> Self::Output {
        self <<= rhs;
        self
    }
}

impl ShrAssign<usize> for BitSet {
    #[inline]
    fn shr_assign(&mut self, rhs: usize) {
        self.shift_right::<false>(rhs);
    }
}

impl Shr<usize> for BitSet {
    type Output = Self;
    fn shr(mut self, rhs: usize) -> Self::Output {
        self >>= rhs;
        self
    }
}

impl BitOrAssign<&BitSet> for BitSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: &Self) {
        self.bitop_assign::<BIT_OR>(rhs);
    }
}

impl BitOr<&BitSet> for BitSet {
    type Output = Self;
    fn bitor(mut self, rhs: &Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOr<&BitSet> for &BitSet {
    type Output = BitSet;
    fn bitor(self, rhs: &BitSet) -> Self::Output {
        let mut res = self.clone();
        res |= rhs;
        res
    }
}

impl BitAndAssign<&BitSet> for BitSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: &Self) {
        self.bitop_assign::<BIT_AND>(rhs);
    }
}

impl BitAnd<&BitSet> for BitSet {
    type Output = Self;
    fn bitand(mut self, rhs: &Self) -> Self::Output {
        self &= rhs;
        self
    }
}

impl BitAnd<&BitSet> for &BitSet {
    type Output = BitSet;
    fn bitand(self, rhs: &BitSet) -> Self::Output {
        let mut res = self.clone();
        res &= rhs;
        res
    }
}

impl BitXorAssign<&BitSet> for BitSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &Self) {
        self.bitop_assign::<BIT_XOR>(rhs);
    }
}

impl BitXor<&BitSet> for BitSet {
    type Output = Self;
    fn bitxor(mut self, rhs: &Self) -> Self::Output {
        self ^= rhs;
        self
    }
}

impl BitXor<&BitSet> for &BitSet {
    type Output = BitSet;
    fn bitxor(self, rhs: &BitSet) -> Self::Output {
        let mut res = self.clone();
        res ^= rhs;
        res
    }
}

impl Not for BitSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for word in self.words_mut() {
            *word = !*word;
        }
        self.trim();
        self
    }
}

impl Not for &BitSet {
    type Output = BitSet;
    fn not(self) -> Self::Output {
        !self.clone()
    }
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)] // SIMD intrinsics and raw pointers are confined here
mod simd {
    use super::{BIT_AND, BIT_OR, BIT_XOR, Block};
    use std::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    pub unsafe fn bitop_avx2<const OP: u8>(lhs: &mut [Block], rhs: &[Block]) {
        let lhs_ptr = lhs.as_mut_ptr().cast::<__m256i>();
        let rhs_ptr = rhs.as_ptr().cast::<__m256i>();
        for i in 0..lhs.len() * 2 {
            let lhs_value = _mm256_load_si256(lhs_ptr.add(i));
            let rhs_value = _mm256_load_si256(rhs_ptr.add(i));
            let value = match OP {
                BIT_AND => _mm256_and_si256(lhs_value, rhs_value),
                BIT_OR => _mm256_or_si256(lhs_value, rhs_value),
                BIT_XOR => _mm256_xor_si256(lhs_value, rhs_value),
                _ => unreachable!(),
            };
            _mm256_store_si256(lhs_ptr.add(i), value);
        }
    }

    #[target_feature(enable = "avx512f")]
    pub unsafe fn bitop_avx512<const OP: u8>(lhs: &mut [Block], rhs: &[Block]) {
        for i in 0..lhs.len() {
            let lhs_value = _mm512_load_si512(lhs.as_ptr().add(i).cast());
            let rhs_value = _mm512_load_si512(rhs.as_ptr().add(i).cast());
            let value = match OP {
                BIT_AND => _mm512_and_si512(lhs_value, rhs_value),
                BIT_OR => _mm512_or_si512(lhs_value, rhs_value),
                BIT_XOR => _mm512_xor_si512(lhs_value, rhs_value),
                _ => unreachable!(),
            };
            _mm512_store_si512(lhs.as_mut_ptr().add(i).cast(), value);
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn count_ones_avx2(bits: &[Block]) -> u64 {
        let table = _mm256_set_epi64x(
            0x0403_0302_0302_0201,
            0x0302_0201_0201_0100,
            0x0403_0302_0302_0201,
            0x0302_0201_0201_0100,
        );
        let low_mask = _mm256_set1_epi8(0x0f);
        let zero = _mm256_setzero_si256();
        let mut sum = zero;
        let ptr = bits.as_ptr().cast::<__m256i>();
        for i in 0..bits.len() * 2 {
            let value = _mm256_load_si256(ptr.add(i));
            let low = _mm256_shuffle_epi8(table, _mm256_and_si256(value, low_mask));
            let high = _mm256_shuffle_epi8(
                table,
                _mm256_and_si256(_mm256_srli_epi16::<4>(value), low_mask),
            );
            sum = _mm256_add_epi64(sum, _mm256_sad_epu8(_mm256_add_epi8(low, high), zero));
        }
        let mut lanes = [0; 4];
        _mm256_storeu_si256(lanes.as_mut_ptr().cast(), sum);
        lanes.into_iter().sum()
    }

    #[target_feature(enable = "avx512f,avx512vpopcntdq")]
    pub unsafe fn count_ones_avx512(bits: &[Block]) -> u64 {
        let mut sum = _mm512_setzero_si512();
        for i in 0..bits.len() {
            let value = _mm512_load_si512(bits.as_ptr().add(i).cast());
            sum = _mm512_add_epi64(sum, _mm512_popcnt_epi64(value));
        }
        let mut lanes = [0; 8];
        _mm512_storeu_si512(lanes.as_mut_ptr().cast(), sum);
        lanes.into_iter().sum()
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn none_avx2(bits: &[Block]) -> bool {
        let ptr = bits.as_ptr().cast::<__m256i>();
        for i in 0..bits.len() * 2 {
            let value = _mm256_load_si256(ptr.add(i));
            if _mm256_testz_si256(value, value) == 0 {
                return false;
            }
        }
        true
    }

    #[target_feature(enable = "avx512f")]
    pub unsafe fn none_avx512(bits: &[Block]) -> bool {
        let mut i = 0;
        while i + 4 <= bits.len() {
            let value = _mm512_or_si512(
                _mm512_or_si512(
                    _mm512_load_si512(bits.as_ptr().add(i).cast()),
                    _mm512_load_si512(bits.as_ptr().add(i + 1).cast()),
                ),
                _mm512_or_si512(
                    _mm512_load_si512(bits.as_ptr().add(i + 2).cast()),
                    _mm512_load_si512(bits.as_ptr().add(i + 3).cast()),
                ),
            );
            if _mm512_test_epi64_mask(value, value) != 0 {
                return false;
            }
            i += 4;
        }
        while i < bits.len() {
            let value = _mm512_load_si512(bits.as_ptr().add(i).cast());
            if _mm512_test_epi64_mask(value, value) != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn all_avx2(bits: &[Block]) -> bool {
        let ones = _mm256_set1_epi64x(-1);
        let ptr = bits.as_ptr().cast::<__m256i>();
        for i in 0..bits.len() * 2 {
            if _mm256_movemask_epi8(_mm256_cmpeq_epi64(_mm256_load_si256(ptr.add(i)), ones)) != -1 {
                return false;
            }
        }
        true
    }

    #[target_feature(enable = "avx512f")]
    pub unsafe fn all_avx512(bits: &[Block]) -> bool {
        let ones = _mm512_set1_epi64(-1);
        let mut i = 0;
        while i + 4 <= bits.len() {
            let value = _mm512_and_si512(
                _mm512_and_si512(
                    _mm512_load_si512(bits.as_ptr().add(i).cast()),
                    _mm512_load_si512(bits.as_ptr().add(i + 1).cast()),
                ),
                _mm512_and_si512(
                    _mm512_load_si512(bits.as_ptr().add(i + 2).cast()),
                    _mm512_load_si512(bits.as_ptr().add(i + 3).cast()),
                ),
            );
            if _mm512_cmpeq_epi64_mask(value, ones) != u8::MAX {
                return false;
            }
            i += 4;
        }
        while i < bits.len() {
            if _mm512_cmpeq_epi64_mask(_mm512_load_si512(bits.as_ptr().add(i).cast()), ones)
                != u8::MAX
            {
                return false;
            }
            i += 1;
        }
        true
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn shift_left_avx2<const OR_ASSIGN: bool>(bits: &mut [u64], rhs: usize) {
        let word_shift = rhs >> 6;
        let bit_shift = rhs & 63;
        let lower = word_shift + usize::from(bit_shift != 0);
        let mut end = bits.len();
        let count = _mm_cvtsi64_si128(bit_shift as i64);
        while end >= lower + 4 {
            let start = end - 4;
            let value = _mm256_loadu_si256(bits.as_ptr().add(start - word_shift).cast());
            let mut value = if bit_shift == 0 {
                value
            } else {
                _mm256_or_si256(
                    _mm256_sll_epi64(value, count),
                    _mm256_srl_epi64(
                        _mm256_loadu_si256(bits.as_ptr().add(start - word_shift - 1).cast()),
                        _mm_cvtsi64_si128((64 - bit_shift) as i64),
                    ),
                )
            };
            if OR_ASSIGN {
                value = _mm256_or_si256(value, _mm256_loadu_si256(bits.as_ptr().add(start).cast()));
            }
            _mm256_storeu_si256(bits.as_mut_ptr().add(start).cast(), value);
            end = start;
        }
        for i in (lower..end).rev() {
            let source = i - word_shift;
            let value = if bit_shift == 0 {
                bits[source]
            } else {
                (bits[source] << bit_shift) | (bits[source - 1] >> (64 - bit_shift))
            };
            if OR_ASSIGN {
                bits[i] |= value;
            } else {
                bits[i] = value;
            }
        }
        if bit_shift != 0 {
            if OR_ASSIGN {
                bits[word_shift] |= bits[0] << bit_shift;
            } else {
                bits[word_shift] = bits[0] << bit_shift;
            }
        }
        if !OR_ASSIGN {
            bits[..word_shift].fill(0);
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn shift_right_avx2<const OR_ASSIGN: bool>(bits: &mut [u64], rhs: usize) {
        let word_shift = rhs >> 6;
        let bit_shift = rhs & 63;
        let upper = bits.len() - word_shift - usize::from(bit_shift != 0);
        let mut start = 0;
        let count = _mm_cvtsi64_si128(bit_shift as i64);
        while start + 4 <= upper {
            let value = _mm256_loadu_si256(bits.as_ptr().add(start + word_shift).cast());
            let mut value = if bit_shift == 0 {
                value
            } else {
                _mm256_or_si256(
                    _mm256_srl_epi64(value, count),
                    _mm256_sll_epi64(
                        _mm256_loadu_si256(bits.as_ptr().add(start + word_shift + 1).cast()),
                        _mm_cvtsi64_si128((64 - bit_shift) as i64),
                    ),
                )
            };
            if OR_ASSIGN {
                value = _mm256_or_si256(value, _mm256_loadu_si256(bits.as_ptr().add(start).cast()));
            }
            _mm256_storeu_si256(bits.as_mut_ptr().add(start).cast(), value);
            start += 4;
        }
        for i in start..upper {
            let source = i + word_shift;
            let value = if bit_shift == 0 {
                bits[source]
            } else {
                (bits[source] >> bit_shift) | (bits[source + 1] << (64 - bit_shift))
            };
            if OR_ASSIGN {
                bits[i] |= value;
            } else {
                bits[i] = value;
            }
        }
        if bit_shift != 0 {
            if OR_ASSIGN {
                bits[bits.len() - word_shift - 1] |= bits[bits.len() - 1] >> bit_shift;
            } else {
                bits[bits.len() - word_shift - 1] = bits[bits.len() - 1] >> bit_shift;
            }
        }
        if !OR_ASSIGN {
            let end = bits.len() - word_shift;
            bits[end..].fill(0);
        }
    }

    #[target_feature(enable = "avx512f,avx512vbmi2")]
    pub unsafe fn shift_left_avx512<const OR_ASSIGN: bool>(bits: &mut [Block], rhs: usize) {
        let block_shift = rhs >> 9;
        let word_shift = rhs >> 6 & 7;
        let bit_shift = rhs & 63;
        if word_shift | bit_shift == 0 {
            for i in (block_shift..bits.len()).rev() {
                let mut value = _mm512_load_si512(bits.as_ptr().add(i - block_shift).cast());
                if OR_ASSIGN {
                    value = _mm512_or_si512(value, _mm512_load_si512(bits.as_ptr().add(i).cast()));
                }
                _mm512_store_si512(bits.as_mut_ptr().add(i).cast(), value);
            }
        } else {
            let zero = _mm512_setzero_si512();
            let indices = _mm512_setr_epi64(0, 1, 2, 3, 4, 5, 6, 7);
            let current_indices =
                _mm512_add_epi64(indices, _mm512_set1_epi64((8 - word_shift) as i64));
            let previous_indices = _mm512_sub_epi64(current_indices, _mm512_set1_epi64(1));
            let count = _mm512_set1_epi64(bit_shift as i64);
            for i in (block_shift..bits.len()).rev() {
                let source = i - block_shift;
                let previous = if source == 0 {
                    zero
                } else {
                    _mm512_load_si512(bits.as_ptr().add(source - 1).cast())
                };
                let current = _mm512_load_si512(bits.as_ptr().add(source).cast());
                let value = if word_shift == 0 {
                    current
                } else {
                    _mm512_permutex2var_epi64(previous, current_indices, current)
                };
                let mut value = if bit_shift == 0 {
                    value
                } else {
                    _mm512_shldv_epi64(
                        value,
                        _mm512_permutex2var_epi64(previous, previous_indices, current),
                        count,
                    )
                };
                if OR_ASSIGN {
                    value = _mm512_or_si512(value, _mm512_load_si512(bits.as_ptr().add(i).cast()));
                }
                _mm512_store_si512(bits.as_mut_ptr().add(i).cast(), value);
            }
        }
        if !OR_ASSIGN {
            bits[..block_shift].fill(Block::default());
        }
    }

    #[target_feature(enable = "avx512f,avx512vbmi2")]
    pub unsafe fn shift_right_avx512<const OR_ASSIGN: bool>(bits: &mut [Block], rhs: usize) {
        let block_shift = rhs >> 9;
        let word_shift = rhs >> 6 & 7;
        let bit_shift = rhs & 63;
        let remaining = bits.len() - block_shift;
        if word_shift | bit_shift == 0 {
            for i in 0..remaining {
                let mut value = _mm512_load_si512(bits.as_ptr().add(i + block_shift).cast());
                if OR_ASSIGN {
                    value = _mm512_or_si512(value, _mm512_load_si512(bits.as_ptr().add(i).cast()));
                }
                _mm512_store_si512(bits.as_mut_ptr().add(i).cast(), value);
            }
        } else {
            let zero = _mm512_setzero_si512();
            let indices = _mm512_setr_epi64(0, 1, 2, 3, 4, 5, 6, 7);
            let current_indices = _mm512_add_epi64(indices, _mm512_set1_epi64(word_shift as i64));
            let next_indices = _mm512_add_epi64(current_indices, _mm512_set1_epi64(1));
            let count = _mm512_set1_epi64(bit_shift as i64);
            for i in 0..remaining {
                let source = i + block_shift;
                let current = _mm512_load_si512(bits.as_ptr().add(source).cast());
                let next = if source + 1 == bits.len() {
                    zero
                } else {
                    _mm512_load_si512(bits.as_ptr().add(source + 1).cast())
                };
                let value = if word_shift == 0 {
                    current
                } else {
                    _mm512_permutex2var_epi64(current, current_indices, next)
                };
                let mut value = if bit_shift == 0 {
                    value
                } else {
                    _mm512_shrdv_epi64(
                        value,
                        _mm512_permutex2var_epi64(current, next_indices, next),
                        count,
                    )
                };
                if OR_ASSIGN {
                    value = _mm512_or_si512(value, _mm512_load_si512(bits.as_ptr().add(i).cast()));
                }
                _mm512_store_si512(bits.as_mut_ptr().add(i).cast(), value);
            }
        }
        if !OR_ASSIGN {
            bits[remaining..].fill(Block::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    const SIZES: [usize; 13] = [0, 1, 2, 63, 64, 65, 255, 256, 257, 511, 512, 513, 4097];

    fn bitset(model: &[bool]) -> BitSet {
        model.iter().copied().collect()
    }

    fn assert_model(actual: &BitSet, expected: &[bool]) {
        assert_eq!(actual.len(), expected.len());
        assert_eq!(
            actual.iter_ones().collect::<Vec<_>>(),
            expected
                .iter()
                .enumerate()
                .filter_map(|(i, &bit)| bit.then_some(i))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            actual.count_ones(),
            expected.iter().filter(|&&bit| bit).count() as u64
        );
        assert_eq!(
            actual.count_zeros(),
            expected.iter().filter(|&&bit| !bit).count() as u64
        );
        assert_eq!(actual.any(), expected.iter().any(|&bit| bit));
        assert_eq!(actual.none(), expected.iter().all(|&bit| !bit));
        assert_eq!(actual.all(), expected.iter().all(|&bit| bit));
        assert!(
            actual
                .bits
                .iter()
                .flat_map(|block| block.0)
                .skip(expected.len().div_ceil(64))
                .all(|word| word == 0)
        );
        if expected.len() & 63 != 0 {
            assert_eq!(actual.words().last().unwrap() >> (expected.len() & 63), 0);
        }
        for (i, &bit) in expected.iter().enumerate() {
            assert_eq!(actual.get(i), bit, "bit {i}");
        }
    }

    fn random_model(rng: &mut Xorshift, size: usize) -> Vec<bool> {
        (0..size)
            .map(|_| rng.random::<u64, _>(..) & 1 != 0)
            .collect()
    }

    #[test]
    fn access_fill_reset_push_extend_resize() {
        assert_eq!(std::mem::size_of::<Block>(), 64);
        assert_eq!(std::mem::align_of::<Block>(), 64);
        let aligned = BitSet::new(1);
        assert_eq!(aligned.bits.as_ptr() as usize & 63, 0);

        let mut rng = Xorshift::default();
        for size in SIZES {
            let model = random_model(&mut rng, size);
            let mut actual = bitset(&model);
            assert_model(&actual, &model);

            actual.fill(true);
            assert_model(&actual, &vec![true; size]);
            actual.fill(false);
            assert_model(&actual, &vec![false; size]);
            actual.fill(true);
            actual.reset();
            assert_model(&actual, &vec![false; size]);

            for (i, &value) in model.iter().enumerate() {
                actual.set(i, value);
            }
            assert_model(&actual, &model);

            let extra = random_model(&mut rng, 67);
            actual.extend(extra.iter().copied());
            let mut extended = model.clone();
            extended.extend(extra);
            assert_model(&actual, &extended);

            actual.resize(size / 2);
            assert_model(&actual, &model[..size / 2]);
            actual.resize(size + 70);
            let mut resized = model[..size / 2].to_vec();
            resized.resize(size + 70, false);
            assert_model(&actual, &resized);

            let value = rng.random::<u64, _>(..) & 1 != 0;
            actual.push(value);
            resized.push(value);
            assert_model(&actual, &resized);
        }
    }

    #[test]
    fn bitwise_operations_match_boolean_model() {
        let mut rng = Xorshift::default();
        for size in SIZES {
            let lhs = random_model(&mut rng, size);
            let rhs = random_model(&mut rng, size);
            let lhs_set = bitset(&lhs);
            let rhs_set = bitset(&rhs);

            assert_model(
                &(&lhs_set & &rhs_set),
                &lhs.iter()
                    .zip(&rhs)
                    .map(|(&x, &y)| x & y)
                    .collect::<Vec<_>>(),
            );
            assert_model(
                &(&lhs_set | &rhs_set),
                &lhs.iter()
                    .zip(&rhs)
                    .map(|(&x, &y)| x | y)
                    .collect::<Vec<_>>(),
            );
            assert_model(
                &(&lhs_set ^ &rhs_set),
                &lhs.iter()
                    .zip(&rhs)
                    .map(|(&x, &y)| x ^ y)
                    .collect::<Vec<_>>(),
            );
            assert_model(&!&lhs_set, &lhs.iter().map(|&x| !x).collect::<Vec<_>>());
        }
    }

    #[test]
    fn shifts_match_boolean_model_at_word_and_vector_boundaries() {
        let mut rng = Xorshift::default();
        for size in SIZES {
            let model = random_model(&mut rng, size);
            for shift in [0, 1, 63, 64, 65, 255, 256, 257, size, size + 1] {
                let mut expected_left = vec![false; size];
                let mut expected_right = vec![false; size];
                for (i, &value) in model.iter().enumerate() {
                    if let Some(i) = i.checked_add(shift)
                        && i < size
                    {
                        expected_left[i] = value;
                    }
                    if i >= shift {
                        expected_right[i - shift] = value;
                    }
                }

                let mut actual = bitset(&model);
                actual <<= shift;
                assert_model(&actual, &expected_left);
                let mut actual = bitset(&model);
                actual >>= shift;
                assert_model(&actual, &expected_right);

                let mut expected = model.clone();
                for (value, shifted) in expected.iter_mut().zip(&expected_left) {
                    *value |= *shifted;
                }
                let mut actual = bitset(&model);
                actual.shl_bitor_assign(shift);
                assert_model(&actual, &expected);

                let mut expected = model.clone();
                for (value, shifted) in expected.iter_mut().zip(&expected_right) {
                    *value |= *shifted;
                }
                let mut actual = bitset(&model);
                actual.shr_bitor_assign(shift);
                assert_model(&actual, &expected);
            }
        }
    }

    #[test]
    fn bitwise_operations_reject_different_lengths_without_mutation() {
        let rhs = BitSet::ones(65);
        for op in [BIT_AND, BIT_OR, BIT_XOR] {
            let mut lhs = BitSet::ones(64);
            let before = lhs.clone();
            let result = catch_unwind(AssertUnwindSafe(|| match op {
                BIT_AND => lhs &= &rhs,
                BIT_OR => lhs |= &rhs,
                BIT_XOR => lhs ^= &rhs,
                _ => unreachable!(),
            }));
            assert!(result.is_err());
            assert_eq!(lhs, before);
        }
    }
}
