use super::{BitVector, RankSelectDictionaries};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct WaveletMatrix {
    len: usize,
    table: Vec<(usize, BitVector)>,
}

impl WaveletMatrix {
    pub fn new<T>(mut v: Vec<T>, bit_length: usize) -> Self
    where
        T: Clone + RankSelectDictionaries,
    {
        let len = v.len();
        let mut table = Vec::new();
        for d in (0..bit_length).rev() {
            let b: BitVector = v.iter().map(|x| x.access(d)).collect();
            table.push((b.rank0(len), b));
            v = v
                .iter()
                .filter(|&x| !x.access(d))
                .chain(v.iter().filter(|&x| x.access(d)))
                .cloned()
                .collect();
        }
        Self { len, table }
    }
    pub fn new_with_init<T, F>(v: Vec<T>, bit_length: usize, mut f: F) -> Self
    where
        T: Clone + RankSelectDictionaries,
        F: FnMut(usize, usize, T),
    {
        let this = Self::new(v.clone(), bit_length);
        for (mut k, v) in v.into_iter().enumerate() {
            for (d, &(c, ref b)) in this.table.iter().rev().enumerate().rev() {
                if v.access(d) {
                    k = c + b.rank1(k);
                } else {
                    k = b.rank0(k);
                }
                f(d, k, v.clone());
            }
        }
        this
    }
    /// get k-th value
    pub fn access(&self, mut k: usize) -> usize {
        let mut val = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if b.access(k) {
                k = c + b.rank1(k);
                val |= 1 << d;
            } else {
                k = b.rank0(k);
            }
        }
        val
    }
    /// the number of val in range
    pub fn rank(&self, val: usize, mut range: Range<usize>) -> usize {
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
        range.end - range.start
    }
    /// index of k-th val
    pub fn select(&self, val: usize, k: usize) -> Option<usize> {
        if self.rank(val, 0..self.len) <= k {
            return None;
        }
        let mut i = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                i = c + b.rank1(i);
            } else {
                i = b.rank0(i);
            }
        }
        i += k;
        for &(c, ref b) in self.table.iter().rev() {
            if i >= c {
                i = b.select1(i - c).unwrap();
            } else {
                i = b.select0(i).unwrap();
            }
        }
        Some(i)
    }
    /// get k-th smallest value in range
    pub fn quantile(&self, mut range: Range<usize>, mut k: usize) -> usize {
        let mut val = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            let z = b.rank0(range.end) - b.rank0(range.start);
            if z <= k {
                k -= z;
                val |= 1 << d;
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
        val
    }
    /// get k-th smallest value out of range
    pub fn quantile_outer(&self, mut range: Range<usize>, mut k: usize) -> usize {
        let mut val = 0;
        let mut orange = 0..self.len;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            let z = b.rank0(orange.end) - b.rank0(orange.start) + b.rank0(range.start)
                - b.rank0(range.end);
            if z <= k {
                k -= z;
                val |= 1 << d;
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
                orange.start = c + b.rank1(orange.start);
                orange.end = c + b.rank1(orange.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
                orange.start = b.rank0(orange.start);
                orange.end = b.rank0(orange.end);
            }
        }
        val
    }
    /// the number of value less than val in range
    pub fn rank_lessthan(&self, val: usize, mut range: Range<usize>) -> usize {
        let mut res = 0;
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                res += b.rank0(range.end) - b.rank0(range.start);
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
        res
    }
    /// the number of valrange in range
    pub fn rank_range(&self, valrange: Range<usize>, range: Range<usize>) -> usize {
        self.rank_lessthan(valrange.end, range.clone()) - self.rank_lessthan(valrange.start, range)
    }
    pub fn query_less_than<F>(&self, val: usize, mut range: Range<usize>, mut f: F)
    where
        F: FnMut(usize, Range<usize>),
    {
        for (d, &(c, ref b)) in self.table.iter().rev().enumerate().rev() {
            if val.access(d) {
                f(d, b.rank0(range.start)..b.rank0(range.end));
                range.start = c + b.rank1(range.start);
                range.end = c + b.rank1(range.end);
            } else {
                range.start = b.rank0(range.start);
                range.end = b.rank0(range.end);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rand_value,
        tools::{NotEmptySegment as Nes, Xorshift},
    };

    #[test]
    fn test_wavelet_matrix() {
        const N: usize = 1_000;
        const Q: usize = 1_000;
        const A: usize = 1 << 8;
        let mut rng = Xorshift::default();
        crate::rand!(rng, v: [..A; N]);
        let wm = WaveletMatrix::new(v.clone(), 8);
        for (i, v) in v.iter().cloned().enumerate() {
            assert_eq!(wm.access(i), v);
        }
        for ((l, r), a) in rand_value!(rng, [(Nes(N), ..A); Q]) {
            assert_eq!(
                wm.rank(a, l..r),
                v[l..r].iter().filter(|&&x| x == a).count()
            );

            if wm.rank(a, 0..N) > 0 {
                let k = rng.random(..wm.rank(a, 0..N));
                assert_eq!(
                    wm.select(a, k).unwrap().min(N),
                    (0..N)
                        .position(|i| wm.rank(a, 0..i + 1) == k + 1)
                        .unwrap_or(N)
                );
            }

            assert_eq!(
                (0..r - l).map(|k| wm.quantile(l..r, k)).collect::<Vec<_>>(),
                {
                    let mut v: Vec<_> = v[l..r].to_vec();
                    v.sort_unstable();
                    v
                }
            );

            assert_eq!(
                (0..N + l - r)
                    .map(|k| wm.quantile_outer(l..r, k))
                    .collect::<Vec<_>>(),
                {
                    let mut v: Vec<_> = v.to_vec();
                    v.drain(l..r);
                    v.sort_unstable();
                    v
                }
            );

            assert_eq!(
                wm.rank_lessthan(a, l..r),
                v[l..r].iter().filter(|&&x| x < a).count()
            );

            let (p, q) = rng.random(Nes(A - 1));
            assert_eq!(
                wm.rank_range(p..q, l..r),
                v[l..r].iter().filter(|&&x| p <= x && x < q).count()
            );
        }
    }
}
