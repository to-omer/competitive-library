use crate::algebra::Monoid;

#[codesnip::entry("LazySegmentTree")]
/// M: folding Monoid
/// E: lazy Monoid
/// F: lazy evaluating
#[derive(Clone, Debug)]
pub struct LazySegmentTree<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> {
    n: usize,
    height: usize,
    seg: Vec<M::T>,
    lazy: Vec<E::T>,
    m: M,
    e: E,
    f: F,
}
#[codesnip::entry("LazySegmentTree")]
impl<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> LazySegmentTree<M, E, F> {
    pub fn new(n: usize, m: M, e: E, f: F) -> Self {
        let height = format!("{:b}", n - 1).len();
        let n = 1 << height;
        let seg = vec![m.unit(); 2 * n];
        let lazy = vec![e.unit(); 2 * n];
        Self {
            n,
            height,
            seg,
            lazy,
            m,
            e,
            f,
        }
    }
    pub fn from_vec(v: Vec<M::T>, m: M, e: E, f: F) -> Self {
        let height = format!("{:b}", v.len() - 1).len();
        let n = 1 << height;
        let mut seg = vec![m.unit(); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[i + n] = x;
        }
        for i in (1..n).rev() {
            seg[i] = m.operate(&seg[2 * i], &seg[2 * i + 1]);
        }
        let lazy = vec![e.unit(); 2 * n];
        Self {
            n,
            height,
            seg,
            lazy,
            m,
            e,
            f,
        }
    }
    #[inline]
    fn propagate(&mut self, k: usize) {
        debug_assert!(k < self.n);
        if self.lazy[k] != self.e.unit() {
            self.lazy[2 * k] = self.e.operate(&self.lazy[2 * k], &self.lazy[k]);
            self.lazy[2 * k + 1] = self.e.operate(&self.lazy[2 * k + 1], &self.lazy[k]);
            self.seg[k] = self.reflect(k);
            self.lazy[k] = self.e.unit();
        }
    }
    #[inline]
    fn thrust(&mut self, k: usize) {
        for i in (1..=self.height).rev() {
            self.propagate(k >> i);
        }
    }
    #[inline]
    fn reflect(&self, k: usize) -> M::T {
        if self.lazy[k] != self.e.unit() {
            (self.f)(&self.seg[k], &self.lazy[k])
        } else {
            self.seg[k].clone()
        }
    }
    #[inline]
    fn recalc(&mut self, mut k: usize) {
        k /= 2;
        while k > 0 {
            self.seg[k] = self
                .m
                .operate(&self.reflect(2 * k), &self.reflect(2 * k + 1));
            k /= 2;
        }
    }
    pub fn update(&mut self, l: usize, r: usize, x: E::T) {
        debug_assert!(l < self.n);
        debug_assert!(r <= self.n);
        let mut a = l + self.n;
        let mut b = r + self.n;
        self.thrust(a);
        self.thrust(b - 1);
        while a < b {
            if a & 1 != 0 {
                self.lazy[a] = self.e.operate(&self.lazy[a], &x);
                a += 1;
            }
            if b & 1 != 0 {
                b -= 1;
                self.lazy[b] = self.e.operate(&self.lazy[b], &x);
            }
            a /= 2;
            b /= 2;
        }
        self.recalc(l + self.n);
        self.recalc(r + self.n - 1);
    }
    pub fn fold(&mut self, l: usize, r: usize) -> M::T {
        debug_assert!(l < self.n);
        debug_assert!(r <= self.n);
        let mut l = l + self.n;
        let mut r = r + self.n;
        self.thrust(l);
        self.thrust(r - 1);
        let mut vl = self.m.unit();
        let mut vr = self.m.unit();
        while l < r {
            if l & 1 != 0 {
                vl = self.m.operate(&vl, &self.reflect(l));
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = self.m.operate(&self.reflect(r), &vr);
            }
            l /= 2;
            r /= 2;
        }
        self.m.operate(&vl, &vr)
    }
    pub fn set(&mut self, k: usize, x: M::T) {
        let k = k + self.n;
        self.thrust(k);
        self.seg[k] = x;
        self.lazy[k] = self.e.unit();
        self.recalc(k);
    }
    pub fn get(&mut self, k: usize) -> M::T {
        self.fold(k, k + 1)
    }
    pub fn fold_all(&mut self) -> M::T {
        self.fold(0, self.n)
    }
    pub fn as_slice(&self) -> &[M::T] {
        &self.seg[self.n..]
    }
    pub fn as_slice_lazy(&self) -> &[E::T] {
        &self.lazy[self.n..]
    }
}

#[test]
fn test_lazy_segment_tree() {
    use crate::algebra::{AdditiveOperation, CartesianOperation, LastOperation, MaxOperation};
    use crate::tools::Xorshift;
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
            let l = rand.rand(n as u64) as usize;
            let r = rand.rand((n - l + 1) as u64) as usize + l;
            let x = rand.rand(1_000_000_000) as usize;
            seg.update(l, r, x);
            for a in arr[l..r].iter_mut() {
                *a += x;
            }
        } else {
            // Range Sum Query
            let l = rand.rand(n as u64) as usize;
            let r = rand.rand((n - l + 1) as u64) as usize + l;
            let mut res = 0;
            for a in arr[l..r].iter() {
                res += a;
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
            let r = rand.rand((n - l + 1) as u64) as usize + l;
            let x = rand.rand(1_000_000_000) as usize;
            seg.update(l, r, Some(x));
            for a in arr[l..r].iter_mut() {
                *a = x;
            }
        } else {
            // Range Max Query
            let l = rand.rand(n as u64) as usize;
            let r = rand.rand((n - l + 1) as u64) as usize + l;
            let mut res = 0;
            for a in arr[l..r].iter() {
                res = std::cmp::max(res, *a);
            }
            assert_eq!(seg.fold(l, r), res);
        }
    }
}
