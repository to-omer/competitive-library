use crate::algebra::base::Monoid;
use cargo_snippet::snippet;

#[snippet("SegmentTree")]
#[derive(Clone, Debug)]
pub struct SegmentTree<M: Monoid> {
    n: usize,
    seg: Vec<M::T>,
    m: M,
}
#[snippet("SegmentTree")]
impl<M: Monoid> SegmentTree<M> {
    pub fn new(n: usize, m: M) -> SegmentTree<M> {
        let n = 1 << format!("{:b}", n - 1).len();
        let seg = vec![m.unit(); 2 * n];
        SegmentTree {
            n: n,
            seg: seg,
            m: m,
        }
    }
    pub fn from_vec(v: Vec<M::T>, m: M) -> SegmentTree<M> {
        let n = 1 << format!("{:b}", v.len() - 1).len();
        let mut seg = vec![m.unit(); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[n + i] = x;
        }
        for i in (1..n).rev() {
            seg[i] = m.operate(&seg[2 * i], &seg[2 * i + 1]);
        }
        SegmentTree {
            n: n,
            seg: seg,
            m: m,
        }
    }
    pub fn set(&mut self, k: usize, x: M::T) {
        debug_assert!(k < self.n);
        let mut k = k + self.n;
        self.seg[k] = x;
        k /= 2;
        while k > 0 {
            self.seg[k] = self.m.operate(&self.seg[2 * k], &self.seg[2 * k + 1]);
            k /= 2;
        }
    }
    pub fn update(&mut self, k: usize, x: M::T) {
        debug_assert!(k < self.n);
        let mut k = k + self.n;
        self.seg[k] = self.m.operate(&self.seg[k], &x);
        k /= 2;
        while k > 0 {
            self.seg[k] = self.m.operate(&self.seg[2 * k], &self.seg[2 * k + 1]);
            k /= 2;
        }
    }
    pub fn get(&self, k: usize) -> M::T {
        debug_assert!(k < self.n);
        self.seg[k + self.n].clone()
    }
    pub fn fold(&self, l: usize, r: usize) -> M::T {
        debug_assert!(l < self.n);
        debug_assert!(r <= self.n);
        let mut l = l + self.n;
        let mut r = r + self.n;
        let mut vl = self.m.unit();
        let mut vr = self.m.unit();
        while l < r {
            if l & 1 != 0 {
                vl = self.m.operate(&vl, &self.seg[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = self.m.operate(&self.seg[r], &vr);
            }
            l /= 2;
            r /= 2;
        }
        self.m.operate(&vl, &vr)
    }
    pub fn fold_all(&self) -> M::T {
        self.seg[1].clone()
    }
    /// left most index [0, r) that satisfies monotonic condition
    pub fn lower_bound_all<F: Fn(&M::T) -> bool>(&self, f: F, r: usize) -> usize {
        if !f(&self.seg[1]) {
            return r;
        }
        let mut acc = self.m.unit();
        let mut pos = 1;
        while pos < self.n {
            pos *= 2;
            let y = self.m.operate(&acc, &self.seg[pos]);
            if !f(&y) {
                acc = y;
                pos += 1;
            }
        }
        std::cmp::min(pos - self.n, r)
    }
    /// left most index [l, r) that satisfies monotonic condition
    pub fn lower_bound<F: Fn(&M::T) -> bool>(&self, f: F, l: usize, r: usize) -> usize {
        let mut acc = self.m.unit();
        let mut pos = l + self.n;
        let mut lim = r + self.n;
        loop {
            let y = self.m.operate(&acc, &self.seg[pos]);
            if f(&y) {
                while pos < self.n {
                    pos *= 2;
                    let y = self.m.operate(&acc, &self.seg[pos]);
                    if !f(&y) {
                        acc = y;
                        pos += 1;
                    }
                }
                return std::cmp::min(pos - self.n, r);
            }
            let is_right = pos & 1 == 1;
            if pos == lim {
                return r;
            }
            pos /= 2;
            lim /= 2;
            if is_right {
                acc = y;
                pos += 1;
            }
        }
    }
}

#[test]
fn test_segment_tree() {
    use crate::algebra::operations::{AdditiveOperation, MaxOperation};
    use crate::algorithm::search::lower_bound;
    use crate::tools::random::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_024;
    let q = 10_000;
    let mut seg = SegmentTree::new(n, AdditiveOperation::new());
    let mut arr = vec![0; n + 1];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        seg.set(k, v);
        arr[k + 1] = v;
    }
    for i in 0..n {
        arr[i + 1] += arr[i];
    }
    for i in 0..n {
        for j in i + 1..n + 1 {
            assert_eq!(seg.fold(i, j), arr[j] - arr[i]);
        }
    }
    for _ in 0..q {
        let v = rand.rand(1_000_000_000 * n as u64) as usize;
        assert_eq!(
            seg.lower_bound_all(|&x| v <= x, n),
            lower_bound(&arr[1..], v)
        );
    }
    for _ in 0..q {
        let v = rand.rand(1_000_000_000 * n as u64) as usize;
        let mut l = rand.rand(n as u64) as usize;
        let mut r = rand.rand(n as u64) as usize;
        if l > r {
            std::mem::swap(&mut l, &mut r);
        }
        assert_eq!(
            seg.lower_bound(|&x| v <= x, l, r),
            lower_bound(&arr[l + 1..r + 1], v + arr[l]) + l
        );
    }

    let n = 1_000;
    let mut seg = SegmentTree::new(n, MaxOperation::new());
    let mut arr = vec![0; n];
    for _ in 0..q {
        let k = rand.rand(n as u64) as usize;
        let v = rand.rand(1_000_000_000) as usize;
        seg.set(k, v);
        arr[k] = v;
    }
    for _ in 0..n {
        let l = rand.rand(n as u64) as usize;
        let r = rand.rand(n as u64) as usize + 1;
        let mut res = 0;
        for j in l..r {
            res = std::cmp::max(res, arr[j]);
        }
        assert_eq!(seg.fold(l, r), res);
    }
}

#[snippet("LazySegmentTree")]
/// M: folding Monoid
/// E: lazy folding Monoid
/// F: forced evaluation (Operatior Monoid)
#[derive(Clone, Debug)]
pub struct LazySegmentTree<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> {
    n: usize,
    seg: Vec<M::T>,
    lazy: Vec<E::T>,
    m: M,
    e: E,
    op: F,
}
#[snippet("LazySegmentTree")]
impl<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> LazySegmentTree<M, E, F> {
    pub fn new(n: usize, m: M, e: E, op: F) -> LazySegmentTree<M, E, F> {
        let n = 1 << format!("{:b}", n - 1).len();
        let seg = vec![m.unit(); 2 * n - 1];
        let lazy = vec![e.unit(); 2 * n - 1];
        LazySegmentTree {
            n: n,
            seg: seg,
            lazy: lazy,
            m: m,
            e: e,
            op: op,
        }
    }
    pub fn from_vec(v: Vec<M::T>, m: M, e: E, op: F) -> LazySegmentTree<M, E, F> {
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
            n: n,
            seg: seg,
            lazy: lazy,
            m: m,
            e: e,
            op: op,
        }
    }
    pub fn eval(&mut self, k: usize) {
        if self.lazy[k] != self.e.unit() {
            if k * 2 + 1 < self.n * 2 - 1 {
                self.lazy[2 * k + 1] = self.e.operate(&self.lazy[2 * k + 1], &self.lazy[k]);
                self.lazy[2 * k + 2] = self.e.operate(&self.lazy[2 * k + 2], &self.lazy[k]);
            }
            self.seg[k] = (self.op)(&self.seg[k], &self.lazy[k]);
            self.lazy[k] = self.e.unit();
        }
    }
    fn update_inner(&mut self, l: usize, r: usize, x: E::T, k: usize, a: usize, b: usize) -> M::T {
        self.eval(k);
        if b <= l || r <= a {
            self.seg[k].clone()
        } else if l <= a && b <= r {
            self.lazy[k] = self.e.operate(&self.lazy[k], &x);
            (self.op)(&self.seg[k], &self.lazy[k])
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
