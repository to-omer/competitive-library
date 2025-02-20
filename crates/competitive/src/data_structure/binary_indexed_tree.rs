use super::{Group, Monoid};
use std::fmt::{self, Debug, Formatter};

pub struct BinaryIndexedTree<M>
where
    M: Monoid,
{
    n: usize,
    bit: Vec<M::T>,
}

impl<M> Clone for BinaryIndexedTree<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            bit: self.bit.clone(),
        }
    }
}

impl<M> Debug for BinaryIndexedTree<M>
where
    M: Monoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryIndexedTree")
            .field("n", &self.n)
            .field("bit", &self.bit)
            .finish()
    }
}

impl<M> BinaryIndexedTree<M>
where
    M: Monoid,
{
    #[inline]
    pub fn new(n: usize) -> Self {
        let bit = vec![M::unit(); n + 1];
        Self { n, bit }
    }
    #[inline]
    pub fn from_slice(slice: &[M::T]) -> Self {
        let n = slice.len();
        let mut bit = vec![M::unit(); n + 1];
        for (i, x) in slice.iter().enumerate() {
            let k = i + 1;
            M::operate_assign(&mut bit[k], x);
            let j = k + (k & (!k + 1));
            if j <= n {
                bit[j] = M::operate(&bit[j], &bit[k]);
            }
        }
        Self { n, bit }
    }
    #[inline]
    /// fold [0, k)
    pub fn accumulate0(&self, mut k: usize) -> M::T {
        debug_assert!(k <= self.n);
        let mut res = M::unit();
        while k > 0 {
            res = M::operate(&res, &self.bit[k]);
            k -= k & (!k + 1);
        }
        res
    }
    #[inline]
    /// fold [0, k]
    pub fn accumulate(&self, k: usize) -> M::T {
        self.accumulate0(k + 1)
    }
    #[inline]
    pub fn update(&mut self, k: usize, x: M::T) {
        debug_assert!(k < self.n);
        let mut k = k + 1;
        while k <= self.n {
            self.bit[k] = M::operate(&self.bit[k], &x);
            k += k & (!k + 1);
        }
    }
}

impl<G: Group> BinaryIndexedTree<G> {
    #[inline]
    pub fn fold(&self, l: usize, r: usize) -> G::T {
        debug_assert!(l <= self.n && r <= self.n);
        G::operate(&G::inverse(&self.accumulate0(l)), &self.accumulate0(r))
    }
    #[inline]
    pub fn get(&self, k: usize) -> G::T {
        self.fold(k, k + 1)
    }
    #[inline]
    pub fn set(&mut self, k: usize, x: G::T) {
        self.update(k, G::operate(&G::inverse(&self.get(k)), &x));
    }
}

impl<M: Monoid> BinaryIndexedTree<M>
where
    M::T: Ord,
{
    #[inline]
    pub fn lower_bound(&self, x: M::T) -> usize {
        let n = self.n;
        let mut acc = M::unit();
        let mut pos = 0;
        let mut k = n.next_power_of_two();
        while k > 0 {
            if k + pos <= n && M::operate(&acc, &self.bit[k + pos]) < x {
                pos += k;
                acc = M::operate(&acc, &self.bit[pos]);
            }
            k >>= 1;
        }
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, MaxOperation},
        algorithm::SliceBisectExt as _,
        tools::Xorshift,
    };

    const N: usize = 10_000;
    const Q: usize = 100_000;
    const A: u64 = 1_000_000_000;
    const B: i64 = 1_000_000_000;

    #[test]
    fn test_binary_indexed_tree() {
        let mut rng = Xorshift::new();
        let mut arr: Vec<_> = rng.random_iter(..A).take(N).collect();
        let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::from_slice(&arr);
        for (k, v) in rng.random_iter((..N, ..A)).take(Q) {
            bit.update(k, v);
            arr[k] += v;
        }
        for i in 0..N - 1 {
            arr[i + 1] += arr[i];
        }
        for (i, a) in arr.iter().cloned().enumerate() {
            assert_eq!(bit.accumulate(i), a);
        }

        let mut arr: Vec<_> = rng.random_iter(..A).take(N).collect();
        let mut bit = BinaryIndexedTree::<MaxOperation<_>>::from_slice(&arr);
        for (k, v) in rng.random_iter((..N, ..A)).take(Q) {
            bit.update(k, v);
            arr[k] = std::cmp::max(arr[k], v);
        }
        for i in 0..N - 1 {
            arr[i + 1] = std::cmp::max(arr[i], arr[i + 1]);
        }
        for (i, a) in arr.iter().cloned().enumerate() {
            assert_eq!(bit.accumulate(i), a);
        }
    }

    #[test]
    fn test_group_binary_indexed_tree() {
        const N: usize = 2_000;
        let mut rng = Xorshift::new();
        let mut arr: Vec<_> = rng.random_iter(-B..B).take(N).collect();
        let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::from_slice(&arr);
        for (k, v) in rng.random_iter((..N, -B..B)).take(Q) {
            bit.set(k, v);
            arr[k] = v;
        }
        for i in 0..N - 1 {
            arr[i + 1] += arr[i];
        }
        for i in 0..N {
            for j in i + 1..N + 1 {
                assert_eq!(
                    bit.fold(i, j),
                    arr[j - 1] - if i == 0 { 0 } else { arr[i - 1] }
                );
            }
        }
    }

    #[test]
    fn test_binary_indexed_tree_lower_bound() {
        let mut rng = Xorshift::new();
        let mut arr: Vec<_> = rng.random_iter(1..B).take(N).collect();
        let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::from_slice(&arr);
        for (k, v) in rng.random_iter((..N, 1..B)).take(Q) {
            bit.set(k, v);
            arr[k] = v;
        }
        for i in 0..N - 1 {
            arr[i + 1] += arr[i];
        }
        for x in rng.random_iter(1..B * N as i64).take(Q) {
            assert_eq!(bit.lower_bound(x), arr.position_bisect(|&a| a >= x));
        }
    }
}
