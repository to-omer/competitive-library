use super::{MonoidAction, Unital};
use std::mem::replace;

#[derive(Clone, Debug)]
pub struct LazySegmentTree<M>
where
    M: MonoidAction,
    M::AT: PartialEq,
{
    n: usize,
    seg: Vec<(M::MT, M::AT)>,
}

impl<M> LazySegmentTree<M>
where
    M: MonoidAction,
    M::AT: PartialEq,
{
    pub fn new(n: usize) -> Self {
        let seg = vec![(M::munit(), M::aunit()); 2 * n];
        Self { n, seg }
    }
    pub fn from_vec(v: Vec<M::MT>) -> Self {
        let n = v.len();
        let mut seg = vec![(M::munit(), M::aunit()); 2 * n];
        for (i, x) in v.into_iter().enumerate() {
            seg[i + n].0 = x;
        }
        for i in (1..n).rev() {
            seg[i].0 = M::moperate(&seg[2 * i].0, &seg[2 * i + 1].0);
        }
        Self { n, seg }
    }
    #[inline]
    fn propagate(&mut self, k: usize) {
        debug_assert!(k < self.n);
        let x = replace(&mut self.seg[k].1, M::aunit());
        if !<M::A as Unital>::is_unit(&x) {
            self.seg[2 * k].1 = M::aoperate(&self.seg[2 * k].1, &x);
            self.seg[2 * k + 1].1 = M::aoperate(&self.seg[2 * k + 1].1, &x);
            M::act_assign(&mut self.seg[k].0, &x);
        }
    }
    #[inline]
    fn thrust(&mut self, k: usize) {
        for i in (1..(k + 1).next_power_of_two().trailing_zeros()).rev() {
            self.propagate(k >> i);
        }
    }
    #[inline]
    fn reflect(&self, k: usize) -> M::MT {
        if !<M::A as Unital>::is_unit(&self.seg[k].1) {
            M::act(&self.seg[k].0, &self.seg[k].1)
        } else {
            self.seg[k].0.clone()
        }
    }
    #[inline]
    fn recalc(&mut self, mut k: usize) {
        k /= 2;
        while k > 0 {
            self.seg[k].0 = M::moperate(&self.reflect(2 * k), &self.reflect(2 * k + 1));
            k /= 2;
        }
    }
    pub fn update(&mut self, l: usize, r: usize, x: M::AT) {
        debug_assert!(l <= r);
        debug_assert!(r <= self.n);
        let mut a = l + self.n;
        let mut b = r + self.n;
        self.thrust(a);
        self.thrust(b - 1);
        while a < b {
            if a & 1 != 0 {
                self.seg[a].1 = M::aoperate(&self.seg[a].1, &x);
                a += 1;
            }
            if b & 1 != 0 {
                b -= 1;
                self.seg[b].1 = M::aoperate(&self.seg[b].1, &x);
            }
            a /= 2;
            b /= 2;
        }
        self.recalc(l + self.n);
        self.recalc(r + self.n - 1);
    }
    pub fn fold(&mut self, l: usize, r: usize) -> M::MT {
        debug_assert!(l <= r);
        debug_assert!(r <= self.n);
        let mut l = l + self.n;
        let mut r = r + self.n;
        self.thrust(l);
        self.thrust(r - 1);
        let mut vl = M::munit();
        let mut vr = M::munit();
        while l < r {
            if l & 1 != 0 {
                vl = M::moperate(&vl, &self.reflect(l));
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                vr = M::moperate(&self.reflect(r), &vr);
            }
            l /= 2;
            r /= 2;
        }
        M::moperate(&vl, &vr)
    }
    pub fn set(&mut self, k: usize, x: M::MT) {
        let k = k + self.n;
        self.thrust(k);
        self.seg[k].0 = x;
        self.seg[k].1 = M::aunit();
        self.recalc(k);
    }
    pub fn get(&mut self, k: usize) -> M::MT {
        self.fold(k, k + 1)
    }
    pub fn fold_all(&mut self) -> M::MT {
        self.fold(0, self.n)
    }
    fn bisect_perfect<P>(&mut self, mut pos: usize, mut acc: M::MT, p: P) -> (usize, M::MT)
    where
        P: Fn(&M::MT) -> bool,
    {
        while pos < self.n {
            self.propagate(pos);
            pos <<= 1;
            let nacc = M::moperate(&acc, &self.reflect(pos));
            if !p(&nacc) {
                acc = nacc;
                pos += 1;
            }
        }
        (pos - self.n, acc)
    }
    fn rbisect_perfect<P>(&mut self, mut pos: usize, mut acc: M::MT, p: P) -> (usize, M::MT)
    where
        P: Fn(&M::MT) -> bool,
    {
        while pos < self.n {
            self.propagate(pos);
            pos = pos * 2 + 1;
            let nacc = M::moperate(&self.reflect(pos), &acc);
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
        P: Fn(&M::MT) -> bool,
    {
        let mut l = l + self.n;
        let r = r + self.n;
        self.thrust(l);
        self.thrust(r - 1);
        let mut k = 0usize;
        let mut acc = M::munit();
        while l < r >> k {
            if l & 1 != 0 {
                let nacc = M::moperate(&acc, &self.reflect(l));
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
                let nacc = M::moperate(&acc, &self.reflect(r - 1));
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
        P: Fn(&M::MT) -> bool,
    {
        let mut l = l + self.n;
        let mut r = r + self.n;
        self.thrust(l);
        self.thrust(r - 1);
        let mut c = 0usize;
        let mut k = 0usize;
        let mut acc = M::munit();
        while l >> k < r {
            c <<= 1;
            if l & 1 << k != 0 {
                l += 1 << k;
                c += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                let nacc = M::moperate(&self.reflect(r), &acc);
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
                let nacc = M::moperate(&self.reflect(l), &acc);
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
        rand!(rng, mut arr: [-A..A; N]);
        let mut seg = LazySegmentTree::<RangeMaxRangeUpdate<_>>::from_vec(arr.clone());
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
