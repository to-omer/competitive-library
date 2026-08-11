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
    bit_vectors: Vec<BitVector>,
    compress: VecCompress<T>,
}

impl<T> WaveletMatrix<T>
where
    T: Ord + Clone,
{
    pub fn new(v: Vec<T>) -> Self {
        let len = v.len();
        let mut sorted: Vec<_> = v
            .into_iter()
            .enumerate()
            .map(|(i, value)| (value, i))
            .collect();
        sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut values = Vec::with_capacity(len);
        let mut indices = vec![0; len];
        for (value, i) in sorted {
            if values.last().is_none_or(|last| last != &value) {
                values.push(value);
            }
            indices[i] = values.len() - 1;
        }
        let compress = VecCompress::from_sorted_unique(values);
        let bit_length = usize::BITS as usize - compress.size().leading_zeros() as usize;
        let mut bit_vectors = Vec::with_capacity(bit_length);
        let mut zeros = Vec::with_capacity(bit_length);
        let mut next = Vec::with_capacity(len);
        let mut ones = Vec::with_capacity(len);
        for d in (0..bit_length).rev() {
            bit_vectors.push(indices.iter().map(|&idx| ((idx >> d) & 1) != 0).collect());
            for &idx in &indices {
                if ((idx >> d) & 1) == 0 {
                    next.push(idx);
                } else {
                    ones.push(idx);
                }
            }
            zeros.push(next.len());
            next.append(&mut ones);
            mem::swap(&mut indices, &mut next);
            next.clear();
        }
        Self {
            len,
            bit_length,
            zeros,
            bit_vectors,
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
        self.bit_vectors[level].rank1(k)
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
            if self.bit_vectors[level].access(k) {
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
            if i >= self.zeros[level] {
                i = self.bit_vectors[level]
                    .select1(i - self.zeros[level])
                    .unwrap();
            } else {
                i = self.bit_vectors[level].select0(i).unwrap();
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

    pub fn quantile_batch(
        &self,
        queries: impl IntoIterator<Item = (Range<usize>, usize)>,
    ) -> Vec<T> {
        let mut queries: Vec<_> = queries
            .into_iter()
            .map(|(range, k)| [range.start as u32, range.end as u32, k as u32, 0])
            .collect();
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            for query in &mut queries {
                let start = query[0] as usize;
                let end = query[1] as usize;
                let start1 = self.rank1(level, start);
                let end1 = self.rank1(level, end);
                let start0 = (start - start1) as u32;
                let end0 = (end - end1) as u32;
                let zeros = end0 - start0;
                let mask = 0u32.wrapping_sub((query[2] >= zeros) as u32);
                query[0] = (start0 & !mask) | ((self.zeros[level] as u32 + start1 as u32) & mask);
                query[1] = (end0 & !mask) | ((self.zeros[level] as u32 + end1 as u32) & mask);
                query[2] -= zeros & mask;
                query[3] |= (1u32 << d) & mask;
            }
        }
        queries
            .into_iter()
            .map(|query| self.compress.values()[query[3] as usize].clone())
            .collect()
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
                if self.bit_vectors[level].access(i) {
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
        let quantile_queries: Vec<_> = (0..Q)
            .map(|_| {
                let l = rng.random(0..N);
                let r = rng.random(l + 1..=N);
                let k = rng.random(0..r - l);
                (l..r, k)
            })
            .collect();
        let expected: Vec<_> = quantile_queries
            .iter()
            .map(|(range, k)| {
                let mut values = v[range.clone()].to_vec();
                values.sort_unstable();
                values[*k]
            })
            .collect();
        assert_eq!(wm.quantile_batch(quantile_queries), expected);
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
