use super::MonoidAction;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    mem::replace,
};

pub struct LazySegmentTreeMap<M>
where
    M: MonoidAction,
    M::Act: PartialEq,
{
    n: usize,
    seg: HashMap<usize, (M::Agg, M::Act)>,
}

impl<M> Clone for LazySegmentTreeMap<M>
where
    M: MonoidAction,
    M::Act: PartialEq,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            seg: self.seg.clone(),
        }
    }
}

impl<M> Debug for LazySegmentTreeMap<M>
where
    M: MonoidAction,
    M::Agg: Debug,
    M::Act: PartialEq + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazySegmentTreeMap")
            .field("n", &self.n)
            .field("seg", &self.seg)
            .finish()
    }
}

impl<M> LazySegmentTreeMap<M>
where
    M: MonoidAction,
    M::Act: PartialEq,
{
    pub fn new(n: usize) -> Self {
        Self {
            n,
            seg: Default::default(),
        }
    }
    #[inline]
    fn get_mut(&mut self, k: usize) -> &mut (M::Agg, M::Act) {
        self.seg.entry(k).or_insert((M::agg_unit(), M::act_unit()))
    }
    #[inline]
    fn update_at(&mut self, k: usize, x: &M::Act) {
        let n = self.n;
        let a = self.get_mut(k);
        let nx = M::act_agg(&a.0, x);
        if k < n {
            a.1 = M::act_operate(&a.1, x);
        }
        if let Some(nx) = nx {
            a.0 = nx;
        } else if k < n {
            self.propagate_at(k);
            self.recalc_at(k);
        } else {
            panic!("act failed on leaf");
        }
    }
    #[inline]
    fn recalc_at(&mut self, k: usize) {
        let x = match (self.seg.get(&(2 * k)), self.seg.get(&(2 * k + 1))) {
            (None, None) => M::agg_unit(),
            (None, Some((y, _))) => y.clone(),
            (Some((x, _)), None) => x.clone(),
            (Some((x, _)), Some((y, _))) => M::agg_operate(x, y),
        };
        self.get_mut(k).0 = x;
    }
    #[inline]
    fn propagate_at(&mut self, k: usize) {
        debug_assert!(k < self.n);
        let x = match self.seg.get_mut(&k) {
            Some((_, x)) => replace(x, M::act_unit()),
            None => M::act_unit(),
        };
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
    pub fn update(&mut self, l: usize, r: usize, x: M::Act) {
        debug_assert!(l <= r);
        debug_assert!(r <= self.n);
        let mut a = l + self.n;
        let mut b = r + self.n;
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
        self.recalc(l + self.n, false, false);
        self.recalc(r + self.n, true, false);
    }
    pub fn fold(&mut self, l: usize, r: usize) -> M::Agg {
        debug_assert!(l <= r);
        debug_assert!(r <= self.n);
        let mut l = l + self.n;
        let mut r = r + self.n;
        self.propagate(l, false, true);
        self.propagate(r, true, true);
        let mut vl = M::agg_unit();
        let mut vr = M::agg_unit();
        while l < r {
            if l & 1 != 0 {
                if let Some((x, _)) = self.seg.get(&l) {
                    vl = M::agg_operate(&vl, x);
                }
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                if let Some((x, _)) = self.seg.get(&r) {
                    vr = M::agg_operate(x, &vr);
                }
            }
            l /= 2;
            r /= 2;
        }
        M::agg_operate(&vl, &vr)
    }
    pub fn set(&mut self, k: usize, x: M::Agg) {
        let k = k + self.n;
        self.propagate(k, false, true);
        *self.get_mut(k) = (x, M::act_unit());
        self.recalc(k, false, true);
    }
    pub fn get(&mut self, k: usize) -> M::Agg {
        self.fold(k, k + 1)
    }
    pub fn fold_all(&mut self) -> M::Agg {
        self.fold(0, self.n)
    }
    fn bisect_perfect<P>(&mut self, mut pos: usize, mut acc: M::Agg, p: P) -> (usize, M::Agg)
    where
        P: Fn(&M::Agg) -> bool,
    {
        while pos < self.n {
            self.propagate_at(pos);
            pos <<= 1;
            let nacc = match self.seg.get(&pos) {
                Some((x, _)) => M::agg_operate(&acc, x),
                None => acc.clone(),
            };
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
            let nacc = match self.seg.get(&pos) {
                Some((x, _)) => M::agg_operate(x, &acc),
                None => acc.clone(),
            };
            if !p(&nacc) {
                acc = nacc;
                pos -= 1;
            }
        }
        (pos - self.n, acc)
    }
    /// Returns the first index that satisfies a accumlative predicate.
    pub fn position_acc<P>(&mut self, l: usize, r: usize, p: P) -> Option<usize>
    where
        P: Fn(&M::Agg) -> bool,
    {
        let mut l = l + self.n;
        let r = r + self.n;
        self.propagate(l, false, true);
        self.propagate(r, true, true);
        let mut k = 0usize;
        let mut acc = M::agg_unit();
        while l < r >> k {
            if l & 1 != 0 {
                let nacc = match self.seg.get(&l) {
                    Some((x, _)) => M::agg_operate(&acc, x),
                    None => acc.clone(),
                };
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
                let nacc = match self.seg.get(&(r - 1)) {
                    Some((x, _)) => M::agg_operate(&acc, x),
                    None => acc.clone(),
                };
                if p(&nacc) {
                    return Some(self.bisect_perfect(r - 1, acc, p).0);
                }
                acc = nacc;
            }
        }
        None
    }
    /// Returns the last index that satisfies a accumlative predicate.
    pub fn rposition_acc<P>(&mut self, l: usize, r: usize, p: P) -> Option<usize>
    where
        P: Fn(&M::Agg) -> bool,
    {
        let mut l = l + self.n;
        let mut r = r + self.n;
        self.propagate(l, false, true);
        self.propagate(r, true, true);
        let mut c = 0usize;
        let mut k = 0usize;
        let mut acc = M::agg_unit();
        while l >> k < r {
            c <<= 1;
            if l & 1 << k != 0 {
                l += 1 << k;
                c += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                let nacc = match self.seg.get(&r) {
                    Some((x, _)) => M::agg_operate(x, &acc),
                    None => acc.clone(),
                };
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
                let nacc = match self.seg.get(&l) {
                    Some((x, _)) => M::agg_operate(x, &acc),
                    None => acc.clone(),
                };
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
    fn test_lazy_segment_tree_map() {
        let mut rng = Xorshift::new();
        // Range Sum Query & Range Add Query
        let mut arr = vec![0i64; N];
        let mut seg = LazySegmentTreeMap::<RangeSumRangeAdd<_>>::new(N);
        for i in 0..N {
            seg.set(i, (0i64, 1i64));
        }
        for _ in 0..Q {
            rand!(rng, (l, r): (NotEmptySegment(N)));
            if rng.rand(2) == 0 {
                // Range Add Query
                rand!(rng, x: (-A..A));
                seg.update(l, r, x);
                for a in arr[l..r].iter_mut() {
                    *a += x;
                }
            } else {
                // Range Sum Query
                let res = arr[l..r].iter().sum();
                assert_eq!(seg.fold(l, r).0, res);
            }
        }

        // Range Max Query & Range Update Query & Binary Search Query
        let mut arr = vec![std::i64::MIN; N];
        let mut seg = LazySegmentTreeMap::<RangeMaxRangeUpdate<_>>::new(N);
        for _ in 0..Q {
            rand!(rng, ty: (0..4), (l, r): (NotEmptySegment(N)));
            match ty {
                0 => {
                    // Range Update Query
                    rand!(rng, x: (-A..A));
                    seg.update(l, r, Some(x));
                    arr[l..r].iter_mut().for_each(|a| *a = x);
                }
                1 => {
                    // Range Max Query
                    let res = arr[l..r].iter().max().cloned().unwrap_or_default();
                    assert_eq!(seg.fold(l, r), res);
                }
                2 => {
                    // Binary Search Query
                    rand!(rng, x: (-A..A));
                    assert_eq!(
                        seg.position_acc(l, r, |&d| d >= x),
                        arr[l..r]
                            .iter()
                            .scan(std::i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map(|i| i + l),
                    );
                }
                _ => {
                    // Binary Search Query
                    rand!(rng, x: (-A..A));
                    assert_eq!(
                        seg.rposition_acc(l, r, |&d| d >= x),
                        arr[l..r]
                            .iter()
                            .rev()
                            .scan(std::i64::MIN, |acc, &a| {
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
