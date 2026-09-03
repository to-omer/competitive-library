use super::{
    AbelianGroup, BinaryIndexedTree, BitVector, Compressor, RankSelectDictionaries, VecCompress,
};
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
        for (mut k, value) in v.into_iter().enumerate() {
            for d in (0..this.bit_length).rev() {
                let level = this.level(d);
                let (bit, rank1) = this.bit_vectors[level].access_rank1(k);
                if bit {
                    k = this.zeros[level] + rank1;
                } else {
                    k -= rank1;
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

    fn reorder<U>(&self, level: usize, current: Vec<U>, mut visit: impl FnMut(&U)) -> Vec<U> {
        assert_eq!(current.len(), self.len);
        let mut next = Vec::with_capacity(self.len);
        next.resize_with(self.len, MaybeUninit::uninit);
        let mut zero = 0;
        let mut one = self.zeros[level];
        for (i, value) in current.into_iter().enumerate() {
            visit(&value);
            if self.bit_vectors[level].access(i) {
                next[one].write(value);
                one += 1;
            } else {
                next[zero].write(value);
                zero += 1;
            }
        }
        // SAFETY: the partition counts fill every slot once, and `MaybeUninit<U>` has `U`'s layout.
        unsafe {
            let mut next = mem::ManuallyDrop::new(next);
            Vec::from_raw_parts(next.as_mut_ptr().cast(), next.len(), next.capacity())
        }
    }

    fn rank_by_index(&self, idx: usize, mut range: Range<usize>) -> usize {
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            if ((idx >> d) & 1) != 0 {
                range.start = self.zeros[level] + start1;
                range.end = self.zeros[level] + end1;
            } else {
                range.start -= start1;
                range.end -= end1;
            }
        }
        range.end - range.start
    }

    /// get k-th value
    pub fn access(&self, mut k: usize) -> T {
        let mut idx = 0;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            let (bit, rank1) = self.bit_vectors[level].access_rank1(k);
            if bit {
                idx |= 1 << d;
                k = self.zeros[level] + rank1;
            } else {
                k -= rank1;
            }
        }
        self.compress.values()[idx].clone()
    }

    /// Returns the values at `indices`, traversing queries together in groups of 16.
    pub fn access_batch(&self, indices: impl IntoIterator<Item = usize>) -> Vec<T> {
        let indices: Vec<_> = indices.into_iter().collect();
        if indices.len() < 8 || self.len > u32::MAX as usize {
            return indices
                .into_iter()
                .map(|index| self.access(index))
                .collect();
        }
        let mut result = Vec::with_capacity(indices.len());
        for indices in indices.chunks(16) {
            if indices.len() < 8 {
                result.extend(indices.iter().map(|&index| self.access(index)));
                continue;
            }
            let mut states = [[0_u32; 2]; 16];
            for (state, &index) in states.iter_mut().zip(indices) {
                state[0] = index as u32;
            }
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                for state in &mut states[..indices.len()] {
                    let position = state[0] as usize;
                    let (bit, rank1) = self.bit_vectors[level].access_rank1(position);
                    if bit {
                        state[0] = (self.zeros[level] + rank1) as u32;
                        state[1] |= 1 << d;
                    } else {
                        state[0] = (position - rank1) as u32;
                    }
                }
            }
            result.extend(
                states[..indices.len()]
                    .iter()
                    .map(|state| self.compress.values()[state[1] as usize].clone()),
            );
        }
        result
    }

    /// the number of val in range
    pub fn rank(&self, val: T, range: Range<usize>) -> usize {
        match self.compress.index_exact(&val) {
            Some(idx) => self.rank_by_index(idx, range),
            None => 0,
        }
    }

    /// Returns the number of exact matches for each `(value, range)` query.
    pub fn rank_batch(&self, queries: impl IntoIterator<Item = (T, Range<usize>)>) -> Vec<usize> {
        let queries: Vec<_> = queries.into_iter().collect();
        if queries.len() < 8 || self.len > u32::MAX as usize {
            return queries
                .into_iter()
                .map(|(value, range)| self.rank(value, range))
                .collect();
        }
        let mut result = vec![0; queries.len()];
        let mut active = Vec::new();
        for (output, (value, range)) in queries.into_iter().enumerate() {
            if let Some(index) = self.compress.index_exact(&value) {
                active.push((output, index, range));
            }
        }
        for queries in active.chunks(16) {
            if queries.len() < 8 {
                for (output, index, range) in queries {
                    result[*output] = self.rank_by_index(*index, range.clone());
                }
                continue;
            }
            let mut states = [[0_u32; 3]; 16];
            for (state, (_, index, range)) in states.iter_mut().zip(queries) {
                *state = [range.start as u32, range.end as u32, *index as u32];
            }
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                for state in &mut states[..queries.len()] {
                    let start = state[0] as usize;
                    let end = state[1] as usize;
                    let start1 = self.rank1(level, start);
                    let end1 = self.rank1(level, end);
                    if ((state[2] >> d) & 1) != 0 {
                        state[0] = (self.zeros[level] + start1) as u32;
                        state[1] = (self.zeros[level] + end1) as u32;
                    } else {
                        state[0] = (start - start1) as u32;
                        state[1] = (end - end1) as u32;
                    }
                }
            }
            for ((output, _, _), state) in queries.iter().zip(&states) {
                result[*output] = (state[1] - state[0]) as usize;
            }
        }
        result
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
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            let z = end0 - start0;
            if z <= k {
                k -= z;
                idx |= 1 << d;
                range.start = self.zeros[level] + start1;
                range.end = self.zeros[level] + end1;
            } else {
                range.start = start0;
                range.end = end0;
            }
        }
        self.compress.values()[idx].clone()
    }

    pub fn quantile_batch(
        &self,
        queries: impl IntoIterator<Item = (Range<usize>, usize)>,
    ) -> Vec<T> {
        let queries: Vec<_> = queries.into_iter().collect();
        if queries.len() < 8 || self.len > u32::MAX as usize {
            return queries
                .into_iter()
                .map(|(range, k)| self.quantile(range, k))
                .collect();
        }
        let mut result = Vec::with_capacity(queries.len());
        for queries in queries.chunks(16) {
            if queries.len() < 8 {
                result.extend(
                    queries
                        .iter()
                        .map(|(range, k)| self.quantile(range.clone(), *k)),
                );
                continue;
            }
            let mut states = [[0_u32; 4]; 16];
            for (state, (range, k)) in states.iter_mut().zip(queries) {
                *state = [range.start as u32, range.end as u32, *k as u32, 0];
            }
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                for state in &mut states[..queries.len()] {
                    let start = state[0] as usize;
                    let end = state[1] as usize;
                    let start1 = self.rank1(level, start);
                    let end1 = self.rank1(level, end);
                    let start0 = (start - start1) as u32;
                    let end0 = (end - end1) as u32;
                    let zeros = end0 - start0;
                    let mask = 0u32.wrapping_sub((state[2] >= zeros) as u32);
                    state[0] =
                        (start0 & !mask) | ((self.zeros[level] as u32 + start1 as u32) & mask);
                    state[1] = (end0 & !mask) | ((self.zeros[level] as u32 + end1 as u32) & mask);
                    state[2] -= zeros & mask;
                    state[3] |= (1u32 << d) & mask;
                }
            }
            result.extend(
                states[..queries.len()]
                    .iter()
                    .map(|state| self.compress.values()[state[3] as usize].clone()),
            );
        }
        result
    }

    /// get k-th smallest value out of range
    pub fn quantile_outer(&self, mut range: Range<usize>, mut k: usize) -> T {
        let mut idx = 0;
        let mut orange = 0..self.len;
        for d in (0..self.bit_length).rev() {
            let level = self.level(d);
            let range_start1 = self.rank1(level, range.start);
            let range_end1 = self.rank1(level, range.end);
            let outer_start1 = self.rank1(level, orange.start);
            let outer_end1 = self.rank1(level, orange.end);
            let range_start0 = range.start - range_start1;
            let range_end0 = range.end - range_end1;
            let outer_start0 = orange.start - outer_start1;
            let outer_end0 = orange.end - outer_end1;
            let z = (outer_end0 - outer_start0) - (range_end0 - range_start0);
            if z <= k {
                k -= z;
                idx |= 1 << d;
                range.start = self.zeros[level] + range_start1;
                range.end = self.zeros[level] + range_end1;
                orange.start = self.zeros[level] + outer_start1;
                orange.end = self.zeros[level] + outer_end1;
            } else {
                range.start = range_start0;
                range.end = range_end0;
                orange.start = outer_start0;
                orange.end = outer_end0;
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
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            if ((idx >> d) & 1) != 0 {
                res += (range.end - end1) - (range.start - start1);
                range.start = self.zeros[level] + start1;
                range.end = self.zeros[level] + end1;
            } else {
                range.start -= start1;
                range.end -= end1;
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
            let start1 = self.rank1(level, range.start);
            let end1 = self.rank1(level, range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((idx >> d) & 1) != 0 {
                f(d, start0..end0);
                range.start = self.zeros[level] + start1;
                range.end = self.zeros[level] + end1;
            } else {
                range.start = start0;
                range.end = end0;
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
            let mut acc = M::unit();
            prefix.push(acc.clone());
            current = self.reorder(level, current, |w| {
                acc = M::operate(&acc, w);
                prefix.push(acc.clone());
            });
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

    pub fn build_point_add<M>(&self, weights: &[M::T]) -> WaveletMatrixPointAdd<'_, T, M>
    where
        M: AbelianGroup,
    {
        assert_eq!(weights.len(), self.len);
        let mut current = weights.to_vec();
        let mut bits = Vec::with_capacity(self.bit_length);
        for level in 0..self.bit_length {
            current = self.reorder(level, current, |_| {});
            bits.push(BinaryIndexedTree::from_slice(&current));
        }
        WaveletMatrixPointAdd {
            wavelet_matrix: self,
            bits,
        }
    }
}

pub struct WaveletMatrixPointAdd<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    wavelet_matrix: &'a WaveletMatrix<T>,
    bits: Vec<BinaryIndexedTree<M>>,
}

impl<'a, T, M> WaveletMatrixPointAdd<'a, T, M>
where
    T: Ord + Clone,
    M: AbelianGroup,
{
    pub fn update(&mut self, mut index: usize, value: M::T) {
        debug_assert!(index < self.wavelet_matrix.len);
        for d in (0..self.wavelet_matrix.bit_length).rev() {
            let level = self.wavelet_matrix.level(d);
            let (bit, rank1) = self.wavelet_matrix.bit_vectors[level].access_rank1(index);
            if bit {
                index = self.wavelet_matrix.zeros[level] + rank1;
            } else {
                index -= rank1;
            }
            self.bits[level].update(index, value.clone());
        }
    }

    pub fn fold_lessthan(&self, value: T, range: Range<usize>) -> M::T {
        let mut result = M::unit();
        self.wavelet_matrix
            .query_less_than(value, range, |d, range| {
                M::operate_assign(
                    &mut result,
                    &self.bits[self.wavelet_matrix.level(d)].fold(range.start, range.end),
                );
            });
        result
    }

    pub fn fold_range(&self, values: Range<T>, range: Range<usize>) -> M::T {
        M::rinv_operate(
            &self.fold_lessthan(values.end, range.clone()),
            &self.fold_lessthan(values.start, range),
        )
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
    pub fn fold_lessthan(&self, val: T, range: Range<usize>) -> M::T {
        self.fold_lessthan_with_count(val, range).1
    }

    pub fn fold_lessthan_with_count(&self, val: T, range: Range<usize>) -> (usize, M::T) {
        debug_assert!(range.end <= self.wavelet_matrix.len);
        self.fold_lessthan_index_with_count(
            self.wavelet_matrix.compress.index_lower_bound(&val),
            range,
            self.wavelet_matrix.bit_length,
        )
    }

    pub fn fold_range(&self, valrange: Range<T>, range: Range<usize>) -> M::T {
        self.fold_range_with_count(valrange, range).1
    }

    pub fn fold_range_with_count(
        &self,
        valrange: Range<T>,
        mut range: Range<usize>,
    ) -> (usize, M::T) {
        debug_assert!(range.end <= self.wavelet_matrix.len);
        let lower = self
            .wavelet_matrix
            .compress
            .index_lower_bound(&valrange.start);
        let upper = self
            .wavelet_matrix
            .compress
            .index_lower_bound(&valrange.end);
        if lower >= upper {
            return (0, M::unit());
        }
        for d in (0..self.wavelet_matrix.bit_length).rev() {
            let level = self.wavelet_matrix.level(d);
            let start1 = self.wavelet_matrix.bit_vectors[level].rank1(range.start);
            let end1 = self.wavelet_matrix.bit_vectors[level].rank1(range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((lower >> d) & 1) == ((upper >> d) & 1) {
                if ((lower >> d) & 1) == 0 {
                    range = start0..end0;
                } else {
                    range = self.wavelet_matrix.zeros[level] + start1
                        ..self.wavelet_matrix.zeros[level] + end1;
                }
                continue;
            }
            let zero_range = start0..end0;
            let one_range =
                self.wavelet_matrix.zeros[level] + start1..self.wavelet_matrix.zeros[level] + end1;
            let (lower_count, lower_sum) =
                self.fold_lessthan_index_with_count(lower, zero_range.clone(), d);
            let (upper_count, upper_sum) = self.fold_lessthan_index_with_count(upper, one_range, d);
            let zero_sum = self.range_sum(level + 1, zero_range.clone());
            return (
                zero_range.len() - lower_count + upper_count,
                M::operate(&M::rinv_operate(&zero_sum, &lower_sum), &upper_sum),
            );
        }
        (0, M::unit())
    }

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

    fn fold_lessthan_index_with_count(
        &self,
        idx: usize,
        mut range: Range<usize>,
        bits: usize,
    ) -> (usize, M::T) {
        let mut count = 0;
        let mut sum = M::unit();
        for d in (0..bits).rev() {
            let level = self.wavelet_matrix.level(d);
            let start1 = self.wavelet_matrix.bit_vectors[level].rank1(range.start);
            let end1 = self.wavelet_matrix.bit_vectors[level].rank1(range.end);
            let start0 = range.start - start1;
            let end0 = range.end - end1;
            if ((idx >> d) & 1) != 0 {
                count += end0 - start0;
                sum = M::operate(&sum, &self.range_sum(level + 1, start0..end0));
                range.start = self.wavelet_matrix.zeros[level] + start1;
                range.end = self.wavelet_matrix.zeros[level] + end1;
            } else {
                range.start = start0;
                range.end = end0;
            }
        }
        (count, sum)
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
        let mut dynamic_weights = w.clone();
        let mut dynamic: WaveletMatrixPointAdd<_, AdditiveOperation<i64>> =
            wm.build_point_add(&dynamic_weights);
        for (i, v) in v.iter().cloned().enumerate() {
            assert_eq!(wm.access(i), v);
        }
        assert_eq!(wm.access_batch(0..N), v);
        assert_eq!(wm.access_batch(0..7), v[..7]);
        assert_eq!(fold.fold_lessthan(A, 0..N), w.iter().sum::<i64>());
        assert_eq!(fold.fold_range(A..A, 0..N), 0);
        assert_eq!(fold.fold_range_with_count(A..A, 0..N), (0, 0));
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
        let rank_queries: Vec<_> = (0..Q)
            .map(|_| {
                let left = rng.random(0..N);
                let right = rng.random(left..=N);
                (rng.random(0..A), left..right)
            })
            .collect();
        let expected: Vec<_> = rank_queries
            .iter()
            .map(|(value, range)| v[range.clone()].iter().filter(|x| *x == value).count())
            .collect();
        assert_eq!(wm.rank_batch(rank_queries), expected);
        let rank_queries: Vec<_> = (0..7).map(|value| (value, 0..N)).collect();
        let expected: Vec<_> = rank_queries
            .iter()
            .map(|(value, range)| v[range.clone()].iter().filter(|x| *x == value).count())
            .collect();
        assert_eq!(wm.rank_batch(rank_queries), expected);
        for ((l, r), a) in rand_value!(rng, [(Nes(N), ..A); Q]) {
            let (i, value) = rng.random((..N, -B..B));
            dynamic.update(i, value);
            dynamic_weights[i] += value;
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
            assert_eq!(
                dynamic.fold_lessthan(a, l..r),
                v[l..r]
                    .iter()
                    .zip(&dynamic_weights[l..r])
                    .filter(|&(&value, _)| value < a)
                    .map(|(_, &weight)| weight)
                    .sum()
            );

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
                dynamic.fold_range(p..q, l..r),
                v[l..r]
                    .iter()
                    .zip(&dynamic_weights[l..r])
                    .filter(|&(&value, _)| p <= value && value < q)
                    .map(|(_, &weight)| weight)
                    .sum()
            );
            assert_eq!(
                fold.fold_range_with_count(p..q, l..r),
                (count_range, sum_range)
            );
        }
    }
}
