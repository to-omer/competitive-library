#![allow(clippy::or_fun_call)]

use crate::algebra::Monoid;

#[codesnip::entry("SegmentTree", include("algebra"))]
#[derive(Clone, Debug)]
pub struct SegmentTree<M: Monoid> {
    n: usize,
    seg: Vec<M::T>,
    m: M,
}
#[codesnip::entry("SegmentTree")]
impl<M: Monoid> SegmentTree<M> {
    pub fn new(n: usize, m: M) -> Self {
        let n = n.next_power_of_two();
        let seg = vec![m.unit(); 2 * n];
        Self { n, seg, m }
    }
    pub fn from_vec(v: Vec<M::T>, m: M) -> Self {
        let n = v.len().next_power_of_two();
        let mut seg = vec![m.unit(); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[n + i] = x;
        }
        for i in (1..n).rev() {
            seg[i] = m.operate(&seg[2 * i], &seg[2 * i + 1]);
        }
        Self { n, seg, m }
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
    pub fn as_slice(&self) -> &[M::T] {
        &self.seg[self.n..]
    }
}

#[test]
fn test_segment_tree() {
    use crate::algebra::{AdditiveOperation, MaxOperation};
    use crate::algorithm::SliceBisectExt;
    use crate::tools::Xorshift;
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
            arr[1..].position_bisect(|&x| x >= v)
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
            arr[l + 1..r + 1].position_bisect(|&x| x >= v + arr[l]) + l
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
        let r = rand.rand((n - l + 1) as u64) as usize + l;
        let mut res = 0;
        for a in arr[l..r].iter().cloned() {
            res = std::cmp::max(res, a);
        }
        assert_eq!(seg.fold(l, r), res);
    }
}

#[codesnip::entry("SegmentTreeMap", include("algebra"))]
#[derive(Clone, Debug)]
pub struct SegmentTreeMap<M: Monoid> {
    n: usize,
    seg: std::collections::HashMap<usize, M::T>,
    m: M,
    u: M::T,
}
#[codesnip::entry("SegmentTreeMap")]
impl<M: Monoid> SegmentTreeMap<M> {
    pub fn new(n: usize, m: M) -> Self {
        let n = n.next_power_of_two();
        let u = m.unit();
        Self {
            n,
            seg: Default::default(),
            m,
            u,
        }
    }
    #[inline]
    fn get_ref(&self, k: usize) -> &M::T {
        self.seg.get(&k).unwrap_or(&self.u)
    }
    pub fn set(&mut self, k: usize, x: M::T) {
        debug_assert!(k < self.n);
        let mut k = k + self.n;
        *self.seg.entry(k).or_insert(self.m.unit()) = x;
        k /= 2;
        while k > 0 {
            *self.seg.entry(k).or_insert(self.m.unit()) =
                self.m.operate(self.get_ref(2 * k), self.get_ref(2 * k + 1));
            k /= 2;
        }
    }
    pub fn update(&mut self, k: usize, x: M::T) {
        debug_assert!(k < self.n);
        let mut k = k + self.n;
        let t = self.seg.entry(k).or_insert(self.m.unit());
        *t = self.m.operate(&t, &x);
        k /= 2;
        while k > 0 {
            *self.seg.entry(k).or_insert(self.m.unit()) =
                self.m.operate(self.get_ref(2 * k), self.get_ref(2 * k + 1));
            k /= 2;
        }
    }
    pub fn get(&self, k: usize) -> M::T {
        debug_assert!(k < self.n);
        self.seg
            .get(&(k + self.n))
            .cloned()
            .unwrap_or_else(|| self.m.unit())
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
                vl = self.m.operate(&vl, self.get_ref(l));
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = self.m.operate(self.get_ref(r), &vr);
            }
            l /= 2;
            r /= 2;
        }
        self.m.operate(&vl, &vr)
    }
    pub fn fold_all(&self) -> M::T {
        self.seg.get(&1).cloned().unwrap_or_else(|| self.m.unit())
    }
    /// left most index [0, r) that satisfies monotonic condition
    pub fn lower_bound_all<F: Fn(&M::T) -> bool>(&self, f: F, r: usize) -> usize {
        if !f(&self.get_ref(1)) {
            return r;
        }
        let mut acc = self.m.unit();
        let mut pos = 1;
        while pos < self.n {
            pos *= 2;
            let y = self.m.operate(&acc, self.get_ref(pos));
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
            let y = self.m.operate(&acc, self.get_ref(pos));
            if f(&y) {
                while pos < self.n {
                    pos *= 2;
                    let y = self.m.operate(&acc, self.get_ref(pos));
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
fn test_segment_tree_map() {
    use crate::algebra::{AdditiveOperation, MaxOperation};
    use crate::algorithm::SliceBisectExt;
    use crate::tools::Xorshift;
    let mut rand = Xorshift::time();
    let n = 1_024;
    let q = 10_000;
    let mut seg = SegmentTreeMap::new(n, AdditiveOperation::new());
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
            arr[1..].position_bisect(|&x| x >= v)
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
            arr[l + 1..r + 1].position_bisect(|&x| x >= v + arr[l]) + l
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
        let r = rand.rand((n - l + 1) as u64) as usize + l;
        let mut res = 0;
        for a in arr[l..r].iter().cloned() {
            res = std::cmp::max(res, a);
        }
        assert_eq!(seg.fold(l, r), res);
    }
}
