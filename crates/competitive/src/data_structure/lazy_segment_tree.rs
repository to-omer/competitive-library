use super::{LazyMapMonoid, RangeBoundsExt};
use std::{
    fmt::{self, Debug, Formatter},
    mem::replace,
    ops::RangeBounds,
};

pub struct LazySegmentTree<M>
where
    M: LazyMapMonoid,
{
    n: usize,
    seg: Vec<(M::Agg, M::Act)>,
}

impl<M> Clone for LazySegmentTree<M>
where
    M: LazyMapMonoid,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            seg: self.seg.clone(),
        }
    }
}

impl<M> Debug for LazySegmentTree<M>
where
    M: LazyMapMonoid,
    M::Agg: Debug,
    M::Act: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazySegmentTree")
            .field("n", &self.n)
            .field("seg", &self.seg)
            .finish()
    }
}

impl<M> LazySegmentTree<M>
where
    M: LazyMapMonoid,
{
    pub fn new(n: usize) -> Self {
        let seg = vec![(M::agg_unit(), M::act_unit()); 2 * n];
        Self { n, seg }
    }
    pub fn from_vec(v: Vec<M::Agg>) -> Self {
        let n = v.len();
        let mut seg = vec![(M::agg_unit(), M::act_unit()); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[i + n].0 = x;
        }
        for i in (1..n).rev() {
            seg[i].0 = M::agg_operate(&seg[2 * i].0, &seg[2 * i + 1].0);
        }
        Self { n, seg }
    }
    pub fn from_keys(keys: impl ExactSizeIterator<Item = M::Key>) -> Self {
        let n = keys.len();
        let mut seg = vec![(M::agg_unit(), M::act_unit()); 2 * n];
        for (i, key) in keys.enumerate() {
            seg[i + n].0 = M::single_agg(&key);
        }
        for i in (1..n).rev() {
            seg[i].0 = M::agg_operate(&seg[2 * i].0, &seg[2 * i + 1].0);
        }
        Self { n, seg }
    }
    #[inline]
    fn update_at(&mut self, k: usize, x: &M::Act) {
        let nx = M::act_agg(&self.seg[k].0, x);
        if k < self.n {
            self.seg[k].1 = M::act_operate(&self.seg[k].1, x);
        }
        if let Some(nx) = nx {
            self.seg[k].0 = nx;
        } else if k < self.n {
            self.propagate_at(k);
            self.recalc_at(k);
        } else {
            panic!("act failed on leaf");
        }
    }
    #[inline]
    fn recalc_at(&mut self, k: usize) {
        self.seg[k].0 = M::agg_operate(&self.seg[2 * k].0, &self.seg[2 * k + 1].0);
    }
    #[inline]
    fn propagate_at(&mut self, k: usize) {
        debug_assert!(k < self.n);
        let x = replace(&mut self.seg[k].1, M::act_unit());
        self.update_at(2 * k, &x);
        self.update_at(2 * k + 1, &x);
    }
    #[inline]
    fn propagate(&mut self, k: usize, right: bool, nofilt: bool) {
        let right = right as usize;
        for i in (1..(k + 1 - right).next_power_of_two().trailing_zeros()).rev() {
            if nofilt || (k >> i) << i != k {
                self.propagate_at((k - right) >> i);
            }
        }
    }
    #[inline]
    fn recalc(&mut self, k: usize, right: bool, nofilt: bool) {
        let right = right as usize;
        for i in 1..(k + 1 - right).next_power_of_two().trailing_zeros() {
            if nofilt || (k >> i) << i != k {
                self.recalc_at((k - right) >> i);
            }
        }
    }
    pub fn update<R>(&mut self, range: R, x: M::Act)
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut a = range.start + self.n;
        let mut b = range.end + self.n;
        self.propagate(a, false, false);
        self.propagate(b, true, false);
        while a < b {
            if a & 1 != 0 {
                self.update_at(a, &x);
                a += 1;
            }
            if b & 1 != 0 {
                b -= 1;
                self.update_at(b, &x);
            }
            a /= 2;
            b /= 2;
        }
        self.recalc(range.start + self.n, false, false);
        self.recalc(range.end + self.n, true, false);
    }
    pub fn fold<R>(&mut self, range: R) -> M::Agg
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        self.propagate(l, false, true);
        self.propagate(r, true, true);
        let mut vl = M::agg_unit();
        let mut vr = M::agg_unit();
        while l < r {
            if l & 1 != 0 {
                vl = M::agg_operate(&vl, &self.seg[l].0);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = M::agg_operate(&self.seg[r].0, &vr);
            }
            l /= 2;
            r /= 2;
        }
        M::agg_operate(&vl, &vr)
    }
    pub fn set(&mut self, k: usize, x: M::Agg) {
        let k = k + self.n;
        self.propagate(k, false, true);
        self.seg[k] = (x, M::act_unit());
        self.recalc(k, false, true);
    }
    pub fn get(&mut self, k: usize) -> M::Agg {
        self.fold(k..k + 1)
    }
    pub fn fold_all(&mut self) -> M::Agg {
        self.fold(0..self.n)
    }
    fn bisect_perfect<P>(&mut self, mut pos: usize, mut acc: M::Agg, p: P) -> (usize, M::Agg)
    where
        P: Fn(&M::Agg) -> bool,
    {
        while pos < self.n {
            self.propagate_at(pos);
            pos <<= 1;
            let nacc = M::agg_operate(&acc, &self.seg[pos].0);
            if !p(&nacc) {
                acc = nacc;
                pos += 1;
            }
        }
        (pos - self.n, acc)
    }
    fn rbisect_perfect<P>(&mut self, mut pos: usize, mut acc: M::Agg, p: P) -> (usize, M::Agg)
    where
        P: Fn(&M::Agg) -> bool,
    {
        while pos < self.n {
            self.propagate_at(pos);
            pos = pos * 2 + 1;
            let nacc = M::agg_operate(&self.seg[pos].0, &acc);
            if !p(&nacc) {
                acc = nacc;
                pos -= 1;
            }
        }
        (pos - self.n, acc)
    }
    /// Returns the first index that satisfies a accumlative predicate.
    pub fn position_acc<R, P>(&mut self, range: R, p: P) -> Option<usize>
    where
        R: RangeBounds<usize>,
        P: Fn(&M::Agg) -> bool,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut l = range.start + self.n;
        let r = range.end + self.n;
        self.propagate(l, false, true);
        self.propagate(r, true, true);
        let mut k = 0usize;
        let mut acc = M::agg_unit();
        while l < r >> k {
            if l & 1 != 0 {
                let nacc = M::agg_operate(&acc, &self.seg[l].0);
                if p(&nacc) {
                    return Some(self.bisect_perfect(l, acc, p).0);
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
                let nacc = M::agg_operate(&acc, &self.seg[r - 1].0);
                if p(&nacc) {
                    return Some(self.bisect_perfect(r - 1, acc, p).0);
                }
                acc = nacc;
            }
        }
        None
    }
    /// Returns the last index that satisfies a accumlative predicate.
    pub fn rposition_acc<R, P>(&mut self, range: R, p: P) -> Option<usize>
    where
        R: RangeBounds<usize>,
        P: Fn(&M::Agg) -> bool,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        self.propagate(l, false, true);
        self.propagate(r, true, true);
        let mut c = 0usize;
        let mut k = 0usize;
        let mut acc = M::agg_unit();
        while l >> k < r {
            c <<= 1;
            if l & (1 << k) != 0 {
                l += 1 << k;
                c += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                let nacc = M::agg_operate(&self.seg[r].0, &acc);
                if p(&nacc) {
                    return Some(self.rbisect_perfect(r, acc, p).0);
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
                let nacc = M::agg_operate(&self.seg[l].0, &acc);
                if p(&nacc) {
                    return Some(self.rbisect_perfect(l, acc, p).0);
                }
                acc = nacc;
            }
            c >>= 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{RangeMaxRangeUpdate, RangeSumRangeAdd},
        rand,
        tools::{NotEmptySegment, Xorshift},
    };

    const N: usize = 1_000;
    const Q: usize = 20_000;
    const A: i64 = 1_000_000_000;

    #[test]
    fn test_lazy_segment_tree() {
        let mut rng = Xorshift::default();
        // Range Sum Query & Range Add Query
        rand!(rng, mut arr: [-A..A; N]);
        let mut seg =
            LazySegmentTree::<RangeSumRangeAdd<_>>::from_vec(arr.iter().map(|&a| (a, 1)).collect());
        for _ in 0..Q {
            rand!(rng, (l, r): NotEmptySegment(N));
            if rng.rand(2) == 0 {
                // Range Add Query
                rand!(rng, x: -A..A);
                seg.update(l..r, x);
                for a in arr[l..r].iter_mut() {
                    *a += x;
                }
            } else {
                // Range Sum Query
                let res = arr[l..r].iter().sum();
                assert_eq!(seg.fold(l..r).0, res);
            }
        }

        // Range Max Query & Range Update Query & Binary Search Query
        rand!(rng, mut arr: [-A..A; N]);
        let mut seg = LazySegmentTree::<RangeMaxRangeUpdate<_>>::from_vec(arr.clone());
        for _ in 0..Q {
            rand!(rng, ty: 0..4, (l, r): NotEmptySegment(N));
            match ty {
                0 => {
                    // Range Update Query
                    rand!(rng, x: -A..A);
                    seg.update(l..r, Some(x));
                    arr[l..r].iter_mut().for_each(|a| *a = x);
                }
                1 => {
                    // Range Max Query
                    let res = arr[l..r].iter().max().cloned().unwrap_or_default();
                    assert_eq!(seg.fold(l..r), res);
                }
                2 => {
                    // Binary Search Query
                    rand!(rng, x: -A..A);
                    assert_eq!(
                        seg.position_acc(l..r, |&d| d >= x),
                        arr[l..r]
                            .iter()
                            .scan(i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| i + l),
                    );
                }
                _ => {
                    // Binary Search Query
                    rand!(rng, x: -A..A);
                    assert_eq!(
                        seg.rposition_acc(l..r, |&d| d >= x),
                        arr[l..r]
                            .iter()
                            .rev()
                            .scan(i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| r - i - 1),
                    );
                }
            }
        }
    }
}
