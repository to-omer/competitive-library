use super::{AbelianGroup, BitVector, Compressor, RankSelectDictionaries, VecCompress};
use std::{
    mem::{self, MaybeUninit},
    ops::Range,
};

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
        let bit_length = usize::BITS as usize - compress.size().leading_zeros() as usize;
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

    pub fn build_fold<M>(&self, weights: &[M::T]) -> WaveletMatrixFold<'_, T, M>
    where
        M: AbelianGroup,
    {
        let len = self.len;
        assert_eq!(weights.len(), len);
        let mut prefix = Vec::with_capacity((self.bit_length + 1) * (len + 1));
        let mut current: Vec<M::T> = weights.to_vec();
        for level in 0..self.bit_length {
            let offset = level * len;
            let zeros = self.zeros[level];
            let mut next: Vec<MaybeUninit<M::T>> = Vec::with_capacity(len);
            next.resize_with(len, MaybeUninit::uninit);
            let mut zero_pos = 0;
            let mut one_pos = zeros;
            let mut acc = M::unit();
            prefix.push(acc.clone());
            for (i, w) in current.into_iter().enumerate() {
                acc = M::operate(&acc, &w);
                prefix.push(acc.clone());
                if self.bit_vector.access(offset + i) {
                    next[one_pos].write(w);
                    one_pos += 1;
                } else {
                    next[zero_pos].write(w);
                    zero_pos += 1;
                }
            }
            debug_assert_eq!(zero_pos, zeros);
            debug_assert_eq!(one_pos, len);
            let next = unsafe {
                let mut next = mem::ManuallyDrop::new(next);
                let ptr = next.as_mut_ptr() as *mut M::T;
                let len = next.len();
                let cap = next.capacity();
                Vec::from_raw_parts(ptr, len, cap)
            };
            current = next;
        }
        let mut acc = M::unit();
        prefix.push(acc.clone());
        for w in current.into_iter() {
            acc = M::operate(&acc, &w);
            prefix.push(acc.clone());
        }
        WaveletMatrixFold {
            wavelet_matrix: self,
            prefix,
        }
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
}

impl<'a, T, M> WaveletMatrixFold<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    #[inline]
    fn range_sum(&self, level: usize, range: Range<usize>) -> M::T {
        let offset = level * (self.wavelet_matrix.len + 1);
        unsafe {
            M::rinv_operate(
                self.prefix.get_unchecked(offset + range.end),
                self.prefix.get_unchecked(offset + range.start),
            )
        }
    }

    pub fn fold_lessthan(&self, val: T, range: Range<usize>) -> M::T {
        self.fold_lessthan_with_count(val, range).1
    }

    pub fn fold_lessthan_with_count(&self, val: T, mut range: Range<usize>) -> (usize, M::T) {
        debug_assert!(range.end <= self.wavelet_matrix.len);
        let idx = self.wavelet_matrix.compress.index_lower_bound(&val);
        let mut count = 0;
        let mut sum = M::unit();
        for d in (0..self.wavelet_matrix.bit_length).rev() {
            let level = self.wavelet_matrix.level(d);
            let start0 = self.wavelet_matrix.rank0(level, range.start);
            let end0 = self.wavelet_matrix.rank0(level, range.end);
            if ((idx >> d) & 1) != 0 {
                count += end0 - start0;
                sum = M::operate(&sum, &self.range_sum(level + 1, start0..end0));
                range.start = self.wavelet_matrix.zeros[level] + (range.start - start0);
                range.end = self.wavelet_matrix.zeros[level] + (range.end - end0);
            } else {
                range.start = start0;
                range.end = end0;
            }
        }
        (count, sum)
    }

    pub fn fold_range(&self, valrange: Range<T>, range: Range<usize>) -> M::T {
        M::rinv_operate(
            &self.fold_lessthan(valrange.end, range.clone()),
            &self.fold_lessthan(valrange.start, range),
        )
    }

    pub fn fold_range_with_count(&self, valrange: Range<T>, range: Range<usize>) -> (usize, M::T) {
        let (count_upper, sum_upper) = self.fold_lessthan_with_count(valrange.end, range.clone());
        let (count_lower, sum_lower) = self.fold_lessthan_with_count(valrange.start, range);
        (
            count_upper - count_lower,
            M::rinv_operate(&sum_upper, &sum_lower),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AdditiveOperation,
        rand_value,
        tools::{NotEmptySegment as Nes, Xorshift},
    };

    #[test]
    fn test_wavelet_matrix() {
        const N: usize = 1_000;
        const Q: usize = 1_000;
        const A: usize = 1 << 8;
        const B: i64 = 1_000_000_000;
        let mut rng = Xorshift::default();
        crate::rand!(rng, v: [..A; N]);
        crate::rand!(rng, w: [-B..B; N]);
        let wm = WaveletMatrix::new(v.clone());
        let fold = wm.build_fold::<AdditiveOperation<i64>>(&w);
        for (i, v) in v.iter().cloned().enumerate() {
            assert_eq!(wm.access(i), v);
        }
        assert_eq!(fold.fold_lessthan(A, 0..N), w.iter().sum::<i64>());
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

            let mut count_lt = 0usize;
            let mut sum_lt = 0i64;
            for (&value, &weight) in v[l..r].iter().zip(w[l..r].iter()) {
                if value < a {
                    count_lt += 1;
                    sum_lt += weight;
                }
            }
            assert_eq!(fold.fold_lessthan_with_count(a, l..r), (count_lt, sum_lt));
            assert_eq!(fold.fold_lessthan(A, l..r), w[l..r].iter().sum::<i64>());

            let (p, q) = rng.random(Nes(A - 1));
            assert_eq!(
                wm.rank_range(p..q, l..r),
                v[l..r].iter().filter(|&&x| p <= x && x < q).count()
            );
            let mut count_range = 0usize;
            let mut sum_range = 0i64;
            for (&value, &weight) in v[l..r].iter().zip(w[l..r].iter()) {
                if p <= value && value < q {
                    count_range += 1;
                    sum_range += weight;
                }
            }
            assert_eq!(fold.fold_range(p..q, l..r), sum_range);
            assert_eq!(
                fold.fold_range_with_count(p..q, l..r),
                (count_range, sum_range)
            );
        }
    }
}
