use super::{BitSet, BitVector, DaryPrefixSumTreeU32, RankSelectDictionaries};

/// A fixed-length mutable bitset with a prefix-count index over its words.
///
/// Updates, rank, and select1 take logarithmic time. The default select0 uses
/// binary search over rank. The bit length must fit in u32.
#[derive(Clone, Debug)]
pub struct RankSelectBitSet {
    bits: BitSet,
    counts: DaryPrefixSumTreeU32,
}

impl RankSelectBitSet {
    pub fn new(len: usize) -> Self {
        assert!(u32::try_from(len).is_ok());
        Self {
            bits: BitSet::new(len),
            counts: DaryPrefixSumTreeU32::new(len.div_ceil(64)),
        }
    }

    /// Builds from low-bit-first words, ignoring unused high bits of the last word.
    pub fn from_words(words: &[u64], len: usize) -> Self {
        assert!(u32::try_from(len).is_ok());
        assert_eq!(words.len(), len.div_ceil(64));
        let mut bits = BitSet::new(len);
        bits.words_mut()[..words.len()].copy_from_slice(words);
        if !len.is_multiple_of(64) {
            bits.words_mut()[len / 64] &= (1 << (len % 64)) - 1;
        }
        let counts: Vec<_> = bits.words()[..words.len()]
            .iter()
            .map(|word| word.count_ones())
            .collect();
        Self {
            bits,
            counts: DaryPrefixSumTreeU32::from_slice(&counts),
        }
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < self.len());
        let word = index / 64;
        let bits = &mut self.bits.words_mut()[word];
        let mask = 1u64 << (index % 64);
        if (*bits & mask != 0) != value {
            *bits ^= mask;
            self.counts.set(word, bits.count_ones());
        }
    }

    pub fn count_ones(&self) -> u64 {
        self.counts.fold_all() as u64
    }

    pub fn count_zeros(&self) -> u64 {
        self.len() as u64 - self.count_ones()
    }

    /// Returns the first set bit in `start..self.len()`.
    pub fn next_one(&self, start: usize) -> Option<usize> {
        assert!(start <= self.len());
        if start == self.len() {
            return None;
        }
        let word = start / 64;
        let bits = self.bits.words()[word] & (u64::MAX << (start % 64));
        if bits != 0 {
            return Some(word * 64 + bits.trailing_zeros() as usize);
        }
        let count = self.counts.accumulate0(word + 1);
        let word = self.counts.partition_point_acc(count);
        (word < self.counts.len())
            .then(|| word * 64 + self.bits.words()[word].trailing_zeros() as usize)
    }

    /// Returns the last set bit in `0..end`.
    pub fn prev_one(&self, end: usize) -> Option<usize> {
        assert!(end <= self.len());
        if end == 0 {
            return None;
        }
        let word = (end - 1) / 64;
        let bits = self.bits.words()[word] & (u64::MAX >> (63 - (end - 1) % 64));
        if bits != 0 {
            return Some(word * 64 + 63 - bits.leading_zeros() as usize);
        }
        let count = self.counts.accumulate0(word);
        if count == 0 {
            return None;
        }
        let word = self.counts.partition_point_acc(count - 1);
        Some(word * 64 + 63 - self.bits.words()[word].leading_zeros() as usize)
    }
}

impl RankSelectDictionaries for RankSelectBitSet {
    fn bit_length(&self) -> usize {
        self.len()
    }

    fn access(&self, index: usize) -> bool {
        assert!(index < self.len());
        self.bits.get(index)
    }

    fn rank1(&self, end: usize) -> usize {
        assert!(end <= self.len());
        let word = end / 64;
        let mut count = self.counts.accumulate0(word) as usize;
        if !end.is_multiple_of(64) {
            count += self.bits.words()[word].rank1(end % 64);
        }
        count
    }

    fn select1(&self, rank: usize) -> Option<usize> {
        if rank >= self.counts.fold_all() as usize {
            return None;
        }
        let (word, prefix) = self.counts.partition_point_acc_with_sum(rank as u32);
        let local = rank - prefix as usize;
        Some(word * 64 + BitVector::select_word(self.bits.words()[word], local))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Xorshift, testutil::sequence_cases};

    #[test]
    fn test_rank_select_bitset() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, 0u8..=1, 0..=4097).collect();
        for mut expected in cases {
            let len = expected.len();
            let mut words = vec![u64::MAX; len.div_ceil(64)];
            for (index, &bit) in expected.iter().enumerate() {
                if bit == 0 {
                    words[index / 64] &= !(1 << (index % 64));
                }
            }
            let mut actual = RankSelectBitSet::from_words(&words, len);
            for step in 0..160 {
                if len != 0 && step != 0 {
                    let index = match step % 4 {
                        0 => 0,
                        1 => len - 1,
                        _ => rng.random(0..len),
                    };
                    let value = rng.random(0u8..=1);
                    actual.set(index, value != 0);
                    actual.set(index, value != 0);
                    expected[index] = value;
                    assert_eq!(actual.access(index), value != 0);
                }
                assert_eq!(actual.bit_length(), len);
                assert_eq!(actual.is_empty(), len == 0);
                let ones: Vec<_> = (0..len).filter(|&i| expected[i] != 0).collect();
                let zeros: Vec<_> = (0..len).filter(|&i| expected[i] == 0).collect();
                assert_eq!(actual.count_ones(), ones.len() as u64);
                assert_eq!(actual.count_zeros(), zeros.len() as u64);
                for end in [0, len, rng.random(0..=len)] {
                    let rank = ones.partition_point(|&i| i < end);
                    assert_eq!(actual.rank1(end), rank);
                    assert_eq!(actual.rank0(end), zeros.partition_point(|&i| i < end));
                    assert_eq!(actual.next_one(end), ones.get(rank).copied());
                    assert_eq!(actual.prev_one(end), rank.checked_sub(1).map(|i| ones[i]));
                }
                for rank in [0, ones.len() / 2, ones.len(), usize::MAX] {
                    assert_eq!(actual.select1(rank), ones.get(rank).copied());
                }
                for rank in [0, zeros.len() / 2, zeros.len(), usize::MAX] {
                    assert_eq!(actual.select0(rank), zeros.get(rank).copied());
                }
            }
            let empty = RankSelectBitSet::new(len);
            assert_eq!(empty.rank1(len), 0);
            assert_eq!(empty.select1(0), None);
        }
    }
}
