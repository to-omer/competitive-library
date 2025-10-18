use super::{AbelianMonoid, Monoid, RangeBoundsExt};
use std::{
    fmt::{self, Debug, Formatter},
    ops::RangeBounds,
};

pub struct SegmentTree<M>
where
    M: Monoid,
{
    n: usize,
    seg: Vec<M::T>,
}

impl<M> Clone for SegmentTree<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            seg: self.seg.clone(),
        }
    }
}

impl<M> Debug for SegmentTree<M>
where
    M: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentTree")
            .field("n", &self.n)
            .field("seg", &self.seg)
            .finish()
    }
}

impl<M> SegmentTree<M>
where
    M: Monoid,
{
    pub fn new(n: usize) -> Self {
        let seg = vec![M::unit(); 2 * n];
        Self { n, seg }
    }
    pub fn from_vec(v: Vec<M::T>) -> Self {
        let n = v.len();
        let mut seg = vec![M::unit(); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[n + i] = x;
        }
        for i in (1..n).rev() {
            seg[i] = M::operate(&seg[2 * i], &seg[2 * i + 1]);
        }
        Self { n, seg }
    }
    pub fn set(&mut self, k: usize, x: M::T) {
        assert!(k < self.n);
        let mut k = k + self.n;
        self.seg[k] = x;
        k /= 2;
        while k > 0 {
            self.seg[k] = M::operate(&self.seg[2 * k], &self.seg[2 * k + 1]);
            k /= 2;
        }
    }
    pub fn clear(&mut self, k: usize) {
        self.set(k, M::unit());
    }
    pub fn update(&mut self, k: usize, x: M::T) {
        assert!(k < self.n);
        let mut k = k + self.n;
        self.seg[k] = M::operate(&self.seg[k], &x);
        k /= 2;
        while k > 0 {
            self.seg[k] = M::operate(&self.seg[2 * k], &self.seg[2 * k + 1]);
            k /= 2;
        }
    }
    pub fn get(&self, k: usize) -> M::T {
        assert!(k < self.n);
        self.seg[k + self.n].clone()
    }
    pub fn fold<R>(&self, range: R) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        let mut vl = M::unit();
        let mut vr = M::unit();
        while l < r {
            if l & 1 != 0 {
                vl = M::operate(&vl, &self.seg[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = M::operate(&self.seg[r], &vr);
            }
            l /= 2;
            r /= 2;
        }
        M::operate(&vl, &vr)
    }
    fn bisect_perfect<F>(&self, mut pos: usize, mut acc: M::T, f: F) -> (usize, M::T)
    where
        F: Fn(&M::T) -> bool,
    {
        while pos < self.n {
            pos <<= 1;
            let nacc = M::operate(&acc, &self.seg[pos]);
            if !f(&nacc) {
                acc = nacc;
                pos += 1;
            }
        }
        (pos - self.n, acc)
    }
    fn rbisect_perfect<F>(&self, mut pos: usize, mut acc: M::T, f: F) -> (usize, M::T)
    where
        F: Fn(&M::T) -> bool,
    {
        while pos < self.n {
            pos = pos * 2 + 1;
            let nacc = M::operate(&self.seg[pos], &acc);
            if !f(&nacc) {
                acc = nacc;
                pos -= 1;
            }
        }
        (pos - self.n, acc)
    }
    /// Returns the first index that satisfies a accumlative predicate.
    pub fn position_acc<R, F>(&self, range: R, f: F) -> Option<usize>
    where
        R: RangeBounds<usize>,
        F: Fn(&M::T) -> bool,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut l = range.start + self.n;
        let r = range.end + self.n;
        let mut k = 0usize;
        let mut acc = M::unit();
        while l < r >> k {
            if l & 1 != 0 {
                let nacc = M::operate(&acc, &self.seg[l]);
                if f(&nacc) {
                    return Some(self.bisect_perfect(l, acc, f).0);
                }
                acc = nacc;
                l += 1;
            }
            l >>= 1;
            k += 1;
        }
        for k in (0..k).rev() {
            let r = r >> k;
            if r & 1 != 0 {
                let nacc = M::operate(&acc, &self.seg[r - 1]);
                if f(&nacc) {
                    return Some(self.bisect_perfect(r - 1, acc, f).0);
                }
                acc = nacc;
            }
        }
        None
    }
    /// Returns the last index that satisfies a accumlative predicate.
    pub fn rposition_acc<R, F>(&self, range: R, f: F) -> Option<usize>
    where
        R: RangeBounds<usize>,
        F: Fn(&M::T) -> bool,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        let mut c = 0usize;
        let mut k = 0usize;
        let mut acc = M::unit();
        while l >> k < r {
            c <<= 1;
            if l & (1 << k) != 0 {
                l += 1 << k;
                c += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                let nacc = M::operate(&self.seg[r], &acc);
                if f(&nacc) {
                    return Some(self.rbisect_perfect(r, acc, f).0);
                }
                acc = nacc;
            }
            r >>= 1;
            k += 1;
        }
        for k in (0..k).rev() {
            if c & 1 != 0 {
                l -= 1 << k;
                let l = l >> k;
                let nacc = M::operate(&self.seg[l], &acc);
                if f(&nacc) {
                    return Some(self.rbisect_perfect(l, acc, f).0);
                }
                acc = nacc;
            }
            c >>= 1;
        }
        None
    }
    pub fn as_slice(&self) -> &[M::T] {
        &self.seg[self.n..]
    }
}
impl<M> SegmentTree<M>
where
    M: AbelianMonoid,
{
    pub fn fold_all(&self) -> M::T {
        self.seg[1].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, MaxOperation},
        algorithm::SliceBisectExt as _,
        rand,
        tools::{NotEmptySegment as Nes, Xorshift},
    };

    const N: usize = 1_000;
    const Q: usize = 10_000;
    const A: i64 = 1_000_000_000;

    #[test]
    fn test_segment_tree() {
        let mut rng = Xorshift::default();
        let mut arr = vec![0; N + 1];
        let mut seg = SegmentTree::<AdditiveOperation<_>>::new(N);
        for (k, v) in rng.random_iter((..N, 1..=A)).take(Q) {
            seg.set(k, v);
            arr[k + 1] = v;
        }
        for i in 0..N {
            arr[i + 1] += arr[i];
        }
        for i in 0..N {
            for j in i + 1..N + 1 {
                assert_eq!(seg.fold(i..j), arr[j] - arr[i]);
            }
        }
        for v in rng.random_iter(1..=A * N as i64).take(Q) {
            assert_eq!(
                seg.position_acc(0..N, |&x| v <= x).unwrap_or(N),
                arr[1..].position_bisect(|&x| x >= v)
            );
        }
        for ((l, r), v) in rng.random_iter((Nes(N), 1..=A)).take(Q) {
            assert_eq!(
                seg.position_acc(l..r, |&x| v <= x).unwrap_or(r),
                arr[l + 1..r + 1].position_bisect(|&x| x - arr[l] >= v) + l
            );
            assert_eq!(
                seg.rposition_acc(l..r, |&x| v <= x).map_or(l, |i| i + 1),
                arr[l..r].rposition_bisect(|&x| arr[r] - x >= v) + l
            );
        }

        rand!(rng, mut arr: [-A..=A; N]);
        let mut seg = SegmentTree::<MaxOperation<_>>::from_vec(arr.clone());
        for (k, v) in rng.random_iter((..N, -A..=A)).take(Q) {
            seg.set(k, v);
            arr[k] = v;
        }
        for (l, r) in rng.random_iter(Nes(N)).take(Q) {
            let res = arr[l..r].iter().max().cloned().unwrap_or_default();
            assert_eq!(seg.fold(l..r), res);
        }
    }
}
