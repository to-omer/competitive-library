use std::iter::FromIterator;

/// rank_i(select_i(k)) = k
/// rank_i(select_i(k) + 1) = k + 1
pub trait RankSelectDictionaries {
    fn bit_length(&self) -> usize;
    /// get k-th bit
    fn access(&self, k: usize) -> bool;
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
        })*
    };
}
impl_rank_select_for_bits!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128);

#[derive(Debug, Clone)]
pub struct BitVector {
    /// [(bit, sum)]
    data: Vec<(usize, usize)>,
    sum: usize,
    len: usize,
}
impl BitVector {
    const WORD_SIZE: usize = 0usize.count_zeros() as usize;

    pub fn with_capacity(bits: usize) -> Self {
        let words = bits.div_ceil(Self::WORD_SIZE) + 1;
        let mut data = Vec::with_capacity(words);
        data.push((0, 0));
        Self {
            data,
            sum: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, bit: bool) {
        let word = self.len / Self::WORD_SIZE;
        let offset = self.len % Self::WORD_SIZE;
        if word == self.data.len() - 1 {
            self.data.push((0, self.sum));
        }
        if bit {
            self.data[word].0 |= 1 << offset;
            self.sum += 1;
        }
        self.len += 1;
        self.data.last_mut().unwrap().1 = self.sum;
    }
}
impl RankSelectDictionaries for BitVector {
    fn bit_length(&self) -> usize {
        self.len
    }
    fn access(&self, k: usize) -> bool {
        debug_assert!(k < self.len);
        self.data[k / Self::WORD_SIZE].0 & (1 << (k % Self::WORD_SIZE)) != 0
    }
    fn rank1(&self, k: usize) -> usize {
        debug_assert!(k <= self.len);
        let (bit, sum) = self.data[k / Self::WORD_SIZE];
        sum + (bit & !(usize::MAX << (k % Self::WORD_SIZE))).count_ones() as usize
    }
    fn select1(&self, mut k: usize) -> Option<usize> {
        let (mut l, mut r) = (0, self.data.len());
        if self.sum <= k {
            return None;
        }
        while r - l > 1 {
            let m = l.midpoint(r);
            if self.data[m].1 <= k {
                l = m;
            } else {
                r = m;
            }
        }
        let (bit, sum) = self.data[l];
        k -= sum;
        Some(l * Self::WORD_SIZE + bit.select1(k).unwrap())
    }
    fn select0(&self, mut k: usize) -> Option<usize> {
        let (mut l, mut r) = (0, self.data.len());
        if self.len - self.sum <= k {
            return None;
        }
        while r - l > 1 {
            let m = l.midpoint(r);
            if m * Self::WORD_SIZE - self.data[m].1 <= k {
                l = m;
            } else {
                r = m;
            }
        }
        let (bit, sum) = self.data[l];
        k -= l * Self::WORD_SIZE - sum;
        Some(l * Self::WORD_SIZE + bit.select0(k).unwrap())
    }
}
impl FromIterator<bool> for BitVector {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let mut bit_vector = match upper {
            Some(upper) => Self::with_capacity(upper),
            None => Self::with_capacity(lower),
        };
        for b in iter {
            bit_vector.push(b);
        }
        bit_vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    const Q: usize = 5_000;

    #[test]
    fn test_rank_select_usize() {
        const WORD_SIZE: usize = 0usize.count_zeros() as usize;
        let mut rng = Xorshift::default();
        for x in rng.random_iter(0u64..).take(Q) {
            for k in 0..=WORD_SIZE {
                assert_eq!(x.rank1(k), (0..k).filter(|&i| x.access(i)).count());
                assert_eq!(x.rank0(k), (0..k).filter(|&i| !x.access(i)).count());
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
        const N: usize = 1_000;
        let mut rng = Xorshift::default();
        let x: BitVector = (0..N).map(|_| rng.rand(2) != 0).collect();
        for k in rng.random_iter(..=N).take(Q) {
            assert_eq!(x.rank1(k), (0..k).filter(|&i| x.access(i)).count());
            assert_eq!(x.rank0(k), (0..k).filter(|&i| !x.access(i)).count());

            if let Some(i) = x.select1(k) {
                assert_eq!((0..i).filter(|&j| x.access(j)).count(), k);
                assert!(x.access(i));
            } else {
                assert!(x.rank1(N) <= k);
            }

            if let Some(i) = x.select0(k) {
                assert_eq!((0..i).filter(|&j| !x.access(j)).count(), k);
                assert!(!x.access(i));
            } else {
                assert!(x.rank0(N) <= k);
            }
        }
        assert_eq!(x.rank1(0), 0);
        assert_eq!(x.rank0(0), 0);
    }
}
