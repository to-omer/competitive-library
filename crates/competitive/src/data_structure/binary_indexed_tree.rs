use super::{Group, Monoid};

#[derive(Clone, Debug)]
pub struct BinaryIndexedTree<M: Monoid> {
    n: usize,
    bit: Vec<M::T>,
}

impl<M: Monoid> BinaryIndexedTree<M> {
    #[inline]
    pub fn new(n: usize) -> Self {
        let bit = vec![M::unit(); n + 1];
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
        debug_assert!(l < self.n && 0 < r && r <= self.n);
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
        let mut rng = Xorshift::time();
        let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(N);
        let mut arr = vec![0; N];
        for (k, v) in rng.gen_iter((..N, ..A)).take(Q) {
            bit.update(k, v);
            arr[k] += v;
        }
        for i in 0..N - 1 {
            arr[i + 1] += arr[i];
        }
        for (i, a) in arr.iter().cloned().enumerate() {
            assert_eq!(bit.accumulate(i), a);
        }

        let mut bit = BinaryIndexedTree::<MaxOperation<_>>::new(N);
        let mut arr = vec![0; N];
        for (k, v) in rng.gen_iter((..N, ..A)).take(Q) {
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
        let mut rng = Xorshift::time();
        let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(N);
        let mut arr = vec![0; N + 1];
        for (k, v) in rng.gen_iter((..N, -B..B)).take(Q) {
            bit.set(k, v);
            arr[k + 1] = v;
        }
        for i in 0..N {
            arr[i + 1] += arr[i];
        }
        for i in 0..N {
            for j in i + 1..N + 1 {
                assert_eq!(bit.fold(i, j), arr[j] - arr[i]);
            }
        }
    }

    #[test]
    fn test_binary_indexed_tree_lower_bound() {
        let mut rng = Xorshift::time();
        let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(N);
        let mut arr = vec![0; N];
        for (k, v) in rng.gen_iter((..N, 1..B)).take(Q) {
            bit.set(k, v);
            arr[k] = v;
        }
        for i in 0..N - 1 {
            arr[i + 1] += arr[i];
        }
        for x in rng.gen_iter(1..B * N as i64).take(Q) {
            assert_eq!(bit.lower_bound(x), arr.position_bisect(|&a| a >= x));
        }
    }
}
