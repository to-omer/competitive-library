use super::{BitVector, Compressor, RankSelectDictionaries, VecCompress};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct WaveletMatrix<T> {
    len: usize,
    bit_length: usize,
    zeros: Vec<usize>,
    ones_prefix: Vec<usize>,
    bit_vector: BitVector,
    compress: VecCompress<T>,
}

impl<T> WaveletMatrix<T>
where
    T: Ord + Clone,
{
    pub fn new(v: Vec<T>) -> Self {
        let len = v.len();
        let compress: VecCompress<T> = v.iter().cloned().collect();
        let bit_length =
            usize::BITS as usize - compress.size().saturating_sub(1).leading_zeros() as usize;
        let mut indices: Vec<usize> = v
            .iter()
            .map(|value| compress.index_exact(value).unwrap())
            .collect();
        let mut bit_vector = BitVector::with_capacity(len * bit_length);
        let mut zeros = Vec::with_capacity(bit_length);
        for d in (0..bit_length).rev() {
            let mut zero_count = 0;
            for &idx in &indices {
                let bit = ((idx >> d) & 1) != 0;
                bit_vector.push(bit);
                if !bit {
                    zero_count += 1;
                }
            }
            zeros.push(zero_count);
            let mut next = Vec::with_capacity(len);
            next.extend(
                indices
                    .iter()
                    .filter(|&&idx| ((idx >> d) & 1) == 0)
                    .copied(),
            );
            next.extend(
                indices
                    .iter()
                    .filter(|&&idx| ((idx >> d) & 1) == 1)
                    .copied(),
            );
            indices = next;
        }
        let mut ones_prefix = Vec::with_capacity(bit_length);
        let mut prefix = 0;
        for &zero in &zeros {
            ones_prefix.push(prefix);
            prefix += len - zero;
        }
        Self {
            len,
            bit_length,
            zeros,
            ones_prefix,
            bit_vector,
            compress,
        }
    }

    pub fn new_with_init<F>(v: Vec<T>, mut f: F) -> Self
    where
        F: FnMut(usize, usize, T),
    {
        let this = Self::new(v.clone());
        let indices: Vec<usize> = v
            .iter()
            .map(|value| this.compress.index_exact(value).unwrap())
            .collect();
        for (mut k, value) in v.into_iter().enumerate() {
            let idx = indices[k];
            for d in (0..this.bit_length).rev() {
                let level = this.level(d);
                if ((idx >> d) & 1) != 0 {
                    k = this.zeros[level] + this.rank1(level, k);
                } else {
                    k = this.rank0(level, k);
                }
                f(d, k, value.clone());
            }
        }
        this
    }

    fn level(&self, d: usize) -> usize {
        self.bit_length - 1 - d
    }

    fn rank1(&self, level: usize, k: usize) -> usize {
        let offset = level * self.len;
        self.bit_vector.rank1(offset + k) - self.ones_prefix[level]
    }

    fn rank0(&self, level: usize, k: usize) -> usize {
        k - self.rank1(level, k)
    }

    fn rank_by_index(&self, idx: usize, mut range: Range<usize>) -> usize {
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            if ((idx >> d) & 1) != 0 {
                range.start = self.zeros[level] + self.rank1(level, range.start);
                range.end = self.zeros[level] + self.rank1(level, range.end);
            } else {
                range.start = self.rank0(level, range.start);
                range.end = self.rank0(level, range.end);
            }
        }
        range.end - range.start
    }

    /// get k-th value
    pub fn access(&self, mut k: usize) -> T {
        let mut idx = 0;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            if self.bit_vector.access(level * self.len + k) {
                idx |= 1 << d;
                k = self.zeros[level] + self.rank1(level, k);
            } else {
                k = self.rank0(level, k);
            }
        }
        self.compress.values()[idx].clone()
    }

    /// the number of val in range
    pub fn rank(&self, val: T, range: Range<usize>) -> usize {
        match self.compress.index_exact(&val) {
            Some(idx) => self.rank_by_index(idx, range),
            None => 0,
        }
    }

    /// index of k-th val
    pub fn select(&self, val: T, k: usize) -> Option<usize> {
        let idx = self.compress.index_exact(&val)?;
        if self.rank_by_index(idx, 0..self.len) <= k {
            return None;
        }
        let mut i = 0;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            if ((idx >> d) & 1) != 0 {
                i = self.zeros[level] + self.rank1(level, i);
            } else {
                i = self.rank0(level, i);
            }
        }
        i += k;
        for level in (0..self.bit_length).rev() {
            let offset = level * self.len;
            if i >= self.zeros[level] {
                let global_k = self.ones_prefix[level] + (i - self.zeros[level]);
                let pos = self.bit_vector.select1(global_k).unwrap();
                i = pos - offset;
            } else {
                let zeros_before = offset - self.ones_prefix[level];
                let global_k = zeros_before + i;
                let pos = self.bit_vector.select0(global_k).unwrap();
                i = pos - offset;
            }
        }
        Some(i)
    }

    /// get k-th smallest value in range
    pub fn quantile(&self, mut range: Range<usize>, mut k: usize) -> T {
        let mut idx = 0;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            let z = self.rank0(level, range.end) - self.rank0(level, range.start);
            if z <= k {
                k -= z;
                idx |= 1 << d;
                range.start = self.zeros[level] + self.rank1(level, range.start);
                range.end = self.zeros[level] + self.rank1(level, range.end);
            } else {
                range.start = self.rank0(level, range.start);
                range.end = self.rank0(level, range.end);
            }
        }
        self.compress.values()[idx].clone()
    }

    /// get k-th smallest value out of range
    pub fn quantile_outer(&self, mut range: Range<usize>, mut k: usize) -> T {
        let mut idx = 0;
        let mut orange = 0..self.len;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            let z = self.rank0(level, orange.end) - self.rank0(level, orange.start)
                + self.rank0(level, range.start)
                - self.rank0(level, range.end);
            if z <= k {
                k -= z;
                idx |= 1 << d;
                range.start = self.zeros[level] + self.rank1(level, range.start);
                range.end = self.zeros[level] + self.rank1(level, range.end);
                orange.start = self.zeros[level] + self.rank1(level, orange.start);
                orange.end = self.zeros[level] + self.rank1(level, orange.end);
            } else {
                range.start = self.rank0(level, range.start);
                range.end = self.rank0(level, range.end);
                orange.start = self.rank0(level, orange.start);
                orange.end = self.rank0(level, orange.end);
            }
        }
        self.compress.values()[idx].clone()
    }

    /// the number of value less than val in range
    pub fn rank_lessthan(&self, val: T, mut range: Range<usize>) -> usize {
        let idx = self.compress.index_lower_bound(&val);
        let size = self.compress.size();
        if size.is_power_of_two() && idx == size {
            return range.end - range.start;
        }
        let mut res = 0;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            if ((idx >> d) & 1) != 0 {
                res += self.rank0(level, range.end) - self.rank0(level, range.start);
                range.start = self.zeros[level] + self.rank1(level, range.start);
                range.end = self.zeros[level] + self.rank1(level, range.end);
            } else {
                range.start = self.rank0(level, range.start);
                range.end = self.rank0(level, range.end);
            }
        }
        res
    }

    /// the number of valrange in range
    pub fn rank_range(&self, valrange: Range<T>, range: Range<usize>) -> usize {
        self.rank_lessthan(valrange.end, range.clone()) - self.rank_lessthan(valrange.start, range)
    }

    pub fn query_less_than<F>(&self, val: T, mut range: Range<usize>, mut f: F)
    where
        F: FnMut(usize, Range<usize>),
    {
        let idx = self.compress.index_lower_bound(&val);
        let size = self.compress.size();
        if size.is_power_of_two() && idx == size {
            if let Some(level) = self.bit_length.checked_sub(1) {
                f(level, range);
            }
            return;
        }
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            if ((idx >> d) & 1) != 0 {
                f(
                    d,
                    self.rank0(level, range.start)..self.rank0(level, range.end),
                );
                range.start = self.zeros[level] + self.rank1(level, range.start);
                range.end = self.zeros[level] + self.rank1(level, range.end);
            } else {
                range.start = self.rank0(level, range.start);
                range.end = self.rank0(level, range.end);
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
        let wm = WaveletMatrix::new(v.clone());
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
