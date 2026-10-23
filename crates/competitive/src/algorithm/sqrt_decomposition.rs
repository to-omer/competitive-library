use super::{Magma, Monoid, RangeBoundsExt, Unital};
use std::{marker::PhantomData, ops::RangeBounds};

pub trait SqrtDecomposition: Sized {
    type M: Monoid;
    type B;
    fn bucket(bsize: usize) -> Self::B;
    fn update_bucket(bucket: &mut Self::B, x: &<Self::M as Magma>::T);
    fn update_cell(
        bucket: &mut Self::B,
        cell: &mut <Self::M as Magma>::T,
        x: &<Self::M as Magma>::T,
    );
    fn fold_bucket(bucket: &Self::B) -> <Self::M as Magma>::T;
    fn fold_cell(bucket: &Self::B, cell: &<Self::M as Magma>::T) -> <Self::M as Magma>::T;
    fn sqrt_decomposition(n: usize, bucket_size: Option<usize>) -> SqrtDecompositionBuckets<Self> {
        let bucket_size = bucket_size
            .unwrap_or((n as f64).sqrt().ceil() as usize)
            .max(1);
        let mut buckets = vec![];
        for l in (0..n).step_by(bucket_size) {
            let bsize = (l + bucket_size).min(n) - l;
            let bucket = Self::bucket(bsize);
            buckets.push(Bucket {
                cells: vec![Self::M::unit(); bsize],
                bucket,
            });
        }
        SqrtDecompositionBuckets {
            n,
            bucket_size,
            buckets,
            _marker: PhantomData,
        }
    }
}

struct Bucket<T, B> {
    cells: Vec<T>,
    bucket: B,
}

pub struct SqrtDecompositionBuckets<S>
where
    S: SqrtDecomposition,
{
    n: usize,
    bucket_size: usize,
    buckets: Vec<Bucket<<S::M as Magma>::T, S::B>>,
    _marker: PhantomData<fn() -> S>,
}
impl<S> SqrtDecompositionBuckets<S>
where
    S: SqrtDecomposition,
{
    pub fn update_cell(&mut self, i: usize, x: <S::M as Magma>::T) {
        let Bucket { cells, bucket } = &mut self.buckets[i / self.bucket_size];
        let j = i % self.bucket_size;
        S::update_cell(bucket, &mut cells[j], &x);
    }
    pub fn update<R>(&mut self, range: R, x: <S::M as Magma>::T)
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        for (i, Bucket { cells, bucket }) in self.buckets.iter_mut().enumerate() {
            let s = i * self.bucket_size;
            let t = s + cells.len();
            if t <= range.start || range.end <= s {
            } else if range.start <= s && t <= range.end {
                S::update_bucket(bucket, &x);
            } else {
                for cell in &mut cells[range.start.max(s) - s..range.end.min(t) - s] {
                    S::update_cell(bucket, cell, &x);
                }
            }
        }
    }
    pub fn get(&self, i: usize) -> <S::M as Magma>::T {
        let Bucket { cells, bucket } = &self.buckets[i / self.bucket_size];
        let j = i % self.bucket_size;
        S::fold_cell(bucket, &cells[j])
    }
    pub fn fold<R>(&self, range: R) -> <S::M as Magma>::T
    where
        R: RangeBounds<usize>,
    {
        let range = range.to_range_bounded(0, self.n).expect("invalid range");
        let mut res = S::M::unit();
        for (i, Bucket { cells, bucket }) in self.buckets.iter().enumerate() {
            let s = i * self.bucket_size;
            let t = s + cells.len();
            if t <= range.start || range.end <= s {
            } else if range.start <= s && t <= range.end {
                <S::M as Magma>::operate_assign(&mut res, &S::fold_bucket(bucket));
            } else {
                for cell in &cells[range.start.max(s) - s..range.end.min(t) - s] {
                    <S::M as Magma>::operate_assign(&mut res, &S::fold_cell(bucket, cell));
                }
            }
        }
        res
    }
}

pub struct RangeUpdateRangeFoldSqrtDecomposition<M>
where
    M: Monoid,
{
    _marker: PhantomData<fn() -> M>,
}

impl<M> SqrtDecomposition for RangeUpdateRangeFoldSqrtDecomposition<M>
where
    M: Monoid,
{
    type M = M;
    // fold, lazy, size
    type B = (M::T, M::T, usize);
    fn bucket(bsize: usize) -> Self::B {
        (M::unit(), M::unit(), bsize)
    }
    fn update_bucket(bucket: &mut Self::B, x: &<Self::M as Magma>::T) {
        M::operate_assign(&mut bucket.1, x);
    }
    fn update_cell(
        bucket: &mut Self::B,
        cell: &mut <Self::M as Magma>::T,
        x: &<Self::M as Magma>::T,
    ) {
        M::operate_assign(&mut bucket.0, x);
        M::operate_assign(cell, x);
    }
    fn fold_bucket(bucket: &Self::B) -> <Self::M as Magma>::T {
        M::operate(&bucket.0, &M::pow(bucket.1.clone(), bucket.2))
    }
    fn fold_cell(bucket: &Self::B, cell: &<Self::M as Magma>::T) -> <Self::M as Magma>::T {
        M::operate(cell, &bucket.1)
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use crate::algebra::BitXorOperation;

    #[kani::proof]
    #[kani::unwind(8)]
    fn proof_range_and_point_updates() {
        let values: [u64; 5] = kani::any();
        let action: u64 = kani::any();
        let point_action: u64 = kani::any();
        let (left, right): (u8, u8) = kani::any();
        let (query_left, query_right): (u8, u8) = kani::any();
        let index: u8 = kani::any();
        let query: u8 = kani::any();
        kani::assume(left <= right && right <= 5);
        kani::assume(query_left <= query_right && query_right <= 5);
        kani::assume(index < 5 && query < 5);
        let mut buckets =
            RangeUpdateRangeFoldSqrtDecomposition::<BitXorOperation<u64>>::sqrt_decomposition(
                5,
                Some(2),
            );
        for (i, value) in values.into_iter().enumerate() {
            buckets.update_cell(i, value);
        }
        buckets.update(left as usize..right as usize, action);
        buckets.update_cell(index as usize, point_action);
        let mut expected = 0;
        let mut expected_point = 0;
        for (i, mut value) in values.into_iter().enumerate() {
            if left as usize <= i && i < right as usize {
                value ^= action;
            }
            if i == index as usize {
                value ^= point_action;
            }
            if query_left as usize <= i && i < query_right as usize {
                expected ^= value;
            }
            if i == query as usize {
                expected_point = value;
            }
        }
        kani::assert(
            buckets.fold(query_left as usize..query_right as usize) == expected,
            "For all five-element u64 XOR arrays with bucket size two, a range update followed by a point update preserves every range fold, including empty ranges and a partial final bucket",
        );
        kani::assert(
            buckets.get(query as usize) == expected_point,
            "Every point lookup combines its cell update with the pending bucket update",
        );
        kani::cover!(left == 0 && right == 4 && index == 1 && query_left == 1 && query_right == 5);
        kani::cover!(left == 1 && right == 3 && query_left == query_right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AdditiveOperation,
        rand,
        tools::{NotEmptySegment as Nes, Xorshift},
    };

    #[test]
    fn test_sqrt_decomposition() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..100, mut a: [0i64..1000; n]);
            let mut s =
                RangeUpdateRangeFoldSqrtDecomposition::<AdditiveOperation<i64>>::sqrt_decomposition(
                    n, None,
                );
            for (i, &a) in a.iter().enumerate() {
                s.update_cell(i, a);
            }
            for _ in 0..100 {
                rand!(rng, ty: 0..3, (l, r): Nes(n), x: 0i64..1000);
                match ty {
                    0 => {
                        s.update(l..r, x);
                        for a in &mut a[l..r] {
                            *a += x;
                        }
                    }
                    1 => {
                        assert_eq!(s.fold(l..r), a[l..r].iter().sum::<i64>())
                    }
                    _ => {
                        assert_eq!(s.get(l), a[l]);
                    }
                }
            }
        }
    }
}
