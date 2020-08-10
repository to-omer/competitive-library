#[cargo_snippet::snippet("RankSelectDictionaries")]
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
            let m = (l + r) / 2;
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
            let m = (l + r) / 2;
            if self.rank0(m) <= k {
                l = m;
            } else {
                r = m;
            }
        }
        Some(l)
    }
}
#[cargo_snippet::snippet("RankSelectDictionaries")]
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
#[cargo_snippet::snippet("RankSelectDictionaries")]
impl_rank_select_for_bits!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128);
#[cargo_snippet::snippet("RankSelectDictionaries")]
pub struct BitVector {
    /// [(bit, sum)]
    data: Vec<(usize, usize)>,
    sum: usize,
}
#[cargo_snippet::snippet("RankSelectDictionaries")]
impl BitVector {
    const WORD_SIZE: usize = 0usize.count_zeros() as usize;
}
#[cargo_snippet::snippet("RankSelectDictionaries")]
impl RankSelectDictionaries for BitVector {
    fn bit_length(&self) -> usize {
        self.data.len() * Self::WORD_SIZE
    }
    fn access(&self, k: usize) -> bool {
        self.data[k / Self::WORD_SIZE].0 & (1 << k % Self::WORD_SIZE) != 0
    }
    fn rank1(&self, k: usize) -> usize {
        let (bit, sum) = self.data[k / Self::WORD_SIZE];
        sum + (bit & !(std::usize::MAX << k % Self::WORD_SIZE)).count_ones() as usize
    }
    fn select1(&self, mut k: usize) -> Option<usize> {
        let (mut l, mut r) = (0, self.data.len());
        if self.sum <= k {
            return None;
        }
        while r - l > 1 {
            let m = (l + r) / 2;
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
        if r * Self::WORD_SIZE - self.sum <= k {
            return None;
        }
        while r - l > 1 {
            let m = (l + r) / 2;
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
#[cargo_snippet::snippet("RankSelectDictionaries")]
impl std::iter::FromIterator<bool> for BitVector {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let mut data = Vec::new();
        let mut sum = 0;
        'outer: loop {
            let mut bit = 0;
            let mut nsum = sum;
            for i in 0..Self::WORD_SIZE {
                if let Some(b) = iter.next() {
                    if b {
                        bit |= 1 << i;
                        nsum += 1;
                    }
                } else {
                    data.push((bit, sum));
                    sum = nsum;
                    break 'outer;
                }
            }
            data.push((bit, sum));
            sum = nsum;
        }
        Self { data, sum }
    }
}

#[test]
fn test_rank_select_usize() {
    use crate::tools::Xorshift;
    const Q: usize = 1_000;
    const WORD_SIZE: usize = 0usize.count_zeros() as usize;
    let mut rand = Xorshift::time();
    for _ in 0..Q {
        let x = rand.next() as usize;
        {
            for k in 0..=WORD_SIZE {
                assert_eq!(x.rank1(k), (0..k).filter(|&i| x.access(i)).count());
                assert_eq!(x.rank0(k), (0..k).filter(|&i| !x.access(i)).count());
            }
        }

        {
            let k = rand.rand(WORD_SIZE as u64) as usize;
            if let Some(i) = x.select1(k) {
                assert_eq!((0..i).filter(|&j| x.access(j)).count(), k);
                assert!(x.access(i));
            } else {
                assert!(x.rank1(WORD_SIZE) <= k);
            }
        }

        {
            let k = rand.rand(WORD_SIZE as u64) as usize;
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
    use crate::tools::Xorshift;
    const Q: usize = 1_000;
    const N: usize = 1_000;
    let mut rand = Xorshift::time();
    let x: BitVector = (0..N).map(|_| rand.rand(2) != 0).collect();
    for _i in 0..Q {
        {
            let k = rand.rand(N as u64) as usize;
            assert_eq!(x.rank1(k), (0..k).filter(|&i| x.access(i)).count());
            assert_eq!(x.rank0(k), (0..k).filter(|&i| !x.access(i)).count());
        }

        {
            let k = rand.rand(N as u64) as usize;
            if let Some(i) = x.select1(k) {
                assert_eq!((0..i).filter(|&j| x.access(j)).count(), k);
                assert!(x.access(i));
            } else {
                assert!(x.rank1(N) <= k);
            }
        }

        {
            let k = rand.rand(N as u64) as usize;
            if let Some(i) = x.select0(k) {
                assert_eq!((0..i).filter(|&j| !x.access(j)).count(), k);
                assert!(!x.access(i));
            } else {
                assert!(x.rank0(N) <= k);
            }
        }
    }
    assert_eq!(x.rank1(0), 0);
    assert_eq!(x.rank0(0), 0);
}
