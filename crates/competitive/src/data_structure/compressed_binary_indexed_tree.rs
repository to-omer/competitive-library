use super::{Monoid, SliceBisectExt};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

pub struct CompressedBinaryIndexedTree<M, X, Inner>
where
    M: Monoid,
{
    compress: Vec<X>,
    bits: Vec<Inner>,
    _marker: PhantomData<fn() -> M>,
}
impl<M, X, Inner> Debug for CompressedBinaryIndexedTree<M, X, Inner>
where
    M: Monoid,
    X: Debug,
    Inner: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompressedBinaryIndexedTree")
            .field("compress", &self.compress)
            .field("bits", &self.bits)
            .finish()
    }
}
impl<M, X, Inner> Clone for CompressedBinaryIndexedTree<M, X, Inner>
where
    M: Monoid,
    X: Clone,
    Inner: Clone,
{
    fn clone(&self) -> Self {
        Self {
            compress: self.compress.clone(),
            bits: self.bits.clone(),
            _marker: self._marker,
        }
    }
}
impl<M, X, Inner> Default for CompressedBinaryIndexedTree<M, X, Inner>
where
    M: Monoid,
{
    fn default() -> Self {
        Self {
            compress: Default::default(),
            bits: Default::default(),
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

macro_rules! impl_compressed_binary_indexed_tree {
    (@tuple ($($l:tt)*) ($($r:tt)*) $T:ident) => {
        ($($l)* $T $($r)*,)
    };
    (@tuple ($($l:tt)*) ($($r:tt)*) $T:ident $($Rest:ident)+) => {
        ($($l)* $T $($r)*, impl_compressed_binary_indexed_tree!(@tuple ($($l)*) ($($r)*) $($Rest)+))
    };
    (@cst $M:ident) => {
        Tag<$M>
    };
    (@cst $M:ident $T:ident $($Rest:ident)*) => {
        CompressedBinaryIndexedTree<$M, $T, impl_compressed_binary_indexed_tree!(@cst $M $($Rest)*)>
    };
    (@from_iter $M:ident $points:ident $T:ident) => {{
        let mut compress: Vec<_> = $points.into_iter().map(|t| t.0.clone()).collect();
        compress.sort_unstable();
        compress.dedup();
        let n = compress.len();
        Self {
            compress,
            bits: vec![Tag(M::unit()); n + 1],
            _marker: PhantomData,
        }
    }};
    (@from_iter $M:ident $points:ident $T:ident $U:ident $($Rest:ident)*) => {{
        let mut compress: Vec<_> = $points.clone().into_iter().map(|t| t.0.clone()).collect();
        compress.sort_unstable();
        compress.dedup();
        let n = compress.len();
        let mut bits = vec![CompressedBinaryIndexedTree::default(); n + 1];
        let mut ps = vec![vec![]; n + 1];
        for (x, q) in $points {
            let i = compress.position_bisect(|c| x <= c);
            ps[i + 1].push(q);
        }
        for i in 1..=n {
            bits[i] = CompressedBinaryIndexedTree::<_, _, impl_compressed_binary_indexed_tree!(@cst $M $($Rest)*)>::from_iter(ps[i].iter().cloned());
            let j = i + (i & (!i + 1));
            if j <= n {
                let [s, ns] = ps.get_disjoint_mut([i, j]).unwrap();
                ns.append(s);
            }
        }
        Self {
            compress,
            bits,
            _marker: PhantomData,
        }
    }};
    (@acc $e:expr, $rng:ident $T:ident) => {
        $e.0
    };
    (@acc $e:expr, $rng:ident $T:ident $($Rest:ident)+) => {
        $e.accumulate(&$rng.1)
    };
    (@update $e:expr, $M:ident $key:ident $x:ident $T:ident) => {
        $M::operate_assign(&mut $e.0, $x);
    };
    (@update $e:expr, $M:ident $key:ident $x:ident $T:ident $($Rest:ident)+) => {
        $e.update(&$key.1, $x);
    };
    (@impl $C:ident $($T:ident)*, $($Q:ident)*) => {
        impl<M, $($T,)*> impl_compressed_binary_indexed_tree!(@cst M $($T)*)
        where
            M: Monoid,
            $($T: Clone + Ord,)*
        {
            pub fn new(points: &[impl_compressed_binary_indexed_tree!(@tuple () () $($T)*)]) -> Self {
                Self::from_iter(points)
            }
            fn from_iter<'a, Iter>(points: Iter) -> Self
            where
                $($T: 'a,)*
                Iter: IntoIterator<Item = &'a impl_compressed_binary_indexed_tree!(@tuple () () $($T)*)> + Clone,
            {
                impl_compressed_binary_indexed_tree!(@from_iter M points $($T)*)
            }
            pub fn accumulate<$($Q,)*>(&self, range: &impl_compressed_binary_indexed_tree!(@tuple () () $($Q)*)) -> M::T
            where
                $($Q: RangeBounds<$T>,)*
            {
                match range.0.start_bound() {
                    Bound::Unbounded => (),
                    _ => panic!("expected `Bound::Unbounded`"),
                };
                let mut k = match range.0.end_bound() {
                    Bound::Included(index) => self.compress.position_bisect(|x| x > &index),
                    Bound::Excluded(index) => self.compress.position_bisect(|x| x >= &index),
                    Bound::Unbounded => self.compress.len(),
                };
                let mut x = M::unit();
                while k > 0 {
                    x = M::operate(&x, &impl_compressed_binary_indexed_tree!(@acc self.bits[k], range $($T)*));
                    k -= k & (!k + 1);
                }
                x
            }
            pub fn update(&mut self, key: &impl_compressed_binary_indexed_tree!(@tuple () () $($T)*), x: &M::T) {
                let mut k = self.compress.binary_search(&key.0).expect("not exist key") + 1;
                while k < self.bits.len() {
                    impl_compressed_binary_indexed_tree!(@update self.bits[k], M key x $($T)*);
                    k += k & (!k + 1);
                }
            }
        }
        pub type $C<M, $($T),*> = impl_compressed_binary_indexed_tree!(@cst M $($T)*);
    };
    (@inner [$C:ident][$($T:ident)*][$($Q:ident)*][]) => {
        impl_compressed_binary_indexed_tree!(@impl $C $($T)*, $($Q)*);
    };
    (@inner [$C:ident][$($T:ident)*][$($Q:ident)*][$D:ident $U:ident $R:ident $($Rest:ident)*]) => {
        impl_compressed_binary_indexed_tree!(@impl $C $($T)*, $($Q)*);
        impl_compressed_binary_indexed_tree!(@inner [$D][$($T)* $U][$($Q)* $R][$($Rest)*]);
    };
    ($C:ident $T:ident $Q:ident $($Rest:ident)* $(;$($t:tt)*)?) => {
        impl_compressed_binary_indexed_tree!(@inner [$C][$T][$Q][$($Rest)*]);
    };
    ($($t:tt)*) => {
        compile_error!($($t:tt)*)
    }
}

impl_compressed_binary_indexed_tree!(
    CompressedBinaryIndexedTree1d A QA
    CompressedBinaryIndexedTree2d B QB
    CompressedBinaryIndexedTree3d C QC
    CompressedBinaryIndexedTree4d D QD;
    CompressedBinaryIndexedTree5d E QE
    CompressedBinaryIndexedTree6d F QF
    CompressedBinaryIndexedTree7d G QG
    CompressedBinaryIndexedTree8d H QH
    CompressedBinaryIndexedTree9d I QI
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AdditiveOperation, tools::Xorshift};
    use std::{collections::HashMap, ops::RangeTo};

    #[test]
    fn test_bit4d() {
        let mut rng = Xorshift::default();
        const N: usize = 100;
        const Q: usize = 5000;
        const A: RangeTo<u64> = ..1_000;
        let mut points: Vec<_> = rng.random_iter(((A), (A, (A, (A,))))).take(N).collect();
        points.sort();
        points.dedup();
        let mut map: HashMap<_, _> = points.iter().map(|p| (p, 0u64)).collect();
        let mut bit =
            CompressedBinaryIndexedTree4d::<AdditiveOperation<u64>, _, _, _, _>::new(&points);
        for _ in 0..Q {
            let p = &points[rng.random(0..points.len())];
            let x = rng.random(A);
            *map.get_mut(p).unwrap() += x;
            bit.update(p, &x);

            let mut f = || {
                (
                    Bound::Unbounded,
                    match rng.rand(3) {
                        0 => Bound::Excluded(rng.random(A)),
                        1 => Bound::Included(rng.random(A)),
                        _ => Bound::Unbounded,
                    },
                )
            };

            let range = (f(), (f(), (f(), (f(),))));
            let (r0, (r1, (r2, (r3,)))) = range;
            let expected: u64 = map
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
            let result = bit.accumulate(&range);
            assert_eq!(expected, result);
        }
    }
}
