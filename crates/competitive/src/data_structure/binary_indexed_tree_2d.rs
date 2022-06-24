use super::{Group, Monoid};
use std::fmt::{self, Debug, Formatter};

pub struct BinaryIndexedTree2D<M>
where
    M: Monoid,
{
    h: usize,
    w: usize,
    bit: Vec<Vec<M::T>>,
}

impl<M> Clone for BinaryIndexedTree2D<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            h: self.h,
            w: self.w,
            bit: self.bit.clone(),
        }
    }
}

impl<M> Debug for BinaryIndexedTree2D<M>
where
    M: Monoid,
    M::T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryIndexedTree2D")
            .field("h", &self.h)
            .field("w", &self.w)
            .field("bit", &self.bit)
            .finish()
    }
}

impl<M> BinaryIndexedTree2D<M>
where
    M: Monoid,
{
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

impl<G> BinaryIndexedTree2D<G>
where
    G: Group,
{
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
        tools::Xorshift,
    };

    const Q: usize = 100_000;
    const A: u64 = 1_000_000_000;
    const B: i64 = 1_000_000_000;

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
