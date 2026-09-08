use std::iter::FromIterator;

/// rank_i(select_i(k)) = k
/// rank_i(select_i(k) + 1) = k + 1
pub trait RankSelectDictionaries {
    fn bit_length(&self) -> usize;
    /// get k-th bit
    fn access(&self, k: usize) -> bool;
    /// Returns the k-th bit and the number of ones before it.
    fn access_rank1(&self, k: usize) -> (bool, usize) {
        (self.access(k), self.rank1(k))
    }
    /// the number of 1 in [0, k)
    fn rank1(&self, k: usize) -> usize {
        (0..k).filter(|&i| self.access(i)).count()
    }
    /// the number of 0 in [0, k)
    fn rank0(&self, k: usize) -> usize {
        k - self.rank1(k)
    }
    /// index of k-th 1
    fn select1(&self, k: usize) -> Option<usize> {
        let n = self.bit_length();
        if self.rank1(n) <= k {
            return None;
        }
        let (mut l, mut r) = (0, n);
        while r - l > 1 {
            let m = l.midpoint(r);
            if self.rank1(m) <= k {
                l = m;
            } else {
                r = m;
            }
        }
        Some(l)
    }
    /// index of k-th 0
    fn select0(&self, k: usize) -> Option<usize> {
        let n = self.bit_length();
        if self.rank0(n) <= k {
            return None;
        }
        let (mut l, mut r) = (0, n);
        while r - l > 1 {
            let m = l.midpoint(r);
            if self.rank0(m) <= k {
                l = m;
            } else {
                r = m;
            }
        }
        Some(l)
    }
}

macro_rules! impl_rank_select_for_bits {
    ($($t:ty)*) => {$(
        impl RankSelectDictionaries for $t {
            fn bit_length(&self) -> usize {
                const WORD_SIZE: usize = (0 as $t).count_zeros() as usize;
                WORD_SIZE
            }
            fn access(&self, k: usize) -> bool {
                const WORD_SIZE: usize = (0 as $t).count_zeros() as usize;
                if k < WORD_SIZE {
                    self & (1 as $t) << k != 0
                } else {
                    false
                }
            }
            fn rank1(&self, k: usize) -> usize {
                const WORD_SIZE: usize = (0 as $t).count_zeros() as usize;
                if k < WORD_SIZE {
                    (self & !(!(0 as $t) << k)).count_ones() as usize
                } else {
                    self.count_ones() as usize
                }
            }
        }
    )*};
}

impl_rank_select_for_bits!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128);

#[inline]
fn select_word_scalar(mut word: u64, mut rank: usize) -> usize {
    let count = word.count_ones() as usize;
    debug_assert!(rank < count);
    if rank < 4 {
        for _ in 0..rank {
            word &= word - 1;
        }
        return word.trailing_zeros() as usize;
    }
    if count - rank <= 4 {
        for _ in 0..count - rank - 1 {
            word &= !(1 << (u64::BITS as usize - 1 - word.leading_zeros() as usize));
        }
        return u64::BITS as usize - 1 - word.leading_zeros() as usize;
    }

    let mut offset = 0;
    let mut width = u64::BITS as usize / 2;
    while width != 0 {
        let mask = u64::MAX >> (u64::BITS as usize - width);
        let count = (word & mask).count_ones() as usize;
        if rank < count {
            word &= mask;
        } else {
            word >>= width;
            rank -= count;
            offset += width;
        }
        width /= 2;
    }
    offset
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)] // BMI2 is confined to a feature-gated function.
mod simd {
    use std::arch::x86_64::_pdep_u64;

    #[target_feature(enable = "bmi2")]
    #[inline]
    pub unsafe fn select_word(word: u64, rank: usize) -> usize {
        _pdep_u64(1 << rank, word).trailing_zeros() as usize
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct BitVectorBlock {
    pub bits: u64,
    pub rank: usize,
}

#[derive(Debug, Clone)]
pub struct BitVector {
    blocks: Vec<BitVectorBlock>,
    len: usize,
    sum: usize,
    select_samples: [Vec<usize>; 2],
}

impl BitVector {
    const WORD_SIZE: usize = u64::BITS as usize;

    /// Builds a bit vector from low-bit-first words. `words.len()` must equal
    /// `len.div_ceil(64)`. Unused high bits in the final word are ignored.
    pub fn from_words(words: &[u64], len: usize) -> Self {
        assert_eq!(words.len(), len.div_ceil(Self::WORD_SIZE));
        let mut sum = 0;
        let mut blocks = Vec::with_capacity(len / Self::WORD_SIZE + 1);
        for (i, &bits) in words.iter().enumerate() {
            let count = (len - i * Self::WORD_SIZE).min(Self::WORD_SIZE);
            let bits = if count == Self::WORD_SIZE {
                bits
            } else {
                bits & ((1u64 << count) - 1)
            };
            blocks.push(BitVectorBlock { bits, rank: sum });
            sum += bits.count_ones() as usize;
        }
        if len.is_multiple_of(Self::WORD_SIZE) {
            blocks.push(BitVectorBlock { bits: 0, rank: sum });
        }
        Self::from_blocks(blocks, len, sum)
    }

    fn from_blocks(blocks: Vec<BitVectorBlock>, len: usize, sum: usize) -> Self {
        let mut select_samples = [Vec::new(), Vec::new()];
        for (i, block) in blocks
            .iter()
            .enumerate()
            .take(len.div_ceil(Self::WORD_SIZE))
        {
            let start = [i * Self::WORD_SIZE - block.rank, block.rank];
            let end1 = blocks.get(i + 1).map_or(sum, |next| next.rank);
            let end = [((i + 1) * Self::WORD_SIZE).min(len) - end1, end1];
            for bit in 0..2 {
                if start[bit].div_ceil(256) != end[bit].div_ceil(256) {
                    select_samples[bit].push(i);
                }
            }
        }
        Self {
            blocks,
            len,
            sum,
            select_samples,
        }
    }

    pub fn with_capacity(bits: usize) -> Self {
        let mut blocks = Vec::with_capacity(bits.div_ceil(Self::WORD_SIZE) + 1);
        blocks.push(BitVectorBlock { bits: 0, rank: 0 });
        Self {
            blocks,
            len: 0,
            sum: 0,
            select_samples: [Vec::new(), Vec::new()],
        }
    }

    pub fn push(&mut self, bit: bool) {
        let word = self.len / Self::WORD_SIZE;
        let rank = if bit { self.sum } else { self.len - self.sum };
        if rank.is_multiple_of(256) {
            self.select_samples[bit as usize].push(word);
        }
        self.blocks[word].bits |= (bit as u64) << (self.len % Self::WORD_SIZE);
        self.sum += bit as usize;
        self.len += 1;
        if self.len.is_multiple_of(Self::WORD_SIZE) {
            self.blocks.push(BitVectorBlock {
                bits: 0,
                rank: self.sum,
            });
        }
    }

    /// Words paired with the number of ones preceding each word. The last block
    /// is partial, or an empty sentinel when the bit length is a multiple of 64.
    pub fn blocks(&self) -> &[BitVectorBlock] {
        &self.blocks
    }

    /// Returns the position of the zero-based occurrence `rank` in `bits`.
    /// `rank` must be less than the number of set bits.
    #[inline]
    pub fn select_word(bits: u64, rank: usize) -> usize {
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("bmi2") {
            // SAFETY: BMI2 is available and the caller checked the occurrence count.
            return unsafe { simd::select_word(bits, rank) };
        }
        select_word_scalar(bits, rank)
    }
}

impl RankSelectDictionaries for BitVector {
    fn bit_length(&self) -> usize {
        self.len
    }

    #[inline]
    fn access(&self, k: usize) -> bool {
        self.blocks[k / Self::WORD_SIZE].bits & (1u64 << (k % Self::WORD_SIZE)) != 0
    }

    #[inline]
    fn access_rank1(&self, k: usize) -> (bool, usize) {
        let block = &self.blocks[k / Self::WORD_SIZE];
        let offset = k % Self::WORD_SIZE;
        (
            block.bits & (1u64 << offset) != 0,
            block.rank + (block.bits & !(u64::MAX << offset)).count_ones() as usize,
        )
    }

    #[inline]
    fn rank1(&self, k: usize) -> usize {
        self.access_rank1(k).1
    }

    fn select1(&self, k: usize) -> Option<usize> {
        if k >= self.sum {
            return None;
        }
        let sample = k / 256;
        let start = self.select_samples[1][sample];
        let end = self.select_samples[1]
            .get(sample + 1)
            .map_or(self.blocks.len(), |&word| word + 1);
        let word = start + self.blocks[start..end].partition_point(|block| block.rank <= k) - 1;
        let rank = k - self.blocks[word].rank;
        Some(word * Self::WORD_SIZE + Self::select_word(self.blocks[word].bits, rank))
    }

    fn select0(&self, k: usize) -> Option<usize> {
        if k >= self.len - self.sum {
            return None;
        }
        let sample = k / 256;
        let mut word = self.select_samples[0][sample];
        let end = self.select_samples[0]
            .get(sample + 1)
            .map_or(self.blocks.len(), |&word| word + 1);
        let mut size = end - word;
        while size > 1 {
            let half = size / 2;
            let middle = word + half;
            word = if middle * Self::WORD_SIZE - self.blocks[middle].rank <= k {
                middle
            } else {
                word
            };
            size -= half;
        }
        let rank = k - (word * Self::WORD_SIZE - self.blocks[word].rank);
        Some(word * Self::WORD_SIZE + Self::select_word(!self.blocks[word].bits, rank))
    }
}

impl FromIterator<bool> for BitVector {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut blocks = Vec::with_capacity(iter.size_hint().0 / Self::WORD_SIZE + 1);
        let mut len = 0usize;
        let mut sum = 0;
        let mut iter = iter.fuse();
        while let Some(first) = iter.next() {
            let mut bits = first as u64;
            let mut count = 1;
            for (i, bit) in iter.by_ref().take(Self::WORD_SIZE - 1).enumerate() {
                bits |= (bit as u64) << (i + 1);
                count += 1;
            }
            blocks.push(BitVectorBlock { bits, rank: sum });
            len += count;
            sum += bits.count_ones() as usize;
        }
        if len.is_multiple_of(Self::WORD_SIZE) {
            blocks.push(BitVectorBlock { bits: 0, rank: sum });
        }
        Self::from_blocks(blocks, len, sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    const Q: usize = 5_000;

    #[test]
    fn test_rank_select_word() {
        const WORD_SIZE: usize = u64::BITS as usize;
        let mut rng = Xorshift::default();
        for x in rng.random_iter(0u64..).take(Q) {
            for k in 0..=WORD_SIZE {
                assert_eq!(x.rank1(k), (0..k).filter(|&i| x.access(i)).count());
                assert_eq!(x.rank0(k), (0..k).filter(|&i| !x.access(i)).count());
                if k < x.count_ones() as usize {
                    assert_eq!(select_word_scalar(x, k), x.select1(k).unwrap());
                }
                if let Some(i) = x.select1(k) {
                    assert_eq!((0..i).filter(|&j| x.access(j)).count(), k);
                    assert!(x.access(i));
                } else {
                    assert!(x.rank1(WORD_SIZE) <= k);
                }
                if let Some(i) = x.select0(k) {
                    assert_eq!((0..i).filter(|&j| !x.access(j)).count(), k);
                    assert!(!x.access(i));
                } else {
                    assert!(x.rank0(WORD_SIZE) <= k);
                }
            }
        }
    }

    #[test]
    fn test_rank_select_bit_vector() {
        let mut events = [Some(true), None, Some(false)].into_iter();
        let collected: BitVector = std::iter::from_fn(|| events.next().flatten()).collect();
        assert_eq!(collected.bit_length(), 1);
        assert_eq!(collected.rank1(1), 1);
        let mut rng = Xorshift::default();
        for len in [
            0,
            1,
            BitVector::WORD_SIZE - 1,
            BitVector::WORD_SIZE,
            BitVector::WORD_SIZE + 1,
            16384 - 1,
            16384,
            16384 + 1,
            65537,
        ] {
            for pattern in 0..7 {
                let bits: Vec<_> = (0..len)
                    .map(|index| match pattern {
                        0 => rng.rand(5) != 0,
                        1 => index.is_multiple_of(BitVector::WORD_SIZE * 3 + 1),
                        2 => !index.is_multiple_of(BitVector::WORD_SIZE * 3 + 1),
                        3 => false,
                        4 => true,
                        5 => index % 8193 == 8192,
                        _ => index % 8193 != 8192,
                    })
                    .collect();
                let mut words = vec![u64::MAX; len.div_ceil(64)];
                let mut positions = [Vec::new(), Vec::new()];
                for (i, &bit) in bits.iter().enumerate() {
                    positions[bit as usize].push(i);
                    if !bit {
                        words[i / 64] &= !(1 << (i % 64));
                    }
                }
                for split in [0, len / 2, len.saturating_sub(1), len] {
                    let mut pushed = BitVector::with_capacity(len);
                    for &bit in &bits[..split] {
                        pushed.push(bit);
                    }
                    let collected: BitVector = bits[..split].iter().copied().collect();
                    let packed = BitVector::from_words(&words[..split.div_ceil(64)], split);
                    for mut actual in [pushed, collected, packed] {
                        for &bit in &bits[split..] {
                            actual.push(bit);
                        }
                        let mut rank1 = 0;
                        for (index, &bit) in bits.iter().enumerate() {
                            assert_eq!(actual.access(index), bit);
                            assert_eq!(actual.access_rank1(index), (bit, rank1));
                            rank1 += bit as usize;
                        }
                        for end in [0, len / 3, len / 2, len] {
                            assert_eq!(
                                actual.rank1(end),
                                bits[..end].iter().filter(|&&bit| bit).count()
                            );
                            assert_eq!(
                                actual.rank0(end),
                                bits[..end].iter().filter(|&&bit| !bit).count()
                            );
                        }
                        for rank in 0..=positions[1].len() {
                            assert_eq!(actual.select1(rank), positions[1].get(rank).copied());
                        }
                        for rank in 0..=positions[0].len() {
                            assert_eq!(actual.select0(rank), positions[0].get(rank).copied());
                        }
                    }
                }
            }
        }
    }
}
