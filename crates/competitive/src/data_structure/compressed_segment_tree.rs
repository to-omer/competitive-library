use super::{Monoid, SliceBisectExt};
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
            let i = compress.position_bisect(|c| x <= c);
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
                    Bound::Included(index) => self.compress.position_bisect(|x| x >= &index),
                    Bound::Excluded(index) => self.compress.position_bisect(|x| x > &index),
                    Bound::Unbounded => 0,
                } + self.compress.len();
                let mut r = match range.0.end_bound() {
                    Bound::Included(index) => self.compress.position_bisect(|x| x > &index),
                    Bound::Excluded(index) => self.compress.position_bisect(|x| x >= &index),
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
}
