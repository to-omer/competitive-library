use super::Monoid;
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
        let mut points: Vec<_> = $points.into_iter().collect();
        points.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let mut compress = Vec::new();
        let mut offsets = vec![0];
        let mut start = 0;
        while start < points.len() {
            let mut end = start + 1;
            while end < points.len() && points[end].0 == points[start].0 {
                end += 1;
            }
            compress.push(points[start].0.clone());
            offsets.push(end);
            start = end;
        }
        let n = compress.len();
        let mut bits: Vec<impl_compressed_binary_indexed_tree!(@cst $M $U $($Rest)*)> =
            vec![Default::default(); n + 1];
        for i in 1..=n {
            let start = i - (i & (!i + 1));
            bits[i] = <impl_compressed_binary_indexed_tree!(@cst $M $U $($Rest)*)>::from_iter(
                points[offsets[start]..offsets[i]]
                    .iter()
                    .map(|point| &point.1),
            );
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
    (@partition_method $T:ident, $Q:ident) => {
        pub fn partition_point_acc<P>(&self, mut pred: P) -> (Option<&$T>, M::T)
        where
            P: FnMut(&M::T) -> bool,
        {
            let n = self.compress.len();
            let mut acc = M::unit();
            let mut pos = 0;
            let mut k = n.next_power_of_two();
            if k > n {
                k >>= 1;
            }
            while k > 0 {
                if k + pos <= n {
                    let nacc = M::operate(&acc, &self.bits[k + pos].0);
                    if pred(&nacc) {
                        pos += k;
                        acc = nacc;
                    }
                }
                k >>= 1;
            }
            (self.compress.get(pos), acc)
        }
    };
    (@partition_method $T:ident $($RestT:ident)+, $Q:ident $($RestQ:ident)+) => {
        pub fn partition_point_acc<P, $($RestQ,)*>(
            &self,
            inner_ranges: &impl_compressed_binary_indexed_tree!(@tuple () () $($RestQ)*),
            mut pred: P,
        ) -> (Option<&$T>, M::T)
        where
            P: FnMut(&M::T) -> bool,
            $($RestQ: RangeBounds<$RestT>,)*
        {
            let n = self.compress.len();
            let mut acc = M::unit();
            let mut pos = 0;
            let mut k = n.next_power_of_two();
            if k > n {
                k >>= 1;
            }
            while k > 0 {
                if k + pos <= n {
                    let nacc = M::operate(
                        &acc,
                        &self.bits[k + pos].accumulate(inner_ranges),
                    );
                    if pred(&nacc) {
                        pos += k;
                        acc = nacc;
                    }
                }
                k >>= 1;
            }
            (self.compress.get(pos), acc)
        }
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
                    Bound::Included(index) => self.compress.partition_point(|x| x <= index),
                    Bound::Excluded(index) => self.compress.partition_point(|x| x < index),
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
            impl_compressed_binary_indexed_tree!(@partition_method $($T)*, $($Q)*);
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
    fn test_bit1d() {
        let mut rng = Xorshift::default();
        const N: usize = 100;
        const Q: usize = 5000;
        const A: RangeTo<u64> = ..1_000;
        let mut points: Vec<_> = rng.random_iter(A).take(N).map(|x| (x,)).collect();
        points.sort();
        points.dedup();
        let mut values: HashMap<_, _> = points.iter().map(|p| (p.0, 0u64)).collect();
        let mut bit = CompressedBinaryIndexedTree1d::<AdditiveOperation<u64>, _>::new(&points);
        for _ in 0..Q {
            let p = &points[rng.random(0..points.len())];
            let x = rng.random(A);
            *values.get_mut(&p.0).unwrap() += x;
            bit.update(p, &x);

            let range = ((
                Bound::Unbounded,
                match rng.rand(3) {
                    0 => Bound::Excluded(rng.random(A)),
                    1 => Bound::Included(rng.random(A)),
                    _ => Bound::Unbounded,
                },
            ),);
            let expected: u64 = values
                .iter()
                .filter_map(|(p, x)| RangeBounds::contains(&range.0, p).then_some(*x))
                .sum();
            assert_eq!(bit.accumulate(&range), expected);

            let target = rng.random(1..A.end * Q as u64);
            let mut expected_acc = 0;
            let mut expected_pos = None;
            for p in &points {
                let nacc = expected_acc + values[&p.0];
                if nacc < target {
                    expected_acc = nacc;
                } else {
                    expected_pos = Some(&p.0);
                    break;
                }
            }
            let result = bit.partition_point_acc(|&acc| acc < target);
            assert_eq!(result, (expected_pos, expected_acc));
        }
    }

    #[test]
    fn test_bit2d_and_4d() {
        let mut rng = Xorshift::default();
        for _ in 0..12 {
            let domain = rng.rand(128) + 1;
            let point_count = rng.rand(96) as usize + 1;
            let registered: Vec<_> = rng
                .random_iter((..domain, (..domain,)))
                .take(point_count)
                .collect();
            let mut points = registered.clone();
            points.sort_unstable();
            points.dedup();
            let mut values: HashMap<_, _> =
                points.iter().copied().map(|point| (point, 0u64)).collect();
            let mut bit =
                CompressedBinaryIndexedTree2d::<AdditiveOperation<u64>, _, _>::new(&registered);
            let query_count = rng.rand(300) as usize + 300;
            for _ in 0..query_count {
                let point = &points[rng.rand(points.len() as u64) as usize];
                let value = rng.rand(domain);
                *values.get_mut(point).unwrap() += value;
                bit.update(point, &value);

                let end_x = rng.rand(domain + 1);
                let end_y = rng.rand(domain + 1);
                let expected = values
                    .iter()
                    .filter_map(|((x, (y,)), value)| (*x < end_x && *y < end_y).then_some(*value))
                    .sum();
                assert_eq!(bit.accumulate(&(..end_x, (..end_y,))), expected);
            }
        }

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

            let target = rng.random(1..A.end * Q as u64);
            let (_, inner_ranges) = &range;
            let (r1, (r2, (r3,))) = inner_ranges;
            let mut expected_acc = 0;
            let mut expected_pos = None;
            for p0 in &bit.compress {
                let value: u64 = map
                    .iter()
                    .filter_map(|((q0, (q1, (q2, (q3,)))), x)| {
                        if q0 == p0
                            && RangeBounds::contains(r1, q1)
                            && RangeBounds::contains(r2, q2)
                            && RangeBounds::contains(r3, q3)
                        {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                    .sum();
                let nacc = expected_acc + value;
                if nacc < target {
                    expected_acc = nacc;
                } else {
                    expected_pos = Some(p0);
                    break;
                }
            }
            let (pos, acc) = bit.partition_point_acc(inner_ranges, |&acc| acc < target);
            assert_eq!((pos, acc), (expected_pos, expected_acc));
        }
    }
}
