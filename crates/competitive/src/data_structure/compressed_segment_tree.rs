use super::Monoid;
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::swap,
    ops::{Bound, RangeBounds},
};

pub struct CompressedSegmentTree<M, X, Inner>
where
    M: Monoid,
{
    compress: Vec<X>,
    segs: Vec<Inner>,
    _marker: PhantomData<fn() -> M>,
}

impl<M, X, Inner> Debug for CompressedSegmentTree<M, X, Inner>
where
    M: Monoid,
    X: Debug,
    Inner: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompressedSegmentTree")
            .field("compress", &self.compress)
            .field("segs", &self.segs)
            .finish()
    }
}

impl<M, X, Inner> Clone for CompressedSegmentTree<M, X, Inner>
where
    M: Monoid,
    X: Clone,
    Inner: Clone,
{
    fn clone(&self) -> Self {
        Self {
            compress: self.compress.clone(),
            segs: self.segs.clone(),
            _marker: self._marker,
        }
    }
}

impl<M, X, Inner> Default for CompressedSegmentTree<M, X, Inner>
where
    M: Monoid,
{
    fn default() -> Self {
        Self {
            compress: Default::default(),
            segs: Default::default(),
            _marker: Default::default(),
        }
    }
}

#[repr(transparent)]
pub struct Tag<M>(M::T)
where
    M: Monoid;

impl<M> Debug for Tag<M>
where
    M: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<M> Clone for Tag<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

macro_rules! impl_compressed_segment_tree {
    (@tuple ($($l:tt)*) ($($r:tt)*) $T:ident) => {
        ($($l)* $T $($r)*,)
    };
    (@tuple ($($l:tt)*) ($($r:tt)*) $T:ident $($Rest:ident)+) => {
        ($($l)* $T $($r)*, impl_compressed_segment_tree!(@tuple ($($l)*) ($($r)*) $($Rest)+))
    };
    (@cst $M:ident) => {
        Tag<$M>
    };
    (@cst $M:ident $T:ident $($Rest:ident)*) => {
        CompressedSegmentTree<$M, $T, impl_compressed_segment_tree!(@cst $M $($Rest)*)>
    };
    (@from_iter $M:ident $points:ident $T:ident) => {{
        let mut compress: Vec<_> = $points.into_iter().map(|t| t.0.clone()).collect();
        compress.sort_unstable();
        compress.dedup();
        let n = compress.len();
        Self {
            compress,
            segs: vec![Tag(M::unit()); n * 2],
            _marker: PhantomData,
        }
    }};
    (@from_iter $M:ident $points:ident $T:ident $U:ident $($Rest:ident)*) => {{
        let mut compress: Vec<_> = $points.clone().into_iter().map(|t| t.0.clone()).collect();
        compress.sort_unstable();
        compress.dedup();
        let n = compress.len();
        let mut segs = vec![CompressedSegmentTree::default(); n * 2];
        let mut ps = vec![vec![]; n * 2];
        for (x, q) in $points {
            let i = compress.binary_search(x).unwrap();
            ps[i + n].push(q);
        }
        for i in (n..n * 2).rev() {
            segs[i] = CompressedSegmentTree::<_, _, impl_compressed_segment_tree!(@cst $M $($Rest)*)>::from_iter(ps[i].iter().cloned());
        }
        for i in (1..n).rev() {
            let [p, l, r] = ps.get_disjoint_mut([i, i * 2, i * 2 + 1]).unwrap();
            swap(p, l);
            p.append(r);
            segs[i] = CompressedSegmentTree::<_, _, impl_compressed_segment_tree!(@cst $M $($Rest)*)>::from_iter(ps[i].iter().cloned());
        }
        Self {
            compress,
            segs,
            _marker: PhantomData,
        }
    }};
    (@fold $e:expr, $rng:ident $T:ident) => {
        $e.0
    };
    (@fold $e:expr, $rng:ident $T:ident $($Rest:ident)+) => {
        $e.fold(&$rng.1)
    };
    (@update $e:expr, $M:ident $key:ident $x:ident $T:ident) => {
        $M::operate_assign(&mut $e.0, $x);
    };
    (@update $e:expr, $M:ident $key:ident $x:ident $T:ident $($Rest:ident)+) => {
        $e.update(&$key.1, $x);
    };
    (@partition_value $e:expr) => {
        $e.0
    };
    (@partition_value $e:expr, $inner_ranges:ident) => {
        $e.fold($inner_ranges)
    };
    (@partition_perfect_body $this:tt, $pos:expr, $acc:expr, $pred:ident $(, $inner_ranges:ident)?) => {{
        let n = $this.compress.len();
        let mut pos = $pos;
        let mut acc = $acc;
        while pos < n {
            pos <<= 1;
            let nacc = M::operate(
                &acc,
                &impl_compressed_segment_tree!(
                    @partition_value $this.segs[pos] $(, $inner_ranges)?
                ),
            );
            if $pred(&nacc) {
                acc = nacc;
                pos += 1;
            }
        }
        (pos - n, acc)
    }};
    (@rpartition_perfect_body $this:tt, $pos:expr, $acc:expr, $pred:ident $(, $inner_ranges:ident)?) => {{
        let n = $this.compress.len();
        let mut pos = $pos;
        let mut acc = $acc;
        while pos < n {
            pos = pos * 2 + 1;
            let nacc = M::operate(
                &impl_compressed_segment_tree!(
                    @partition_value $this.segs[pos] $(, $inner_ranges)?
                ),
                &acc,
            );
            if $pred(&nacc) {
                acc = nacc;
                pos -= 1;
            }
        }
        (pos - n + 1, acc)
    }};
    (@partition_body $this:tt, $left:ident, $pred:ident $(, $inner_ranges:ident)?) => {{
        let n = $this.compress.len();
        let mut l = $this.compress.partition_point(|x| x < $left) + n;
        let r = 2 * n;
        let mut k = 0usize;
        let mut acc = M::unit();
        while l < r >> k {
            if l & 1 != 0 {
                let nacc = M::operate(
                    &acc,
                    &impl_compressed_segment_tree!(
                        @partition_value $this.segs[l] $(, $inner_ranges)?
                    ),
                );
                if !$pred(&nacc) {
                    let (pos, acc) = impl_compressed_segment_tree!(
                        @partition_perfect_body $this, l, acc, $pred $(, $inner_ranges)?
                    );
                    return ($this.compress.get(pos), acc);
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
                let nacc = M::operate(
                    &acc,
                    &impl_compressed_segment_tree!(
                        @partition_value $this.segs[r - 1] $(, $inner_ranges)?
                    ),
                );
                if !$pred(&nacc) {
                    let (pos, acc) = impl_compressed_segment_tree!(
                        @partition_perfect_body $this, r - 1, acc, $pred $(, $inner_ranges)?
                    );
                    return ($this.compress.get(pos), acc);
                }
                acc = nacc;
            }
        }
        ($this.compress.get(n), acc)
    }};
    (@rpartition_body $this:tt, $right:ident, $pred:ident $(, $inner_ranges:ident)?) => {{
        let n = $this.compress.len();
        let mut l = n;
        let mut r = $this.compress.partition_point(|x| x < $right) + n;
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
                let nacc = M::operate(
                    &impl_compressed_segment_tree!(
                        @partition_value $this.segs[r] $(, $inner_ranges)?
                    ),
                    &acc,
                );
                if !$pred(&nacc) {
                    let (pos, acc) = impl_compressed_segment_tree!(
                        @rpartition_perfect_body $this, r, acc, $pred $(, $inner_ranges)?
                    );
                    return ($this.compress.get(pos), acc);
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
                let nacc = M::operate(
                    &impl_compressed_segment_tree!(
                        @partition_value $this.segs[l] $(, $inner_ranges)?
                    ),
                    &acc,
                );
                if !$pred(&nacc) {
                    let (pos, acc) = impl_compressed_segment_tree!(
                        @rpartition_perfect_body $this, l, acc, $pred $(, $inner_ranges)?
                    );
                    return ($this.compress.get(pos), acc);
                }
                acc = nacc;
            }
            c >>= 1;
        }
        ($this.compress.first(), acc)
    }};
    (@partition_methods $T:ident, $Q:ident) => {
        pub fn partition_point_acc<P>(
            &self,
            left: &$T,
            mut pred: P,
        ) -> (Option<&$T>, M::T)
        where
            P: FnMut(&M::T) -> bool,
        {
            impl_compressed_segment_tree!(@partition_body self, left, pred)
        }
        pub fn rpartition_point_acc<P>(
            &self,
            right: &$T,
            mut pred: P,
        ) -> (Option<&$T>, M::T)
        where
            P: FnMut(&M::T) -> bool,
        {
            impl_compressed_segment_tree!(@rpartition_body self, right, pred)
        }
    };
    (@partition_methods $T:ident $($RestT:ident)+, $Q:ident $($RestQ:ident)+) => {
        pub fn partition_point_acc<P, $($RestQ,)*>(
            &self,
            left: &$T,
            inner_ranges: &impl_compressed_segment_tree!(@tuple () () $($RestQ)*),
            mut pred: P,
        ) -> (Option<&$T>, M::T)
        where
            P: FnMut(&M::T) -> bool,
            $($RestQ: RangeBounds<$RestT>,)*
        {
            impl_compressed_segment_tree!(
                @partition_body self, left, pred, inner_ranges
            )
        }
        pub fn rpartition_point_acc<P, $($RestQ,)*>(
            &self,
            right: &$T,
            inner_ranges: &impl_compressed_segment_tree!(@tuple () () $($RestQ)*),
            mut pred: P,
        ) -> (Option<&$T>, M::T)
        where
            P: FnMut(&M::T) -> bool,
            $($RestQ: RangeBounds<$RestT>,)*
        {
            impl_compressed_segment_tree!(
                @rpartition_body self, right, pred, inner_ranges
            )
        }
    };
    (@impl $C:ident $($T:ident)*, $($Q:ident)*) => {
        impl<M, $($T,)*> impl_compressed_segment_tree!(@cst M $($T)*)
        where
            M: Monoid,
            $($T: Clone + Ord,)*
        {
            pub fn new(points: &[impl_compressed_segment_tree!(@tuple () () $($T)*)]) -> Self {
                Self::from_iter(points)
            }
            fn from_iter<'a, Iter>(points: Iter) -> Self
            where
                $($T: 'a,)*
                Iter: IntoIterator<Item = &'a impl_compressed_segment_tree!(@tuple () () $($T)*)> + Clone,
            {
                impl_compressed_segment_tree!(@from_iter M points $($T)*)
            }
            pub fn fold<$($Q,)*>(&self, range: &impl_compressed_segment_tree!(@tuple () () $($Q)*)) -> M::T
            where
                $($Q: RangeBounds<$T>,)*
            {
                let mut l = match range.0.start_bound() {
                    Bound::Included(index) => self.compress.partition_point(|x| x < index),
                    Bound::Excluded(index) => self.compress.partition_point(|x| x <= index),
                    Bound::Unbounded => 0,
                } + self.compress.len();
                let mut r = match range.0.end_bound() {
                    Bound::Included(index) => self.compress.partition_point(|x| x <= index),
                    Bound::Excluded(index) => self.compress.partition_point(|x| x < index),
                    Bound::Unbounded => self.compress.len(),
                } + self.compress.len();
                let mut x = M::unit();
                while l < r {
                    if l & 1 != 0 {
                        x = M::operate(&x, &impl_compressed_segment_tree!(@fold self.segs[l], range $($T)*));
                        l += 1;
                    }
                    if r & 1 != 0 {
                        r -= 1;
                        x = M::operate(&impl_compressed_segment_tree!(@fold self.segs[r], range $($T)*), &x);
                    }
                    l /= 2;
                    r /= 2;
                }
                x
            }
            pub fn update(&mut self, key: &impl_compressed_segment_tree!(@tuple () () $($T)*), x: &M::T) {
                let mut i = self.compress.binary_search(&key.0).expect("not exist key") + self.compress.len();
                while i > 0 {
                    impl_compressed_segment_tree!(@update self.segs[i], M key x $($T)*);
                    i /= 2;
                }
            }
            impl_compressed_segment_tree!(
                @partition_methods $($T)*, $($Q)*
            );
        }
        pub type $C<M, $($T),*> = impl_compressed_segment_tree!(@cst M $($T)*);
    };
    (@inner [$C:ident][$($T:ident)*][$($Q:ident)*][]) => {
        impl_compressed_segment_tree!(@impl $C $($T)*, $($Q)*);
    };
    (@inner [$C:ident][$($T:ident)*][$($Q:ident)*][$D:ident $U:ident $R:ident $($Rest:ident)*]) => {
        impl_compressed_segment_tree!(@impl $C $($T)*, $($Q)*);
        impl_compressed_segment_tree!(@inner [$D][$($T)* $U][$($Q)* $R][$($Rest)*]);
    };
    ($C:ident $T:ident $Q:ident $($Rest:ident)* $(;$($t:tt)*)?) => {
        impl_compressed_segment_tree!(@inner [$C][$T][$Q][$($Rest)*]);
    };
}

impl_compressed_segment_tree!(
    CompressedSegmentTree1d T1 Q1
    CompressedSegmentTree2d T2 Q2
    CompressedSegmentTree3d T3 Q3
    CompressedSegmentTree4d T4 Q4;
    CompressedSegmentTree5d T5 Q5
    CompressedSegmentTree6d T6 Q6
    CompressedSegmentTree7d T7 Q7
    CompressedSegmentTree8d T8 Q8
    CompressedSegmentTree9d T9 Q9
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AdditiveOperation,
        tools::{RandRange as RR, Xorshift},
    };
    use std::{collections::HashMap, ops::Range};

    #[test]
    fn test_seg1d_partition_point_acc() {
        let mut rng = Xorshift::default();
        const N: usize = 100;
        const Q: usize = 5000;
        const A: Range<u64> = 0..1_000;
        let mut points: Vec<_> = rng.random_iter(A).take(N).map(|x| (x,)).collect();
        points.sort();
        points.dedup();
        let mut values: HashMap<_, _> = points.iter().map(|p| (p.0, 0u64)).collect();
        let mut seg = CompressedSegmentTree1d::<AdditiveOperation<u64>, _>::new(&points);
        for _ in 0..Q {
            let p = &points[rng.random(0..points.len())];
            let x = rng.random(A);
            *values.get_mut(&p.0).unwrap() += x;
            seg.update(p, &x);

            let left = rng.random(A);
            let target = rng.random(1..A.end * Q as u64);
            let mut expected_acc = 0;
            let mut expected_pos = seg.compress.partition_point(|x| x < &left);
            while expected_pos < seg.compress.len() {
                let nacc = expected_acc + values[&seg.compress[expected_pos]];
                if nacc < target {
                    expected_acc = nacc;
                    expected_pos += 1;
                } else {
                    break;
                }
            }
            let (pos, acc) = seg.partition_point_acc(&left, |&acc| acc < target);
            assert_eq!((pos, acc), (seg.compress.get(expected_pos), expected_acc));

            let right = rng.random(A);
            let target = rng.random(1..A.end * Q as u64);
            let mut expected_acc = 0;
            let mut expected_pos = seg.compress.partition_point(|x| x < &right);
            while expected_pos > 0 {
                let nacc = values[&seg.compress[expected_pos - 1]] + expected_acc;
                if nacc < target {
                    expected_acc = nacc;
                    expected_pos -= 1;
                } else {
                    break;
                }
            }
            let (pos, acc) = seg.rpartition_point_acc(&right, |&acc| acc < target);
            assert_eq!((pos, acc), (seg.compress.get(expected_pos), expected_acc));
        }
    }

    #[test]
    fn test_seg4d() {
        let mut rng = Xorshift::default();
        const N: usize = 100;
        const Q: usize = 5000;
        const A: Range<i64> = -1_000..1_000;
        let mut points: Vec<_> = rng.random_iter(((A), (A, (A, (A,))))).take(N).collect();
        points.sort();
        points.dedup();
        let mut map: HashMap<_, _> = points.iter().map(|p| (p, 0i64)).collect();
        let mut seg = CompressedSegmentTree4d::<AdditiveOperation<i64>, _, _, _, _>::new(&points);
        for _ in 0..Q {
            let p = &points[rng.random(0..points.len())];
            let x = rng.random(A);
            *map.get_mut(p).unwrap() += x;
            seg.update(p, &x);

            let range = rng.random((RR::new(A), (RR::new(A), (RR::new(A), (RR::new(A),)))));
            let (r0, (r1, (r2, (r3,)))) = range;
            let expected: i64 = map
                .iter()
                .filter_map(|((p0, (p1, (p2, (p3,)))), x)| {
                    if RangeBounds::contains(&r0, p0)
                        && RangeBounds::contains(&r1, p1)
                        && RangeBounds::contains(&r2, p2)
                        && RangeBounds::contains(&r3, p3)
                    {
                        Some(*x)
                    } else {
                        None
                    }
                })
                .sum();
            let result = seg.fold(&range);
            assert_eq!(expected, result);
        }
    }

    #[test]
    fn test_seg4d_partition_point_acc() {
        let mut rng = Xorshift::default();
        const N: usize = 100;
        const Q: usize = 1000;
        const A: Range<u64> = 0..1_000;
        let mut points: Vec<_> = rng.random_iter(((A), (A, (A, (A,))))).take(N).collect();
        points.sort();
        points.dedup();
        let mut values: HashMap<_, _> = points.iter().map(|p| (p, 0u64)).collect();
        let mut seg = CompressedSegmentTree4d::<AdditiveOperation<u64>, _, _, _, _>::new(&points);
        for _ in 0..Q {
            let p = &points[rng.random(0..points.len())];
            let x = rng.random(A);
            *values.get_mut(p).unwrap() += x;
            seg.update(p, &x);

            let inner_ranges = rng.random((RR::new(A), (RR::new(A), (RR::new(A),))));
            let (r1, (r2, (r3,))) = &inner_ranges;
            let mut groups = vec![0; seg.compress.len()];
            for ((p0, (p1, (p2, (p3,)))), x) in &values {
                if RangeBounds::contains(r1, p1)
                    && RangeBounds::contains(r2, p2)
                    && RangeBounds::contains(r3, p3)
                {
                    groups[seg.compress.binary_search(p0).unwrap()] += *x;
                }
            }

            let left = rng.random(A);
            let target = rng.random(1..A.end * Q as u64);
            let mut expected_acc = 0;
            let mut expected_pos = seg.compress.partition_point(|x| x < &left);
            while expected_pos < seg.compress.len() {
                let nacc = expected_acc + groups[expected_pos];
                if nacc < target {
                    expected_acc = nacc;
                    expected_pos += 1;
                } else {
                    break;
                }
            }
            let (pos, acc) = seg.partition_point_acc(&left, &inner_ranges, |&acc| acc < target);
            assert_eq!((pos, acc), (seg.compress.get(expected_pos), expected_acc));

            let right = rng.random(A);
            let target = rng.random(1..A.end * Q as u64);
            let mut expected_acc = 0;
            let mut expected_pos = seg.compress.partition_point(|x| x < &right);
            while expected_pos > 0 {
                let nacc = groups[expected_pos - 1] + expected_acc;
                if nacc < target {
                    expected_acc = nacc;
                    expected_pos -= 1;
                } else {
                    break;
                }
            }
            let (pos, acc) = seg.rpartition_point_acc(&right, &inner_ranges, |&acc| acc < target);
            assert_eq!((pos, acc), (seg.compress.get(expected_pos), expected_acc));
        }
    }
}
