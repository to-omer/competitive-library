use super::SemiGroup;
use std::{
    fmt::{self, Debug, Formatter},
    ops::Index,
};

pub struct DisjointSparseTable<S>
where
    S: SemiGroup,
{
    table: Vec<Vec<S::T>>,
}

impl<S> Clone for DisjointSparseTable<S>
where
    S: SemiGroup,
{
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
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
            .finish()
    }
}

impl<S> DisjointSparseTable<S>
where
    S: SemiGroup,
{
    pub fn new(v: Vec<S::T>) -> Self {
        let n = v.len();
        let mut table = vec![v];
        let mut k = 2;
        while k < n {
            let mut v = table[0].clone();
            for i in (0..n).step_by(k * 2) {
                for j in (i..n.min(i + k) - 1).rev() {
                    v[j] = S::operate(&v[j], &v[j + 1]);
                }
                for j in i + k + 1..n.min(i + k * 2) {
                    v[j] = S::operate(&v[j - 1], &v[j]);
                }
            }
            table.push(v);
            k *= 2;
        }
        Self { table }
    }
    #[inline]
    pub fn height(&self) -> usize {
        self.table[0].len()
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
            S::operate(&self.table[x][l], &self.table[x][r])
        } else {
            self.table[0][l].clone()
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
        &self.table[0][index]
    }
}
