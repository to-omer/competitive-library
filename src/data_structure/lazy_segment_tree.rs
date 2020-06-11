use crate::algebra::magma::Monoid;

#[cargo_snippet::snippet("LazySegmentTree")]
/// M: folding Monoid
/// E: lazy Monoid
/// F: lazy evaluating
#[derive(Clone, Debug)]
pub struct LazySegmentTree<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> {
    n: usize,
    seg: Vec<M::T>,
    lazy: Vec<E::T>,
    m: M,
    e: E,
    f: F,
}
#[cargo_snippet::snippet("LazySegmentTree")]
impl<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> LazySegmentTree<M, E, F> {
    pub fn new(n: usize, m: M, e: E, f: F) -> Self {
        let n = 1 << format!("{:b}", n - 1).len();
        let seg = vec![m.unit(); 2 * n - 1];
        let lazy = vec![e.unit(); 2 * n - 1];
        LazySegmentTree {
            n,
            seg,
            lazy,
            m,
            e,
            f,
        }
    }
    pub fn from_vec(v: Vec<M::T>, m: M, e: E, f: F) -> Self {
        let n = 1 << format!("{:b}", v.len() - 1).len();
        let mut seg = vec![m.unit(); 2 * n - 1];
        for (i, x) in v.into_iter().enumerate() {
            seg[i + n - 1] = x;
        }
        for i in (0..n - 1).rev() {
            seg[i] = m.operate(&seg[2 * i + 1], &seg[2 * i + 2]);
        }
        let lazy = vec![e.unit(); 2 * n - 1];
        LazySegmentTree {
            n,
            seg,
            lazy,
            m,
            e,
            f,
        }
    }
    pub fn eval(&mut self, k: usize) {
        if self.lazy[k] != self.e.unit() {
            if k * 2 + 1 < self.n * 2 - 1 {
                self.lazy[2 * k + 1] = self.e.operate(&self.lazy[2 * k + 1], &self.lazy[k]);
                self.lazy[2 * k + 2] = self.e.operate(&self.lazy[2 * k + 2], &self.lazy[k]);
            }
            self.seg[k] = (self.f)(&self.seg[k], &self.lazy[k]);
            self.lazy[k] = self.e.unit();
        }
    }
    fn update_inner(&mut self, l: usize, r: usize, x: E::T, k: usize, a: usize, b: usize) -> M::T {
        self.eval(k);
        if b <= l || r <= a {
            self.seg[k].clone()
        } else if l <= a && b <= r {
            self.lazy[k] = self.e.operate(&self.lazy[k], &x);
            (self.f)(&self.seg[k], &self.lazy[k])
        } else {
            let lx = self.update_inner(l, r, x.clone(), k * 2 + 1, a, (a + b) / 2);
            let rx = self.update_inner(l, r, x, k * 2 + 2, (a + b) / 2, b);
            let res = self.m.operate(&lx, &rx);
            self.seg[k] = res.clone();
            res
        }
    }
    pub fn update(&mut self, l: usize, r: usize, x: E::T) -> M::T {
        let n = self.n;
        self.update_inner(l, r, x, 0, 0, n)
    }
    fn fold_inner(&mut self, l: usize, r: usize, k: usize, a: usize, b: usize) -> M::T {
        self.eval(k);
        if b <= l || r <= a {
            self.m.unit()
        } else if l <= a && b <= r {
            self.seg[k].clone()
        } else {
            let lx = self.fold_inner(l, r, k * 2 + 1, a, (a + b) / 2);
            let rx = self.fold_inner(l, r, k * 2 + 2, (a + b) / 2, b);
            self.m.operate(&lx, &rx)
        }
    }
    pub fn fold(&mut self, l: usize, r: usize) -> M::T {
        let n = self.n;
        self.fold_inner(l, r, 0, 0, n)
    }
}

#[test]
fn test_lazy_segment_tree() {
    use crate::algebra::operations::{
        AdditiveOperation, CartesianOperation, LastOperation, MaxOperation,
    };
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_024;
    let m = 20_000;
    // Range Sum Query & Range Add Query
    let mut seg = LazySegmentTree::from_vec(
        vec![(0, 1); n],
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AdditiveOperation::new(),
        |x, &y| (x.0 + y * x.1, x.1),
    );
    let mut arr = vec![0; n];
    for _ in 0..m {
        let q = rand.rand(2);
        if q == 0 {
            // Range Add Query
            let mut l = rand.rand(n as u64) as usize;
            let mut r = rand.rand(n as u64) as usize + 1;
            if r < l {
                std::mem::swap(&mut l, &mut r);
            }
            let x = rand.rand(1_000_000_000) as usize;
            seg.update(l, r, x);
            for i in l..r {
                arr[i] += x;
            }
        } else {
            // Range Sum Query
            let mut l = rand.rand(n as u64) as usize;
            let mut r = rand.rand(n as u64) as usize + 1;
            if r < l {
                std::mem::swap(&mut l, &mut r);
            }
            let mut res = 0;
            for i in l..r {
                res += arr[i];
            }
            assert_eq!(seg.fold(l, r).0, res);
        }
    }

    // Range Max Query & Range Update Query
    let mut seg = LazySegmentTree::new(n, MaxOperation::new(), LastOperation::new(), |&x, y| {
        y.unwrap_or(x)
    });
    let mut arr = vec![0; n];
    for _ in 0..m {
        let q = rand.rand(2);
        if q == 0 {
            // Range Update Query
            let l = rand.rand(n as u64) as usize;
            let r = rand.rand(n as u64) as usize + 1;
            let x = rand.rand(1_000_000_000) as usize;
            seg.update(l, r, Some(x));
            for i in l..r {
                arr[i] = x;
            }
        } else {
            // Range Max Query
            let l = rand.rand(n as u64) as usize;
            let r = rand.rand(n as u64) as usize + 1;
            let mut res = 0;
            for i in l..r {
                res = std::cmp::max(res, arr[i]);
            }
            assert_eq!(seg.fold(l, r), res);
        }
    }
}
