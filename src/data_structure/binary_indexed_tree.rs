use crate::algebra::magma::{Group, Monoid};

#[cargo_snippet::snippet("BinaryIndexedTree")]
#[derive(Clone, Debug)]
pub struct BinaryIndexedTree<M: Monoid> {
    bit: Vec<M::T>,
    monoid: M,
}
#[cargo_snippet::snippet("BinaryIndexedTree")]
impl<M: Monoid> BinaryIndexedTree<M> {
    #[inline]
    pub fn new(n: usize, monoid: M) -> BinaryIndexedTree<M> {
        let bit = vec![monoid.unit(); n + 1];
        BinaryIndexedTree {
            bit: bit,
            monoid: monoid,
        }
    }
    #[inline]
    pub fn ident(&self) -> M::T {
        self.monoid.unit()
    }
    #[inline]
    pub fn operate(&self, x: &M::T, y: &M::T) -> M::T {
        self.monoid.operate(x, y)
    }
    #[inline]
    /// 0-indexed [1, k)
    pub fn accumulate(&self, k: usize) -> M::T {
        let mut res = self.ident();
        let mut k = k;
        while k > 0 {
            res = self.operate(&res, &self.bit[k]);
            k -= k & !k + 1;
        }
        res
    }
    #[inline]
    /// 1-indexed
    pub fn update(&mut self, k: usize, x: M::T) {
        assert!(k > 0);
        let mut k = k;
        while k < self.bit.len() {
            self.bit[k] = self.operate(&self.bit[k], &x);
            k += k & !k + 1;
        }
    }
}

#[test]
fn test_binary_indexed_tree() {
    use crate::algebra::operations::{AdditiveOperation, MaxOperation};
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    let n = 10_000;
    let q = 100_000;
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(k + 1, v);
        arr[k] += v;
    }
    for i in 0..n - 1 {
        arr[i + 1] += arr[i];
    }
    for i in 0..n {
        assert_eq!(bit.accumulate(i + 1), arr[i]);
    }

    let mut bit = BinaryIndexedTree::new(n, MaxOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(k + 1, v);
        arr[k] = std::cmp::max(arr[k], v);
    }
    for i in 0..n - 1 {
        arr[i + 1] = std::cmp::max(arr[i], arr[i + 1]);
    }
    for i in 0..n {
        assert_eq!(bit.accumulate(i + 1), arr[i]);
    }
}

#[cargo_snippet::snippet("BinaryIndexedTree")]
impl<G: Group> BinaryIndexedTree<G> {
    #[inline]
    pub fn inverse(&self, x: &G::T) -> G::T {
        self.monoid.inverse(x)
    }
    #[inline]
    /// 0-indexed [l, r)
    pub fn fold(&self, l: usize, r: usize) -> G::T {
        self.operate(&self.inverse(&self.accumulate(l)), &self.accumulate(r))
    }
    #[inline]
    /// 1-indexed
    pub fn get(&self, k: usize) -> G::T {
        self.fold(k - 1, k)
    }
    #[inline]
    /// 1-indexed
    pub fn set(&mut self, k: usize, x: G::T) {
        let y = self.inverse(&self.get(k));
        let z = self.operate(&y, &x);
        self.update(k, z);
    }
}

#[test]
fn test_group_binary_indexed_tree() {
    use crate::algebra::operations::AdditiveOperation;
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_000;
    let q = 10_000;
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n + 1];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(2_000_000_000) as i64 - 1_000_000_000;
        bit.set(k + 1, v);
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
    /// 1-indexed
    pub fn lower_bound(&self, x: M::T) -> usize {
        let n = self.bit.len() - 1;
        let mut acc = self.ident();
        let mut pos = 0;
        let mut k = 1 << format!("{:b}", n).len();
        while k > 0 {
            if k + pos <= n && self.operate(&acc, &self.bit[k + pos]) < x {
                pos += k;
                acc = self.operate(&acc, &self.bit[pos]);
            }
            k >>= 1;
        }
        pos + 1
    }
}

#[test]
fn test_binary_indexed_tree_lower_bound() {
    use crate::algebra::operations::AdditiveOperation;
    use crate::algorithm::search::lower_bound;
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_000;
    let q = 10_000;
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as i64;
        bit.set(k + 1, v);
        arr[k] = v;
    }
    for i in 0..n - 1 {
        arr[i + 1] += arr[i];
    }
    for _ in 0..n {
        let x = rand.rand(5_000_000_000_000) as i64;
        assert_eq!(bit.lower_bound(x), lower_bound(&arr, x) + 1);
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
    pub fn new(h: usize, w: usize, m: M) -> BinaryIndexedTree2D<M> {
        let bit = vec![vec![m.unit(); w + 1]; h + 1];
        BinaryIndexedTree2D {
            h: h,
            w: w,
            bit: bit,
            m: m,
        }
    }
    /// 0-indexed [0, i) x [0, j)
    pub fn accumulate(&self, i: usize, j: usize) -> M::T {
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
    /// 1-indexed
    pub fn update(&mut self, i: usize, j: usize, x: M::T) {
        let mut a = i;
        while a <= self.h {
            let mut b = j;
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
    use crate::algebra::operations::{AdditiveOperation, MaxOperation};
    use crate::tools::random::Xorshift;
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
        bit.update(i + 1, j + 1, v);
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
            assert_eq!(bit.accumulate(i + 1, j + 1), arr[i][j]);
        }
    }

    let mut bit = BinaryIndexedTree2D::new(h, w, MaxOperation::new());
    let mut arr = vec![vec![0; w]; h];
    for _ in 0..q {
        let i = rand.rand(h as u64) as usize;
        let j = rand.rand(w as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        bit.update(i + 1, j + 1, v);
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
            assert_eq!(bit.accumulate(i + 1, j + 1), arr[i][j]);
        }
    }
}

#[cargo_snippet::snippet("BinaryIndexedTree2D")]
impl<G: Group> BinaryIndexedTree2D<G> {
    /// 0-indexed [i1, i2) x [j1, j2)
    pub fn fold(&self, i1: usize, j1: usize, i2: usize, j2: usize) -> G::T {
        let mut res = self.m.unit();
        res = self.m.operate(&res, &self.accumulate(i1, j1));
        res = self
            .m
            .operate(&res, &self.m.inverse(&self.accumulate(i1, j2)));
        res = self
            .m
            .operate(&res, &self.m.inverse(&self.accumulate(i2, j1)));
        res = self.m.operate(&res, &self.accumulate(i2, j2));
        res
    }
    /// 1-indexed
    pub fn get(&self, i: usize, j: usize) -> G::T {
        self.fold(i - 1, j - 1, i, j)
    }
    /// 1-indexed
    pub fn set(&mut self, i: usize, j: usize, x: G::T) {
        let y = self.m.inverse(&self.get(i, j));
        let z = self.m.operate(&y, &x);
        self.update(i, j, z);
    }
}

#[test]
fn test_group_binary_indexed_tree2d() {
    use crate::algebra::operations::AdditiveOperation;
    use crate::tools::random::Xorshift;
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
        bit.set(i + 1, j + 1, v);
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
