use crate::algebra::{Group, Monoid};

#[codesnip::entry("BinaryIndexedTree", include("algebra"))]
#[derive(Clone, Debug)]
pub struct BinaryIndexedTree<M: Monoid> {
    n: usize,
    bit: Vec<M::T>,
}
#[codesnip::entry("BinaryIndexedTree")]
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

#[codesnip::entry("BinaryIndexedTree")]
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

#[codesnip::entry("BinaryIndexedTree")]
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

#[codesnip::entry("BinaryIndexedTree2D", include("algebra"))]
#[derive(Clone, Debug)]
pub struct BinaryIndexedTree2D<M: Monoid> {
    h: usize,
    w: usize,
    bit: Vec<Vec<M::T>>,
}
#[codesnip::entry("BinaryIndexedTree2D")]
impl<M: Monoid> BinaryIndexedTree2D<M> {
    #[inline]
    pub fn new(h: usize, w: usize) -> Self {
        let bit = vec![vec![M::unit(); w + 1]; h + 1];
        Self { h, w, bit }
    }
    #[inline]
    /// fold [0, i) x [0, j)
    pub fn accumulate0(&self, i: usize, j: usize) -> M::T {
        let mut res = M::unit();
        let mut a = i;
        while a > 0 {
            let mut b = j;
            while b > 0 {
                res = M::operate(&res, &self.bit[a][b]);
                b -= b & (!b + 1);
            }
            a -= a & (!a + 1);
        }
        res
    }
    #[inline]
    /// fold [0, i] x [0, j]
    pub fn accumulate(&self, i: usize, j: usize) -> M::T {
        self.accumulate0(i + 1, j + 1)
    }
    #[inline]
    pub fn update(&mut self, i: usize, j: usize, x: M::T) {
        let mut a = i + 1;
        while a <= self.h {
            let mut b = j + 1;
            while b <= self.w {
                self.bit[a][b] = M::operate(&self.bit[a][b], &x);
                b += b & (!b + 1);
            }
            a += a & (!a + 1);
        }
    }
}

#[codesnip::entry("BinaryIndexedTree2D")]
impl<G: Group> BinaryIndexedTree2D<G> {
    #[inline]
    /// 0-indexed [i1, i2) x [j1, j2)
    pub fn fold(&self, i1: usize, j1: usize, i2: usize, j2: usize) -> G::T {
        let mut res = G::unit();
        res = G::operate(&res, &self.accumulate0(i1, j1));
        res = G::rinv_operate(&res, &self.accumulate0(i1, j2));
        res = G::rinv_operate(&res, &self.accumulate0(i2, j1));
        res = G::operate(&res, &self.accumulate0(i2, j2));
        res
    }
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> G::T {
        self.fold(i, j, i + 1, j + 1)
    }
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, x: G::T) {
        let y = G::inverse(&self.get(i, j));
        let z = G::operate(&y, &x);
        self.update(i, j, z);
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

    #[test]
    fn test_binary_indexed_tree_2d() {
        let mut rng = Xorshift::time();
        const H: usize = 150;
        const W: usize = 250;
        let mut bit = BinaryIndexedTree2D::<AdditiveOperation<_>>::new(H, W);
        let mut arr = vec![vec![0; W]; H];
        for (i, j, v) in rng.gen_iter((..H, ..W, ..A)).take(Q) {
            bit.update(i, j, v);
            arr[i][j] += v;
        }
        for arr in arr.iter_mut() {
            for j in 0..W - 1 {
                arr[j + 1] += arr[j];
            }
        }
        for i in 0..H - 1 {
            for j in 0..W {
                arr[i + 1][j] += arr[i][j];
            }
        }
        for (i, arr) in arr.iter().enumerate() {
            for (j, a) in arr.iter().cloned().enumerate() {
                assert_eq!(bit.accumulate(i, j), a);
            }
        }

        let mut bit = BinaryIndexedTree2D::<MaxOperation<_>>::new(H, W);
        let mut arr = vec![vec![0; W]; H];
        for (i, j, v) in rng.gen_iter((..H, ..W, ..A)).take(Q) {
            bit.update(i, j, v);
            arr[i][j] = std::cmp::max(arr[i][j], v);
        }
        for arr in arr.iter_mut() {
            for j in 0..W - 1 {
                arr[j + 1] = std::cmp::max(arr[j + 1], arr[j]);
            }
        }
        for i in 0..H - 1 {
            for j in 0..W {
                arr[i + 1][j] = std::cmp::max(arr[i + 1][j], arr[i][j]);
            }
        }
        for (i, arr) in arr.iter().enumerate() {
            for (j, a) in arr.iter().cloned().enumerate() {
                assert_eq!(bit.accumulate(i, j), a);
            }
        }
    }

    #[test]
    fn test_group_binary_indexed_tree2d() {
        let mut rng = Xorshift::time();
        const H: usize = 15;
        const W: usize = 25;
        let mut bit = BinaryIndexedTree2D::<AdditiveOperation<_>>::new(H, W);
        let mut arr = vec![vec![0; W + 1]; H + 1];
        for (i, j, v) in rng.gen_iter((..H, ..W, -B..B)).take(Q) {
            bit.set(i, j, v);
            arr[i + 1][j + 1] = v;
        }
        for arr in arr.iter_mut() {
            for j in 0..W {
                arr[j + 1] += arr[j];
            }
        }
        for i in 0..H {
            for j in 0..W + 1 {
                arr[i + 1][j] += arr[i][j];
            }
        }
        for i1 in 0..H {
            for i2 in i1 + 1..H + 1 {
                for j1 in 0..W {
                    for j2 in j1 + 1..W + 1 {
                        assert_eq!(
                            bit.fold(i1, j1, i2, j2),
                            arr[i2][j2] - arr[i2][j1] - arr[i1][j2] + arr[i1][j1]
                        );
                    }
                }
            }
        }
    }
}
