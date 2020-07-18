use crate::algebra::{Group, Monoid};

#[cargo_snippet::snippet("BinaryIndexedTree")]
#[derive(Clone, Debug)]
pub struct BinaryIndexedTree<M: Monoid> {
    n: usize,
    bit: Vec<M::T>,
    m: M,
}
#[cargo_snippet::snippet("BinaryIndexedTree")]
impl<M: Monoid> BinaryIndexedTree<M> {
    #[inline]
    pub fn new(n: usize, m: M) -> Self {
        let bit = vec![m.unit(); n + 1];
        Self { n, bit, m }
    }
    #[inline]
    /// fold [0, k)
    pub fn accumulate0(&self, mut k: usize) -> M::T {
        debug_assert!(k <= self.n);
        let mut res = self.m.unit();
        while k > 0 {
            res = self.m.operate(&res, &self.bit[k]);
            k -= k & !k + 1;
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
            self.bit[k] = self.m.operate(&self.bit[k], &x);
            k += k & !k + 1;
        }
    }
}

#[test]
fn test_binary_indexed_tree() {
    use crate::algebra::{AdditiveOperation, MaxOperation};
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    let n = 10_000;
    let q = 100_000;
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(k, v);
        arr[k] += v;
    }
    for i in 0..n - 1 {
        arr[i + 1] += arr[i];
    }
    for i in 0..n {
        assert_eq!(bit.accumulate(i), arr[i]);
    }

    let mut bit = BinaryIndexedTree::new(n, MaxOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(k, v);
        arr[k] = std::cmp::max(arr[k], v);
    }
    for i in 0..n - 1 {
        arr[i + 1] = std::cmp::max(arr[i], arr[i + 1]);
    }
    for i in 0..n {
        assert_eq!(bit.accumulate(i), arr[i]);
    }
}

#[cargo_snippet::snippet("BinaryIndexedTree")]
impl<G: Group> BinaryIndexedTree<G> {
    #[inline]
    pub fn fold(&self, l: usize, r: usize) -> G::T {
        debug_assert!(l < self.n && 0 < r && r <= self.n);
        self.m
            .operate(&self.m.inverse(&self.accumulate0(l)), &self.accumulate0(r))
    }
    #[inline]
    pub fn get(&self, k: usize) -> G::T {
        self.fold(k, k + 1)
    }
    #[inline]
    pub fn set(&mut self, k: usize, x: G::T) {
        let y = self.m.inverse(&self.get(k));
        self.update(k, self.m.operate(&y, &x));
    }
}

#[test]
fn test_group_binary_indexed_tree() {
    use crate::algebra::AdditiveOperation;
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_000;
    let q = 10_000;
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n + 1];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(2_000_000_000) as i64 - 1_000_000_000;
        bit.set(k, v);
        arr[k + 1] = v;
    }
    for i in 0..n {
        arr[i + 1] += arr[i];
    }
    for i in 0..n {
        for j in i + 1..n + 1 {
            assert_eq!(bit.fold(i, j), arr[j] - arr[i]);
        }
    }
}

#[cargo_snippet::snippet("BinaryIndexedTree")]
impl<M: Monoid> BinaryIndexedTree<M>
where
    M::T: Ord,
{
    #[inline]
    pub fn lower_bound(&self, x: M::T) -> usize {
        let n = self.n;
        let mut acc = self.m.unit();
        let mut pos = 0;
        let mut k = 1 << format!("{:b}", n).len();
        while k > 0 {
            if k + pos <= n && self.m.operate(&acc, &self.bit[k + pos]) < x {
                pos += k;
                acc = self.m.operate(&acc, &self.bit[pos]);
            }
            k >>= 1;
        }
        pos
    }
}

#[test]
fn test_binary_indexed_tree_lower_bound() {
    use crate::algebra::AdditiveOperation;
    use crate::algorithm::lower_bound;
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_000;
    let q = 10_000;
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as i64;
        bit.set(k, v);
        arr[k] = v;
    }
    for i in 0..n - 1 {
        arr[i + 1] += arr[i];
    }
    for _ in 0..n {
        let x = rand.rand(5_000_000_000_000) as i64;
        assert_eq!(bit.lower_bound(x), lower_bound(&arr, x));
    }
}

#[cargo_snippet::snippet("BinaryIndexedTree2D")]
#[derive(Clone, Debug)]
pub struct BinaryIndexedTree2D<M: Monoid> {
    h: usize,
    w: usize,
    bit: Vec<Vec<M::T>>,
    m: M,
}
#[cargo_snippet::snippet("BinaryIndexedTree2D")]
impl<M: Monoid> BinaryIndexedTree2D<M> {
    #[inline]
    pub fn new(h: usize, w: usize, m: M) -> Self {
        let bit = vec![vec![m.unit(); w + 1]; h + 1];
        Self { h, w, bit, m }
    }
    #[inline]
    /// fold [0, i) x [0, j)
    pub fn accumulate0(&self, i: usize, j: usize) -> M::T {
        let mut res = self.m.unit();
        let mut a = i;
        while a > 0 {
            let mut b = j;
            while b > 0 {
                res = self.m.operate(&res, &self.bit[a][b]);
                b -= b & !b + 1;
            }
            a -= a & !a + 1;
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
                self.bit[a][b] = self.m.operate(&self.bit[a][b], &x);
                b += b & !b + 1;
            }
            a += a & !a + 1;
        }
    }
}

#[test]
fn test_binary_indexed_tree_2d() {
    use crate::algebra::{AdditiveOperation, MaxOperation};
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    let h = 200;
    let w = 200;
    let q = 100_000;
    let mut bit = BinaryIndexedTree2D::new(h, w, AdditiveOperation::new());
    let mut arr = vec![vec![0; w]; h];
    for _ in 0..q {
        let i = rand.rand(h as u64) as usize;
        let j = rand.rand(w as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(i, j, v);
        arr[i][j] += v;
    }
    for i in 0..h {
        for j in 0..w - 1 {
            arr[i][j + 1] += arr[i][j];
        }
    }
    for i in 0..h - 1 {
        for j in 0..w {
            arr[i + 1][j] += arr[i][j];
        }
    }
    for i in 0..h {
        for j in 0..w {
            assert_eq!(bit.accumulate(i, j), arr[i][j]);
        }
    }

    let mut bit = BinaryIndexedTree2D::new(h, w, MaxOperation::new());
    let mut arr = vec![vec![0; w]; h];
    for _ in 0..q {
        let i = rand.rand(h as u64) as usize;
        let j = rand.rand(w as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(i, j, v);
        arr[i][j] = std::cmp::max(arr[i][j], v);
    }
    for i in 0..h {
        for j in 0..w - 1 {
            arr[i][j + 1] = std::cmp::max(arr[i][j + 1], arr[i][j]);
        }
    }
    for i in 0..h - 1 {
        for j in 0..w {
            arr[i + 1][j] = std::cmp::max(arr[i + 1][j], arr[i][j]);
        }
    }
    for i in 0..h {
        for j in 0..w {
            assert_eq!(bit.accumulate(i, j), arr[i][j]);
        }
    }
}

#[cargo_snippet::snippet("BinaryIndexedTree2D")]
impl<G: Group> BinaryIndexedTree2D<G> {
    #[inline]
    /// 0-indexed [i1, i2) x [j1, j2)
    pub fn fold(&self, i1: usize, j1: usize, i2: usize, j2: usize) -> G::T {
        let mut res = self.m.unit();
        res = self.m.operate(&res, &self.accumulate0(i1, j1));
        res = self
            .m
            .operate(&res, &self.m.inverse(&self.accumulate0(i1, j2)));
        res = self
            .m
            .operate(&res, &self.m.inverse(&self.accumulate0(i2, j1)));
        res = self.m.operate(&res, &self.accumulate0(i2, j2));
        res
    }
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> G::T {
        self.fold(i, j, i + 1, j + 1)
    }
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, x: G::T) {
        let y = self.m.inverse(&self.get(i, j));
        let z = self.m.operate(&y, &x);
        self.update(i, j, z);
    }
}

#[test]
fn test_group_binary_indexed_tree2d() {
    use crate::algebra::AdditiveOperation;
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    let h = 20;
    let w = 20;
    let q = 100_000;
    let mut bit = BinaryIndexedTree2D::new(h, w, AdditiveOperation::new());
    let mut arr = vec![vec![0; w + 1]; h + 1];
    for _ in 0..q {
        let i = rand.rand(h as u64) as usize;
        let j = rand.rand(w as u64) as usize;
        let v = rand.rand(2_000_000_000) as i64 - 1_000_000_000;
        bit.set(i, j, v);
        arr[i + 1][j + 1] = v;
    }
    for i in 0..h + 1 {
        for j in 0..w {
            arr[i][j + 1] += arr[i][j];
        }
    }
    for i in 0..h {
        for j in 0..w + 1 {
            arr[i + 1][j] += arr[i][j];
        }
    }
    for i1 in 0..h {
        for i2 in i1 + 1..h + 1 {
            for j1 in 0..w {
                for j2 in j1 + 1..w + 1 {
                    assert_eq!(
                        bit.fold(i1, j1, i2, j2),
                        arr[i2][j2] - arr[i2][j1] - arr[i1][j2] + arr[i1][j1]
                    );
                }
            }
        }
    }
}
