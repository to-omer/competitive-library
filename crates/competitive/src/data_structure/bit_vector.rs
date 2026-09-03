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

/// An append-only rank/select dictionary with 256-word prefix blocks.
///
/// The layout reduces metadata and improves large or select-heavy workloads. A compact
/// rank-only workload can be faster with an absolute prefix stored beside every word.
#[derive(Debug, Clone)]
pub struct BitVector {
    words: Vec<u64>,
    super_prefix: Vec<usize>,
    sub_prefix: Vec<u16>,
    sum: usize,
    len: usize,
    #[cfg(target_arch = "x86_64")]
    bmi2: bool,
}
impl BitVector {
    const WORD_SIZE: usize = u64::BITS as usize;
    const SUPER_WORDS: usize = 256;

    pub fn with_capacity(bits: usize) -> Self {
        let words = bits.div_ceil(Self::WORD_SIZE);
        let mut word_values = Vec::with_capacity(words + 1);
        word_values.push(0);
        let mut super_prefix = Vec::with_capacity(words / Self::SUPER_WORDS + 1);
        super_prefix.push(0);
        let mut sub_prefix = Vec::with_capacity(words + 1);
        sub_prefix.push(0);
        Self {
            words: word_values,
            super_prefix,
            sub_prefix,
            sum: 0,
            len: 0,
            #[cfg(target_arch = "x86_64")]
            bmi2: is_x86_feature_detected!("bmi2"),
        }
    }

    pub fn push(&mut self, bit: bool) {
        let word = self.len / Self::WORD_SIZE;
        let offset = self.len % Self::WORD_SIZE;
        if offset == 0 {
            self.words.push(0);
            self.sub_prefix.push(0);
        }
        if bit {
            self.words[word] |= 1u64 << offset;
            self.sum += 1;
        }
        self.len += 1;
        if self.len.is_multiple_of(Self::WORD_SIZE) {
            let word = self.len / Self::WORD_SIZE;
            if word.is_multiple_of(Self::SUPER_WORDS) {
                if let Some(prefix) = self.super_prefix.get_mut(word / Self::SUPER_WORDS) {
                    *prefix = self.sum;
                } else {
                    self.super_prefix.push(self.sum);
                }
            }
            self.sub_prefix[word] = (self.sum - self.super_prefix[word / Self::SUPER_WORDS]) as u16;
        }
    }

    fn from_words(mut words: Vec<u64>, len: usize) -> Self {
        let mut super_prefix = Vec::with_capacity(words.len().div_ceil(Self::SUPER_WORDS));
        let mut sub_prefix = Vec::with_capacity(words.len() + 1);
        let mut sum = 0;
        let mut super_sum = 0;
        for index in 0..=words.len() {
            if index.is_multiple_of(Self::SUPER_WORDS) {
                if index < words.len() || len.is_multiple_of(Self::WORD_SIZE) {
                    super_sum = sum;
                    super_prefix.push(sum);
                }
                sub_prefix.push(0);
            } else {
                sub_prefix.push((sum - super_sum) as u16);
            }
            if let Some(&word) = words.get(index) {
                sum += word.count_ones() as usize;
            }
        }
        words.push(0);
        Self {
            words,
            super_prefix,
            sub_prefix,
            sum,
            len,
            #[cfg(target_arch = "x86_64")]
            bmi2: is_x86_feature_detected!("bmi2"),
        }
    }

    #[inline]
    fn select_word(&self, word: u64, rank: usize) -> usize {
        #[cfg(target_arch = "x86_64")]
        if self.bmi2 {
            // SAFETY: support for BMI2 was cached during construction.
            return unsafe { simd::select_word(word, rank) };
        }
        select_word_scalar(word, rank)
    }

    #[inline]
    fn locate_one(&self, mut rank: usize) -> (usize, usize) {
        let block = self.super_prefix.partition_point(|&sum| sum <= rank) - 1;
        rank -= self.super_prefix[block];
        let word_start = block * Self::SUPER_WORDS;
        let word_end = (word_start + Self::SUPER_WORDS).min(self.words.len() - 1);
        let lane =
            self.sub_prefix[word_start..word_end].partition_point(|&sum| sum as usize <= rank) - 1;
        let word = word_start + lane;
        rank -= self.sub_prefix[word] as usize;
        (word, rank)
    }

    #[inline]
    fn locate_zero(&self, mut rank: usize) -> (usize, usize) {
        let mut block = 0;
        let mut right = self.super_prefix.len();
        while right - block > 1 {
            let middle = block.midpoint(right);
            if middle * Self::SUPER_WORDS * Self::WORD_SIZE - self.super_prefix[middle] <= rank {
                block = middle;
            } else {
                right = middle;
            }
        }
        rank -= block * Self::SUPER_WORDS * Self::WORD_SIZE - self.super_prefix[block];
        let word_start = block * Self::SUPER_WORDS;
        let word_end = (word_start + Self::SUPER_WORDS).min(self.words.len() - 1);
        let mut word = word_start;
        let mut right = word_end;
        while right - word > 1 {
            let middle = word.midpoint(right);
            let zeros = (middle - word_start) * Self::WORD_SIZE - self.sub_prefix[middle] as usize;
            if zeros <= rank {
                word = middle;
            } else {
                right = middle;
            }
        }
        rank -= (word - word_start) * Self::WORD_SIZE - self.sub_prefix[word] as usize;
        (word, rank)
    }
}
impl RankSelectDictionaries for BitVector {
    fn bit_length(&self) -> usize {
        self.len
    }

    #[inline]
    fn access(&self, k: usize) -> bool {
        debug_assert!(k < self.len);
        self.words[k / Self::WORD_SIZE] & (1u64 << (k % Self::WORD_SIZE)) != 0
    }
    fn access_rank1(&self, k: usize) -> (bool, usize) {
        debug_assert!(k <= self.len);
        let word = k / Self::WORD_SIZE;
        let offset = k % Self::WORD_SIZE;
        let bits = self.words[word];
        (
            bits & (1u64 << offset) != 0,
            self.super_prefix[word / Self::SUPER_WORDS]
                + self.sub_prefix[word] as usize
                + (bits & !(u64::MAX << offset)).count_ones() as usize,
        )
    }
    fn rank1(&self, k: usize) -> usize {
        self.access_rank1(k).1
    }
    fn select1(&self, k: usize) -> Option<usize> {
        if self.sum <= k {
            return None;
        }
        let (word, rank) = self.locate_one(k);
        Some(word * Self::WORD_SIZE + self.select_word(self.words[word], rank))
    }
    fn select0(&self, k: usize) -> Option<usize> {
        if self.len - self.sum <= k {
            return None;
        }
        let (word, rank) = self.locate_zero(k);
        let mut bits = !self.words[word];
        if word + 1 == self.words.len() - 1 && !self.len.is_multiple_of(Self::WORD_SIZE) {
            bits &= u64::MAX >> (Self::WORD_SIZE - self.len % Self::WORD_SIZE);
        }
        Some(word * Self::WORD_SIZE + self.select_word(bits, rank))
    }
}
impl FromIterator<bool> for BitVector {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let capacity = match upper {
            Some(upper) => upper,
            None => lower,
        };
        let mut words = Vec::with_capacity(capacity.div_ceil(Self::WORD_SIZE) + 1);
        let mut word = 0u64;
        let mut word_len = 0;
        let mut len = 0;
        for bit in iter {
            word |= (bit as u64) << word_len;
            word_len += 1;
            len += 1;
            if word_len == Self::WORD_SIZE {
                words.push(word);
                word = 0;
                word_len = 0;
            }
        }
        if word_len != 0 {
            words.push(word);
        }
        Self::from_words(words, len)
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
            let word = x;
            for k in 0..=WORD_SIZE {
                assert_eq!(x.rank1(k), (0..k).filter(|&i| x.access(i)).count());
                assert_eq!(x.rank0(k), (0..k).filter(|&i| !x.access(i)).count());
                if k < word.count_ones() as usize {
                    assert_eq!(select_word_scalar(word, k), word.select1(k).unwrap());
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
        let mut rng = Xorshift::default();
        for len in [
            0,
            1,
            BitVector::WORD_SIZE - 1,
            BitVector::WORD_SIZE,
            BitVector::WORD_SIZE + 1,
            BitVector::WORD_SIZE * BitVector::SUPER_WORDS - 1,
            BitVector::WORD_SIZE * BitVector::SUPER_WORDS,
            BitVector::WORD_SIZE * BitVector::SUPER_WORDS + 1,
            BitVector::WORD_SIZE * (BitVector::SUPER_WORDS * 2 + 17),
        ] {
            for pattern in 0..3 {
                let bits: Vec<_> = (0..len)
                    .map(|index| match pattern {
                        0 => rng.rand(5) != 0,
                        1 => index.is_multiple_of(BitVector::WORD_SIZE * 3 + 1),
                        _ => !index.is_multiple_of(BitVector::WORD_SIZE * 3 + 1),
                    })
                    .collect();
                let mut pushed = BitVector::with_capacity(len);
                for &bit in &bits {
                    pushed.push(bit);
                }
                let collected: BitVector = bits.iter().copied().collect();
                let split = len / 2;
                let mut extended: BitVector = bits[..split].iter().copied().collect();
                for &bit in &bits[split..] {
                    extended.push(bit);
                }
                let split = len.saturating_sub(1);
                let mut appended: BitVector = bits[..split].iter().copied().collect();
                for &bit in &bits[split..] {
                    appended.push(bit);
                }
                for actual in [pushed, collected, extended, appended] {
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
                    let ones: Vec<_> = bits
                        .iter()
                        .enumerate()
                        .filter_map(|(index, &bit)| bit.then_some(index))
                        .collect();
                    let zeros: Vec<_> = bits
                        .iter()
                        .enumerate()
                        .filter_map(|(index, &bit)| (!bit).then_some(index))
                        .collect();
                    for rank in 0..=ones.len() {
                        assert_eq!(actual.select1(rank), ones.get(rank).copied());
                    }
                    for rank in 0..=zeros.len() {
                        assert_eq!(actual.select0(rank), zeros.get(rank).copied());
                    }
                }
            }
        }
    }
}
