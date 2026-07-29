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
    len: usize,
    n: usize,
    seg: Vec<M::Agg>,
    lazy: Vec<M::Act>,
}

impl<M> Clone for LazySegmentTree<M>
where
    M: LazyMapMonoid,
{
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            n: self.n,
            seg: self.seg.clone(),
            lazy: self.lazy.clone(),
        }
    }
}

impl<M> Debug for LazySegmentTree<M>
where
    M: LazyMapMonoid<Agg: Debug, Act: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazySegmentTree")
            .field("len", &self.len)
            .field("n", &self.n)
            .field("seg", &self.seg)
            .field("lazy", &self.lazy)
            .finish()
    }
}

impl<M> LazySegmentTree<M>
where
    M: LazyMapMonoid,
{
    pub fn new(len: usize) -> Self {
        let n = len.next_power_of_two();
        let seg = vec![M::agg_unit(); 2 * n];
        let lazy = vec![M::act_unit(); n];
        Self { len, n, seg, lazy }
    }
    pub fn from_vec(v: Vec<M::Agg>) -> Self {
        let len = v.len();
        let n = len.next_power_of_two();
        let mut seg = vec![M::agg_unit(); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[i + n] = x;
        }
        for i in (1..n).rev() {
            seg[i] = M::agg_operate(&seg[2 * i], &seg[2 * i + 1]);
        }
        let lazy = vec![M::act_unit(); n];
        Self { len, n, seg, lazy }
    }
    pub fn from_keys(keys: impl ExactSizeIterator<Item = M::Key>) -> Self {
        let len = keys.len();
        let n = len.next_power_of_two();
        let mut seg = vec![M::agg_unit(); 2 * n];
        for (i, key) in keys.enumerate() {
            seg[i + n] = M::single_agg(&key);
        }
        for i in (1..n).rev() {
            seg[i] = M::agg_operate(&seg[2 * i], &seg[2 * i + 1]);
        }
        let lazy = vec![M::act_unit(); n];
        Self { len, n, seg, lazy }
    }
    #[inline]
    fn update_at(&mut self, k: usize, x: &M::Act) {
        if M::is_act_unit(x) {
            return;
        }
        let nx = M::act_agg(&self.seg[k], x);
        if k < self.n {
            self.lazy[k] = M::act_operate(&self.lazy[k], x);
        }
        if let Some(nx) = nx {
            self.seg[k] = nx;
        } else if k < self.n {
            self.propagate_at(k);
            self.recalc_at(k);
        } else {
            panic!("act failed on leaf");
        }
    }
    #[inline]
    fn recalc_at(&mut self, k: usize) {
        self.seg[k] = M::agg_operate(&self.seg[2 * k], &self.seg[2 * k + 1]);
    }
    #[inline]
    fn propagate_at(&mut self, k: usize) {
        debug_assert!(k < self.n);
        let x = replace(&mut self.lazy[k], M::act_unit());
        if M::is_act_unit(&x) {
            return;
        }
        self.update_at(2 * k, &x);
        self.update_at(2 * k + 1, &x);
    }
    #[inline]
    fn propagate(&mut self, k: usize) {
        for i in (1..=self.n.trailing_zeros()).rev() {
            self.propagate_at(k >> i);
        }
    }
    #[inline]
    fn recalc(&mut self, mut k: usize) {
        while k > 1 {
            k >>= 1;
            self.recalc_at(k);
        }
    }
    pub fn update<R>(&mut self, range: R, x: M::Act)
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.len).expect("invalid range");
        if range.is_empty() || M::is_act_unit(&x) {
            return;
        }
        let mut a = range.start + self.n;
        let mut b = range.end + self.n;
        for i in (1..=self.n.trailing_zeros()).rev() {
            if (a >> i) << i != a {
                self.propagate_at(a >> i);
            }
            if (b >> i) << i != b {
                self.propagate_at((b - 1) >> i);
            }
        }
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
        let a = range.start + self.n;
        let b = range.end + self.n;
        for i in 1..=self.n.trailing_zeros() {
            if (a >> i) << i != a {
                self.recalc_at(a >> i);
            }
            if (b >> i) << i != b {
                self.recalc_at((b - 1) >> i);
            }
        }
    }
    pub fn fold<R>(&mut self, range: R) -> M::Agg
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.len).expect("invalid range");
        if range.is_empty() {
            return M::agg_unit();
        }
        if let Some(result) = (|| {
            let mut left_index = range.start + self.n - 1;
            let mut right_index = range.end + self.n;
            let mut left = M::agg_unit();
            let mut right = M::agg_unit();
            let mut has_left = false;
            let mut has_right = false;
            for _ in 0..(left_index ^ right_index).ilog2() {
                if left_index & 1 == 0 {
                    left = M::agg_operate(&left, &self.seg[left_index ^ 1]);
                    has_left = true;
                }
                if right_index & 1 != 0 {
                    right = M::agg_operate(&self.seg[right_index ^ 1], &right);
                    has_right = true;
                }
                left_index >>= 1;
                right_index >>= 1;
                if has_left {
                    left = M::act_agg(&left, &self.lazy[left_index])?;
                }
                if has_right && right_index < self.n {
                    right = M::act_agg(&right, &self.lazy[right_index])?;
                }
            }
            let mut result = M::agg_operate(&left, &right);
            while left_index > 1 {
                left_index >>= 1;
                result = M::act_agg(&result, &self.lazy[left_index])?;
            }
            Some(result)
        })() {
            return result;
        }
        let mut l = range.start + self.n;
        let mut r = range.end + self.n;
        self.propagate(l);
        self.propagate(r - 1);
        let mut vl = M::agg_unit();
        let mut vr = M::agg_unit();
        while l < r {
            if l & 1 != 0 {
                vl = M::agg_operate(&vl, &self.seg[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = M::agg_operate(&self.seg[r], &vr);
            }
            l /= 2;
            r /= 2;
        }
        M::agg_operate(&vl, &vr)
    }
    pub fn set(&mut self, k: usize, x: M::Agg) {
        assert!(k < self.len);
        let k = k + self.n;
        self.propagate(k);
        self.seg[k] = x;
        self.recalc(k);
    }
    pub fn get(&mut self, k: usize) -> M::Agg {
        self.fold(k..k + 1)
    }
    pub fn fold_all(&mut self) -> M::Agg {
        self.fold(0..self.len)
    }
    pub fn partition_point_acc<P>(&mut self, left: usize, mut pred: P) -> usize
    where
        P: FnMut(&M::Agg) -> bool,
    {
        let mut acc = M::agg_unit();
        if left == self.len {
            return self.len;
        }
        let mut k = left + self.n;
        self.propagate(k);
        loop {
            while k & 1 == 0 {
                k >>= 1;
            }
            let nacc = M::agg_operate(&acc, &self.seg[k]);
            if !pred(&nacc) {
                while k < self.n {
                    self.propagate_at(k);
                    k <<= 1;
                    let nacc = M::agg_operate(&acc, &self.seg[k]);
                    if pred(&nacc) {
                        acc = nacc;
                        k += 1;
                    }
                }
                return k - self.n;
            }
            acc = nacc;
            k += 1;
            if k.is_power_of_two() {
                return self.len;
            }
        }
    }
    pub fn rpartition_point_acc<P>(&mut self, right: usize, mut pred: P) -> usize
    where
        P: FnMut(&M::Agg) -> bool,
    {
        let mut acc = M::agg_unit();
        if right == 0 {
            return 0;
        }
        let mut k = right + self.n;
        self.propagate(k - 1);
        loop {
            k -= 1;
            while k > 1 && k & 1 != 0 {
                k >>= 1;
            }
            let nacc = M::agg_operate(&self.seg[k], &acc);
            if !pred(&nacc) {
                while k < self.n {
                    self.propagate_at(k);
                    k = 2 * k + 1;
                    let nacc = M::agg_operate(&self.seg[k], &acc);
                    if pred(&nacc) {
                        acc = nacc;
                        k -= 1;
                    }
                }
                return k + 1 - self.n;
            }
            acc = nacc;
            if k.is_power_of_two() {
                return 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{
            RangeChminChmaxAdd, RangeMaxRangeUpdate, RangeSumRangeAdd, RangeSumRangeChminChmaxAdd,
        },
        num::Saturating,
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
            match rng.rand(3) {
                0 => {
                    // Range Add Query
                    rand!(rng, x: -A..A);
                    seg.update(l..r, x);
                    for a in arr[l..r].iter_mut() {
                        *a += x;
                    }
                }
                1 => {
                    // Point Set Query
                    rand!(rng, k: 0..N, x: -A..A);
                    seg.set(k, (x, 1));
                    arr[k] = x;
                }
                _ => {
                    // Range Sum Query
                    let res = arr[l..r].iter().sum();
                    assert_eq!(seg.fold(l..r).0, res);
                }
            }
            rand!(rng, k: 0..N);
            assert_eq!(seg.get(k).0, arr[k]);
            assert_eq!(seg.fold_all().0, arr.iter().sum());
        }

        // Range Max Query & Range Update Query & Binary Search Query
        rand!(rng, mut arr: [-A..A; N]);
        let mut seg = LazySegmentTree::<RangeMaxRangeUpdate<_>>::from_vec(arr.clone());
        for _ in 0..Q {
            rand!(rng, ty: 0..5, (l, r): NotEmptySegment(N));
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
                    rand!(rng, left: ..=N, x: -A..A);
                    assert_eq!(
                        seg.partition_point_acc(left, |&d| d < x),
                        arr[left..]
                            .iter()
                            .scan(i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map_or(N, |i| i + left),
                    );
                }
                3 => {
                    // Binary Search Query
                    rand!(rng, right: ..=N, x: -A..A);
                    assert_eq!(
                        seg.rpartition_point_acc(right, |&d| d < x),
                        arr[..right]
                            .iter()
                            .rev()
                            .scan(i64::MIN, |acc, &a| {
                                *acc = a.max(*acc);
                                Some(*acc)
                            })
                            .position(|acc| acc >= x)
                            .map_or(0, |i| right - i),
                    );
                }
                _ => {
                    // Point Set Query
                    rand!(rng, k: 0..N, x: -A..A);
                    seg.set(k, x);
                    arr[k] = x;
                }
            }
            rand!(rng, k: 0..N);
            assert_eq!(seg.get(k), arr[k]);
            assert_eq!(seg.fold_all(), *arr.iter().max().unwrap());
        }

        // Range Sum Query & Range Chmin/Chmax/Add Query
        let mut arr = rng
            .random_iter(-1_000..=1_000)
            .map(Saturating)
            .take(N)
            .collect::<Vec<_>>();
        let mut seg =
            LazySegmentTree::<RangeSumRangeChminChmaxAdd<_>>::from_keys(arr.iter().copied());
        for _ in 0..Q {
            rand!(rng, ty: 0..4, (l, r): NotEmptySegment(N), x: -1_000..=1_000);
            let x = Saturating(x);
            match ty {
                0 => {
                    seg.update(l..r, RangeChminChmaxAdd::chmin(x));
                    arr[l..r].iter_mut().for_each(|a| *a = (*a).min(x));
                }
                1 => {
                    seg.update(l..r, RangeChminChmaxAdd::chmax(x));
                    arr[l..r].iter_mut().for_each(|a| *a = (*a).max(x));
                }
                2 => {
                    seg.update(l..r, RangeChminChmaxAdd::add(x));
                    arr[l..r].iter_mut().for_each(|a| *a += x);
                }
                _ => assert_eq!(
                    seg.fold(l..r).sum,
                    arr[l..r].iter().copied().sum::<Saturating<i64>>()
                ),
            }
        }
    }
}
