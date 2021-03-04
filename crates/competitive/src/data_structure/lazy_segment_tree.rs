use crate::algebra::Monoid;

#[codesnip::entry("LazySegmentTree", include("algebra"))]
/// M: folding Monoid
/// E: lazy Monoid
/// F: lazy evaluating
#[derive(Clone, Debug)]
pub struct LazySegmentTree<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> {
    n: usize,
    height: u32,
    seg: Vec<M::T>,
    lazy: Vec<E::T>,
    m: M,
    e: E,
    f: F,
}
#[codesnip::entry("LazySegmentTree")]
impl<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> LazySegmentTree<M, E, F> {
    pub fn new(n: usize, m: M, e: E, f: F) -> Self {
        let n = n.next_power_of_two();
        let height = n.trailing_zeros();
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
        let n = v.len().next_power_of_two();
        let height = n.trailing_zeros();
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

#[codesnip::entry("LazySegmentTreeMap", include("algebra"))]
#[derive(Clone, Debug)]
pub struct LazySegmentTreeMap<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> {
    n: usize,
    height: u32,
    seg: std::collections::HashMap<usize, (M::T, E::T)>,
    m: M,
    e: E,
    f: F,
}
#[codesnip::entry("LazySegmentTreeMap")]
impl<M: Monoid, E: Monoid, F: Fn(&M::T, &E::T) -> M::T> LazySegmentTreeMap<M, E, F> {
    pub fn new(n: usize, m: M, e: E, f: F) -> Self {
        Self {
            n: n.next_power_of_two(),
            height: n.next_power_of_two().trailing_zeros(),
            seg: Default::default(),
            m,
            e,
            f,
        }
    }
    #[inline]
    fn propagate(&mut self, k: usize) {
        debug_assert!(k < self.n);
        let x = self
            .seg
            .get(&k)
            .map(|t| t.1.clone())
            .unwrap_or_else(|| self.e.unit());
        if x != self.e.unit() {
            let tl = self
                .seg
                .entry(2 * k)
                .or_insert((self.m.unit(), self.e.unit()));
            tl.1 = self.e.operate(&tl.1, &x);
            let tr = self
                .seg
                .entry(2 * k + 1)
                .or_insert((self.m.unit(), self.e.unit()));
            tr.1 = self.e.operate(&tr.1, &x);
            *self.seg.entry(k).or_insert((self.m.unit(), self.e.unit())) =
                (self.reflect(k), self.e.unit());
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
        let u = (self.m.unit(), self.e.unit());
        let t = self.seg.get(&k).unwrap_or(&u);
        if t.1 != self.e.unit() {
            (self.f)(&t.0, &t.1)
        } else {
            t.0.clone()
        }
    }
    #[inline]
    fn recalc(&mut self, mut k: usize) {
        k /= 2;
        while k > 0 {
            self.seg
                .entry(k)
                .or_insert((self.m.unit(), self.e.unit()))
                .0 = self
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
                let t = self.seg.entry(a).or_insert((self.m.unit(), self.e.unit()));
                t.1 = self.e.operate(&t.1, &x);
                a += 1;
            }
            if b & 1 != 0 {
                b -= 1;
                let t = self.seg.entry(b).or_insert((self.m.unit(), self.e.unit()));
                t.1 = self.e.operate(&t.1, &x);
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
        *self.seg.entry(k).or_insert((self.m.unit(), self.e.unit())) = (x, self.e.unit());
        self.recalc(k);
    }
    pub fn get(&mut self, k: usize) -> M::T {
        self.fold(k, k + 1)
    }
    pub fn fold_all(&mut self) -> M::T {
        self.fold(0, self.n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, CartesianOperation, LastOperation, MaxOperation},
        rand,
        tools::{NotEmptySegment, Xorshift},
    };

    const N: usize = 1_024;
    const Q: usize = 20_000;
    const A: i64 = 1_000_000_000;

    #[test]
    fn test_lazy_segment_tree() {
        let mut rng = Xorshift::time();
        // Range Sum Query & Range Add Query
        rand!(rng, mut arr: [-A..A; N]);
        let mut seg = LazySegmentTree::from_vec(
            arr.iter().map(|&a| (a, 1)).collect(),
            CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
            AdditiveOperation::new(),
            |x, &y| (x.0 + y * x.1, x.1),
        );
        for _ in 0..Q {
            if rng.rand(2) == 0 {
                // Range Add Query
                rand!(rng, (l,r): (NotEmptySegment(N)), x: (-A..A));
                seg.update(l, r, x);
                for a in arr[l..r].iter_mut() {
                    *a += x;
                }
            } else {
                // Range Sum Query
                rand!(rng, (l, r): (NotEmptySegment(N)));
                let res = arr[l..r].iter().sum();
                assert_eq!(seg.fold(l, r).0, res);
            }
        }

        // Range Max Query & Range Update Query
        rand!(rng, mut arr: [-A..A; N]);
        let mut seg = LazySegmentTree::from_vec(
            arr.clone(),
            MaxOperation::new(),
            LastOperation::new(),
            |&x, y| y.unwrap_or(x),
        );
        for _ in 0..Q {
            if rng.rand(2) == 0 {
                // Range Update Query
                rand!(rng, (l,r): (NotEmptySegment(N)), x: (-A..A));
                seg.update(l, r, Some(x));
                for a in arr[l..r].iter_mut() {
                    *a = x;
                }
            } else {
                // Range Max Query
                rand!(rng, (l, r): (NotEmptySegment(N)));
                let res = arr[l..r].iter().max().cloned().unwrap_or_default();
                assert_eq!(seg.fold(l, r), res);
            }
        }
    }

    #[test]
    fn test_lazy_segment_tree_map() {
        let mut rng = Xorshift::time();
        // Range Sum Query & Range Add Query
        let mut arr = vec![0i64; N];
        let mut seg = LazySegmentTreeMap::new(
            N,
            CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
            AdditiveOperation::new(),
            |x, &y| (x.0 + y * x.1, x.1),
        );
        for i in 0..N {
            seg.set(i, (0i64, 1i64));
        }
        for _ in 0..Q {
            if rng.rand(2) == 0 {
                // Range Add Query
                rand!(rng, (l,r): (NotEmptySegment(N)), x: (-A..A));
                seg.update(l, r, x);
                for a in arr[l..r].iter_mut() {
                    *a += x;
                }
            } else {
                // Range Sum Query
                rand!(rng, (l, r): (NotEmptySegment(N)));
                let res = arr[l..r].iter().sum();
                assert_eq!(seg.fold(l, r).0, res);
            }
        }

        // Range Max Query & Range Update Query
        let mut arr = vec![std::i64::MIN; N];
        let mut seg =
            LazySegmentTreeMap::new(N, MaxOperation::new(), LastOperation::new(), |&x, y| {
                y.unwrap_or(x)
            });
        for _ in 0..Q {
            if rng.rand(2) == 0 {
                // Range Update Query
                rand!(rng, (l,r): (NotEmptySegment(N)), x: (-A..A));
                seg.update(l, r, Some(x));
                for a in arr[l..r].iter_mut() {
                    *a = x;
                }
            } else {
                // Range Max Query
                rand!(rng, (l, r): (NotEmptySegment(N)));
                let res = arr[l..r].iter().max().cloned().unwrap_or_default();
                assert_eq!(seg.fold(l, r), res);
            }
        }
    }
}
