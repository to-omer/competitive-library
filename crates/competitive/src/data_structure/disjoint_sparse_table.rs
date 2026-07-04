use super::SemiGroup;
use std::{
    fmt::{self, Debug, Formatter},
    ops::Index,
};

pub struct DisjointSparseTable<S>
where
    S: SemiGroup,
{
    table: Vec<S::T>,
    offsets: Vec<usize>,
    len: usize,
}

impl<S> Clone for DisjointSparseTable<S>
where
    S: SemiGroup,
{
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            offsets: self.offsets.clone(),
            len: self.len,
        }
    }
}

impl<S> Debug for DisjointSparseTable<S>
where
    S: SemiGroup<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisjointSparseTable")
            .field("table", &self.table)
            .field("offsets", &self.offsets)
            .field("len", &self.len)
            .finish()
    }
}

impl<S> DisjointSparseTable<S>
where
    S: SemiGroup,
{
    pub fn new(v: Vec<S::T>) -> Self {
        let n = v.len();
        let mut levels = 1;
        let mut k = 2;
        while k < n {
            levels += 1;
            k *= 2;
        }

        let mut table = Vec::with_capacity(n * levels);
        table.extend(v);
        let mut offsets = vec![0];
        let mut k = 2;
        while k < n {
            let offset = table.len();
            offsets.push(offset);
            table.extend_from_within(0..n);
            for i in (0..n).step_by(k * 2) {
                for j in (i..n.min(i + k) - 1).rev() {
                    let j = offset + j;
                    let x = S::operate(&table[j], &table[j + 1]);
                    table[j] = x;
                }
                for j in i + k + 1..n.min(i + k * 2) {
                    let j = offset + j;
                    let x = S::operate(&table[j - 1], &table[j]);
                    table[j] = x;
                }
            }
            k *= 2;
        }
        Self {
            table,
            offsets,
            len: n,
        }
    }
    #[inline]
    pub fn height(&self) -> usize {
        self.len
    }
    #[inline]
    fn most_significant_bit_place(x: usize) -> Option<usize> {
        const C: u32 = usize::MAX.count_ones();
        ((C - x.leading_zeros()) as usize).checked_sub(1)
    }
    #[inline]
    pub fn fold_close(&self, l: usize, r: usize) -> S::T {
        debug_assert!(l < self.height());
        debug_assert!(r < self.height());
        debug_assert!(l <= r);
        if let Some(x) = Self::most_significant_bit_place(l ^ r) {
            let offset = self.offsets[x];
            S::operate(&self.table[offset + l], &self.table[offset + r])
        } else {
            self.table[l].clone()
        }
    }
    #[inline]
    pub fn fold(&self, l: usize, r: usize) -> S::T {
        debug_assert!(l < r);
        self.fold_close(l, r - 1)
    }
}

impl<S> Index<usize> for DisjointSparseTable<S>
where
    S: SemiGroup,
{
    type Output = S::T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, ConcatenateOperation, MinOperation},
        tools::Xorshift,
    };
    use std::fmt::Debug;

    fn assert_all_ranges<S>(data: Vec<S::T>)
    where
        S: SemiGroup,
        S::T: Debug + PartialEq,
    {
        let table = DisjointSparseTable::<S>::new(data.clone());
        assert_eq!(table.height(), data.len());
        for (i, x) in data.iter().enumerate() {
            assert_eq!(&table[i], x);
        }
        for l in 0..data.len() {
            let mut expected = data[l].clone();
            assert_eq!(table.fold(l, l + 1), expected);
            assert_eq!(table.fold_close(l, l), expected);
            for r in l + 2..=data.len() {
                expected = S::operate(&expected, &data[r - 1]);
                assert_eq!(table.fold(l, r), expected);
                assert_eq!(table.fold_close(l, r - 1), expected);
            }
        }
    }

    #[test]
    fn test_disjoint_sparse_table_randomized_exhaustive() {
        let mut rng = Xorshift::default();
        let mut sizes = vec![0];
        sizes.extend((0..96).map(|_| rng.random(0usize..=300)));
        for n in sizes {
            let data: Vec<i64> = (0..n).map(|_| rng.random(-1000..=1000)).collect();
            assert_all_ranges::<AdditiveOperation<i64>>(data.clone());
            assert_all_ranges::<MinOperation<i64>>(data);

            let n = n.min(80);
            let data: Vec<Vec<i32>> = (0..n).map(|_| vec![rng.random(-1000..=1000)]).collect();
            assert_all_ranges::<ConcatenateOperation<i32>>(data);
        }
    }
}
