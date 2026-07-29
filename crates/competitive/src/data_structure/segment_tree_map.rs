use super::{AbelianMonoid, FibHashMap, Monoid, RangeBoundsExt};
use std::{
    fmt::{self, Debug, Formatter},
    ops::RangeBounds,
};

pub struct SegmentTreeMap<M>
where
    M: Monoid,
{
    n: usize,
    seg: FibHashMap<usize, M::T>,
    u: M::T,
}

impl<M> Clone for SegmentTreeMap<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            seg: self.seg.clone(),
            u: self.u.clone(),
        }
    }
}

impl<M> Debug for SegmentTreeMap<M>
where
    M: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentTreeMap")
            .field("n", &self.n)
            .field("seg", &self.seg)
            .field("u", &self.u)
            .finish()
    }
}

impl<M> SegmentTreeMap<M>
where
    M: Monoid,
{
    pub fn new(n: usize) -> Self {
        let u = M::unit();
        Self {
            n,
            seg: Default::default(),
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
        *self.seg.entry(k).or_insert(M::unit()) = x;
        k /= 2;
        while k > 0 {
            *self.seg.entry(k).or_insert(M::unit()) =
                M::operate(self.get_ref(2 * k), self.get_ref(2 * k + 1));
            k /= 2;
        }
    }
    pub fn update(&mut self, k: usize, x: M::T) {
        debug_assert!(k < self.n);
        let mut k = k + self.n;
        let t = self.seg.entry(k).or_insert(M::unit());
        *t = M::operate(t, &x);
        k /= 2;
        while k > 0 {
            *self.seg.entry(k).or_insert(M::unit()) =
                M::operate(self.get_ref(2 * k), self.get_ref(2 * k + 1));
            k /= 2;
        }
    }
    pub fn get(&self, k: usize) -> M::T {
        debug_assert!(k < self.n);
        self.seg.get(&(k + self.n)).cloned().unwrap_or_else(M::unit)
    }
    pub fn fold<R>(&self, range: R) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range();
        debug_assert!(range.end <= self.n);
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        let mut vl = M::unit();
        let mut vr = M::unit();
        while l < r {
            if l & 1 != 0 {
                vl = M::operate(&vl, self.get_ref(l));
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = M::operate(self.get_ref(r), &vr);
            }
            l /= 2;
            r /= 2;
        }
        M::operate(&vl, &vr)
    }
    fn partition_point_perfect<P>(
        &self,
        mut pos: usize,
        mut acc: M::T,
        mut pred: P,
    ) -> (usize, M::T)
    where
        P: FnMut(&M::T) -> bool,
    {
        while pos < self.n {
            pos <<= 1;
            let nacc = M::operate(&acc, self.get_ref(pos));
            if pred(&nacc) {
                acc = nacc;
                pos += 1;
            }
        }
        (pos - self.n, acc)
    }
    fn rpartition_point_perfect<P>(
        &self,
        mut pos: usize,
        mut acc: M::T,
        mut pred: P,
    ) -> (usize, M::T)
    where
        P: FnMut(&M::T) -> bool,
    {
        while pos < self.n {
            pos = pos * 2 + 1;
            let nacc = M::operate(self.get_ref(pos), &acc);
            if pred(&nacc) {
                acc = nacc;
                pos -= 1;
            }
        }
        (pos - self.n, acc)
    }
    pub fn partition_point_acc<P>(&self, left: usize, mut pred: P) -> usize
    where
        P: FnMut(&M::T) -> bool,
    {
        let mut l = left + self.n;
        let r = 2 * self.n;
        let mut k = 0usize;
        let mut acc = M::unit();
        while l < r >> k {
            if l & 1 != 0 {
                let nacc = M::operate(&acc, self.get_ref(l));
                if !pred(&nacc) {
                    return self.partition_point_perfect(l, acc, pred).0;
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
                let nacc = M::operate(&acc, self.get_ref(r - 1));
                if !pred(&nacc) {
                    return self.partition_point_perfect(r - 1, acc, pred).0;
                }
                acc = nacc;
            }
        }
        self.n
    }
    pub fn rpartition_point_acc<P>(&self, right: usize, mut pred: P) -> usize
    where
        P: FnMut(&M::T) -> bool,
    {
        let mut l = self.n;
        let mut r = right + self.n;
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
                let nacc = M::operate(self.get_ref(r), &acc);
                if !pred(&nacc) {
                    return self.rpartition_point_perfect(r, acc, pred).0 + 1;
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
                let nacc = M::operate(self.get_ref(l), &acc);
                if !pred(&nacc) {
                    return self.rpartition_point_perfect(l, acc, pred).0 + 1;
                }
                acc = nacc;
            }
            c >>= 1;
        }
        0
    }
}

impl<M> SegmentTreeMap<M>
where
    M: AbelianMonoid,
{
    pub fn fold_all(&self) -> M::T {
        self.seg.get(&1).cloned().unwrap_or_else(M::unit)
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
    fn test_segment_tree_map() {
        let mut rng = Xorshift::default();
        let mut arr = vec![0; N + 1];
        let mut seg = SegmentTreeMap::<AdditiveOperation<_>>::new(N);
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
        for (left, v) in rng.random_iter((..=N, 1..=A * N as i64)).take(Q) {
            assert_eq!(
                seg.partition_point_acc(left, |&x| x < v),
                arr[left + 1..].position_bisect(|&x| x - arr[left] >= v) + left
            );
        }
        for (right, v) in rng.random_iter((..=N, 1..=A)).take(Q) {
            assert_eq!(
                seg.rpartition_point_acc(right, |&x| x < v),
                arr[..right].rposition_bisect(|&x| arr[right] - x >= v)
            );
        }

        rand!(rng, mut arr: [-A..=A; N]);
        let mut seg = SegmentTreeMap::<MaxOperation<_>>::new(N);
        for (i, a) in arr.iter().cloned().enumerate() {
            seg.set(i, a);
        }
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
