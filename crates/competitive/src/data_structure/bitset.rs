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

    /// Parses ASCII `0` and `1`, with the first character at bit index zero.
    /// Returns `None` if any other character occurs.
    pub fn from_binary(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        let mut bits = Self::new(bytes.len());
        let end = bytes.len() / 64 * 64;
        #[cfg(target_arch = "x86_64")]
        let parsed = if avx512_enabled() && is_x86_feature_detected!("avx512bw") {
            // SAFETY: AVX-512BW is available; each complete chunk contains 64 bytes.
            unsafe { simd::parse_binary_avx512(&bytes[..end], bits.words_mut()) }
        } else if is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 is available; each complete chunk contains 64 bytes.
            unsafe { simd::parse_binary_avx2(&bytes[..end], bits.words_mut()) }
        } else {
            Self::parse_binary_scalar(&bytes[..end], bits.words_mut())
        };
        #[cfg(not(target_arch = "x86_64"))]
        let parsed = Self::parse_binary_scalar(&bytes[..end], bits.words_mut());
        if !parsed {
            return None;
        }
        if end != bytes.len() {
            let mut word = 0;
            for (i, &b) in bytes[end..].iter().enumerate() {
                if b != b'0' && b != b'1' {
                    return None;
                }
                word |= u64::from(b & 1) << i;
            }
            bits.words_mut()[end / 64] = word;
        }
        Some(bits)
    }

    fn parse_binary_scalar(bytes: &[u8], words: &mut [u64]) -> bool {
        for (chunk, word) in bytes.as_chunks::<64>().0.iter().zip(words) {
            for (i, byte) in chunk.as_chunks::<8>().0.iter().enumerate() {
                let x = u64::from_le_bytes(*byte);
                if x & 0xfefe_fefe_fefe_fefe != 0x3030_3030_3030_3030 {
                    return false;
                }
                *word |= ((x & 0x0101_0101_0101_0101).wrapping_mul(0x0102_0408_1020_4080) >> 56)
                    << (i * 8);
            }
        }
        true
    }

    /// Returns ASCII `0` and `1` in increasing bit-index order.
    pub fn to_binary(&self) -> String {
        const TABLE: [[u8; 8]; 256] = {
            let mut table = [[b'0'; 8]; 256];
            let mut i = 0;
            while i < 256 {
                let mut j = 0;
                while j < 8 {
                    table[i][j] |= ((i >> j) & 1) as u8;
                    j += 1;
                }
                i += 1;
            }
            table
        };
        let mut bytes = vec![b'0'; self.size.div_ceil(8) * 8];
        #[cfg(target_arch = "x86_64")]
        let end = if self.size >= 64 && avx512_enabled() && is_x86_feature_detected!("avx512bw") {
            let end = self.size / 64 * 64;
            // SAFETY: AVX-512BW is available; each output chunk holds one 64-bit word.
            unsafe {
                simd::write_binary_avx512(&mut bytes[..end], self.words());
            }
            end
        } else if self.size >= 64 && is_x86_feature_detected!("avx2") {
            let end = self.size / 64 * 64;
            // SAFETY: AVX2 is available; each output chunk holds one complete 64-bit word.
            unsafe {
                simd::write_binary_avx2(&mut bytes[..end], self.words());
            }
            end
        } else {
            0
        };
        #[cfg(not(target_arch = "x86_64"))]
        let end = 0;
        for (chunk, &word) in bytes[end..].chunks_mut(64).zip(&self.words()[end / 64..]) {
            for (i, byte) in chunk.as_chunks_mut::<8>().0.iter_mut().enumerate() {
                byte.copy_from_slice(&TABLE[(word >> (i * 8) & 255) as usize]);
            }
        }
        bytes.truncate(self.size);
        // SAFETY: every output byte is ASCII `0` or `1`.
        unsafe { String::from_utf8_unchecked(bytes) }
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

    /// Returns words in increasing bit-index order; unused high bits in the last word are zero.
    pub fn words(&self) -> &[u64] {
        // SAFETY: `Block` is exactly eight contiguous `u64`s, with no trailing padding.
        unsafe { std::slice::from_raw_parts(self.bits.as_ptr().cast(), self.size.div_ceil(64)) }
    }

    /// Returns mutable words in increasing bit-index order.
    /// Callers must keep unused high bits in the last word zero.
    pub fn words_mut(&mut self) -> &mut [u64] {
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

    #[target_feature(enable = "avx512bw")]
    pub unsafe fn write_binary_avx512(bytes: &mut [u8], words: &[u64]) {
        let one = _mm512_set1_epi8(1);
        let zero = _mm512_set1_epi8(b'0' as i8);
        for (chunk, &word) in bytes.as_chunks_mut::<64>().0.iter_mut().zip(words) {
            let value = _mm512_mask_add_epi8(zero, word, zero, one);
            _mm512_storeu_si512(chunk.as_mut_ptr().cast(), value);
        }
    }
    #[target_feature(enable = "avx2")]
    pub unsafe fn write_binary_avx2(bytes: &mut [u8], words: &[u64]) {
        let indices = _mm256_setr_epi64x(
            0,
            0x0101_0101_0101_0101,
            0x0202_0202_0202_0202,
            0x0303_0303_0303_0303,
        );
        let mask = _mm256_set1_epi64x(0x8040_2010_0804_0201u64 as i64);
        let one = _mm256_set1_epi8(b'1' as i8);
        let zero = _mm256_setzero_si256();
        for (chunk, &word) in bytes.as_chunks_mut::<64>().0.iter_mut().zip(words) {
            let lo = _mm256_shuffle_epi8(_mm256_set1_epi32(word as i32), indices);
            let hi = _mm256_shuffle_epi8(_mm256_set1_epi32((word >> 32) as i32), indices);
            let lo = _mm256_add_epi8(one, _mm256_cmpeq_epi8(_mm256_and_si256(lo, mask), zero));
            let hi = _mm256_add_epi8(one, _mm256_cmpeq_epi8(_mm256_and_si256(hi, mask), zero));
            _mm256_storeu_si256(chunk.as_mut_ptr().cast(), lo);
            _mm256_storeu_si256(chunk.as_mut_ptr().add(32).cast(), hi);
        }
    }
    #[target_feature(enable = "avx2")]
    pub unsafe fn parse_binary_avx2(bytes: &[u8], words: &mut [u64]) -> bool {
        let one = _mm256_set1_epi8(b'1' as i8);
        let mask = _mm256_set1_epi8(!1);
        let zero = _mm256_set1_epi8(b'0' as i8);
        for (chunk, word) in bytes.as_chunks::<64>().0.iter().zip(words) {
            let a = _mm256_loadu_si256(chunk.as_ptr().cast());
            let b = _mm256_loadu_si256(chunk.as_ptr().add(32).cast());
            let valid_a = _mm256_cmpeq_epi8(_mm256_and_si256(a, mask), zero);
            let valid_b = _mm256_cmpeq_epi8(_mm256_and_si256(b, mask), zero);
            if _mm256_movemask_epi8(_mm256_and_si256(valid_a, valid_b)) != -1 {
                return false;
            }
            *word = _mm256_movemask_epi8(_mm256_cmpeq_epi8(a, one)) as u32 as u64
                | ((_mm256_movemask_epi8(_mm256_cmpeq_epi8(b, one)) as u32 as u64) << 32);
        }
        true
    }

    #[target_feature(enable = "avx512bw")]
    pub unsafe fn parse_binary_avx512(bytes: &[u8], words: &mut [u64]) -> bool {
        let one = _mm512_set1_epi8(b'1' as i8);
        let mask = _mm512_set1_epi8(!1);
        let zero = _mm512_set1_epi8(b'0' as i8);
        for (chunk, word) in bytes.as_chunks::<64>().0.iter().zip(words) {
            let x = _mm512_loadu_si512(chunk.as_ptr().cast());
            if _mm512_cmpeq_epi8_mask(_mm512_and_si512(x, mask), zero) != u64::MAX {
                return false;
            }
            *word = _mm512_cmpeq_epi8_mask(x, one);
        }
        true
    }

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
        let mut i = 0;
        while i + 16 <= bits.len() * 2 {
            // Sixteen vectors contribute at most 128 set bits to each byte.
            let mut counts = zero;
            for offset in 0..16 {
                let value = _mm256_load_si256(ptr.add(i + offset));
                let low = _mm256_shuffle_epi8(table, _mm256_and_si256(value, low_mask));
                let high = _mm256_shuffle_epi8(
                    table,
                    _mm256_and_si256(_mm256_srli_epi16::<4>(value), low_mask),
                );
                counts = _mm256_add_epi8(counts, _mm256_add_epi8(low, high));
            }
            sum = _mm256_add_epi64(sum, _mm256_sad_epu8(counts, zero));
            i += 16;
        }
        while i < bits.len() * 2 {
            let value = _mm256_load_si256(ptr.add(i));
            let low = _mm256_shuffle_epi8(table, _mm256_and_si256(value, low_mask));
            let high = _mm256_shuffle_epi8(
                table,
                _mm256_and_si256(_mm256_srli_epi16::<4>(value), low_mask),
            );
            sum = _mm256_add_epi64(sum, _mm256_sad_epu8(_mm256_add_epi8(low, high), zero));
            i += 1;
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
        for i in 0..bits.len() {
            let value = _mm256_and_si256(
                _mm256_load_si256(ptr.add(i * 2)),
                _mm256_load_si256(ptr.add(i * 2 + 1)),
            );
            if _mm256_movemask_epi8(_mm256_cmpeq_epi64(value, ones)) != -1 {
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
    use crate::tools::testutil::sample_usize;
    use std::{
        mem::size_of,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    #[test]
    fn test_binary_conversion() {
        let mut rng = Xorshift::new_with_seed(41513);
        for case in 0..512 {
            let size = match case % 3 {
                0 => rng.random(0..32),
                1 => rng.random(0..512),
                _ => rng.random(0..20000),
            };
            let density = rng.rand(1010);
            let model: Vec<bool> = (0..size).map(|_| rng.rand(1009) < density).collect();
            let s: String = model.iter().map(|&b| if b { '1' } else { '0' }).collect();
            let expected: BitSet = model.iter().copied().collect();
            let parsed = BitSet::from_binary(&s).unwrap();
            assert_model(&parsed, &model);
            assert_eq!(parsed, expected);
            assert_eq!(expected.to_binary(), s);
            let words: Vec<u64> = model
                .chunks(64)
                .map(|chunk| {
                    chunk
                        .iter()
                        .enumerate()
                        .fold(0, |w, (i, &b)| w | (u64::from(b) << i))
                })
                .collect();
            assert_eq!(expected.words(), words);
            let mut packed = BitSet::new(size);
            packed.words_mut().copy_from_slice(&words);
            assert_model(&packed, &model);

            let mut modified = s.as_bytes().to_vec();
            if size != 0 {
                for _ in 0..rng.random(1..=size.min(16)) {
                    modified[rng.random(0..size)] = rng.random(0..128);
                }
            }
            let valid = modified.iter().all(|b| matches!(b, b'0' | b'1'));
            let modified = String::from_utf8(modified).unwrap();
            assert_eq!(BitSet::from_binary(&modified).is_some(), valid);
            let mut unicode = s.clone();
            unicode.insert(
                rng.random(0..=size),
                char::from_u32(rng.random(0xe000..0x110000)).unwrap(),
            );
            assert!(BitSet::from_binary(&unicode).is_none());

            let end = size / 64 * 64;
            for invalid in [false, true] {
                let mut input = s.as_bytes()[..end].to_vec();
                if invalid && end != 0 {
                    for _ in 0..rng.random(1..=end.min(16)) {
                        input[rng.random(0..end)] = rng.random(0..=255);
                    }
                }
                let valid = input.iter().all(|b| matches!(b, b'0' | b'1'));
                let mut scalar = BitSet::new(end);
                assert_eq!(
                    BitSet::parse_binary_scalar(&input, scalar.words_mut()),
                    valid
                );
                if valid {
                    assert_model(
                        &scalar,
                        &input.iter().map(|&b| b == b'1').collect::<Vec<_>>(),
                    );
                }
                #[cfg(target_arch = "x86_64")]
                {
                    if is_x86_feature_detected!("avx2") {
                        let mut parsed = BitSet::new(end);
                        // SAFETY: AVX2 support was checked; input has full 64-byte chunks.
                        assert_eq!(
                            unsafe { simd::parse_binary_avx2(&input, parsed.words_mut()) },
                            valid
                        );
                        if valid {
                            assert_eq!(parsed, scalar);
                            let mut output = vec![0; end];
                            // SAFETY: AVX2 support was checked; output has full 64-byte chunks.
                            unsafe { simd::write_binary_avx2(&mut output, parsed.words()) };
                            assert_eq!(output, input);
                        }
                    }
                    if is_x86_feature_detected!("avx512bw") {
                        let mut parsed = BitSet::new(end);
                        // SAFETY: AVX-512BW support was checked; input has full 64-byte chunks.
                        assert_eq!(
                            unsafe { simd::parse_binary_avx512(&input, parsed.words_mut()) },
                            valid
                        );
                        if valid {
                            assert_eq!(parsed, scalar);
                            let mut output = vec![0; end];
                            // SAFETY: AVX-512BW support was checked; output has full chunks.
                            unsafe { simd::write_binary_avx512(&mut output, parsed.words()) };
                            assert_eq!(output, input);
                        }
                    }
                }
            }
        }
    }

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

    fn bitset_sizes(rng: &mut Xorshift) -> Vec<usize> {
        let block_bits = size_of::<Block>() * u8::BITS as usize;
        let max_size = 16 * block_bits + 1;
        let mut sizes = sample_usize(rng, u64::BITS as usize, 0..=max_size, 30);
        // Cover every block boundary, including both sides of SIMD dispatch.
        sizes.extend(
            (0..max_size)
                .step_by(block_bits)
                .flat_map(|boundary| boundary.saturating_sub(1)..=boundary + 1),
        );
        sizes.sort_unstable();
        sizes.dedup();
        sizes
    }

    #[test]
    fn access_fill_reset_push_extend_resize() {
        assert_eq!(size_of::<Block>(), 64);
        assert_eq!(std::mem::align_of::<Block>(), 64);
        let aligned = BitSet::new(1);
        assert_eq!(aligned.bits.as_ptr() as usize & 63, 0);

        let mut rng = Xorshift::default();
        for size in bitset_sizes(&mut rng) {
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

            let extra_len = rng.random(64..=128);
            let extra = random_model(&mut rng, extra_len);
            actual.extend(extra.iter().copied());
            let mut extended = model.clone();
            extended.extend(extra);
            assert_model(&actual, &extended);

            actual.resize(size / 2);
            assert_model(&actual, &model[..size / 2]);
            actual.resize(size + extra_len);
            let mut resized = model[..size / 2].to_vec();
            resized.resize(size + extra_len, false);
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
        for size in bitset_sizes(&mut rng) {
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
    fn shifts_match_boolean_model() {
        let mut rng = Xorshift::default();
        for size in bitset_sizes(&mut rng) {
            let model = random_model(&mut rng, size);
            for shift in sample_usize(&mut rng, 16, 0..=size + 512, 10)
                .into_iter()
                .chain([size, size + 1])
            {
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
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let size = rng.random(0..=1024);
            let other_size = size + rng.random(1..=1024usize);
            let mut lhs = bitset(&random_model(&mut rng, size));
            let rhs = bitset(&random_model(&mut rng, other_size));
            let before = lhs.clone();
            let op = rng.random(0..3);
            let result = catch_unwind(AssertUnwindSafe(|| match op {
                0 => lhs &= &rhs,
                1 => lhs |= &rhs,
                _ => lhs ^= &rhs,
            }));
            assert!(result.is_err());
            assert_eq!(lhs, before);
        }
    }
}
