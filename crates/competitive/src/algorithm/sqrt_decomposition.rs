use super::{Magma, Monoid, Unital};

pub trait SqrtDecomposition: Sized {
    type M: Monoid<T = Self::T>;
    type T: Clone;
    type B;
    fn bucket(bsize: usize) -> Self::B;
    fn update_bucket(bucket: &mut Self::B, x: &Self::T);
    fn update_cell(bucket: &mut Self::B, cell: &mut Self::T, x: &Self::T);
    fn fold_bucket(bucket: &Self::B) -> Self::T;
    fn fold_cell(bucket: &Self::B, cell: &Self::T) -> Self::T;
    fn sqrt_decomposition(n: usize, bucket_size: usize) -> SqrtDecompositionBuckets<Self> {
        let mut buckets = vec![];
        for l in (0..n).step_by(bucket_size) {
            let bsize = (l + bucket_size).min(n) - l;
            let x = Self::bucket(bsize);
            buckets.push((vec![Self::M::unit(); bsize], x));
        }
        SqrtDecompositionBuckets {
            bucket_size,
            buckets,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct SqrtDecompositionBuckets<S>
where
    S: SqrtDecomposition,
{
    bucket_size: usize,
    buckets: Vec<(Vec<S::T>, S::B)>,
    _marker: std::marker::PhantomData<fn() -> S>,
}
impl<S> SqrtDecompositionBuckets<S>
where
    S: SqrtDecomposition,
{
    pub fn update(&mut self, l: usize, r: usize, x: S::T) {
        for (i, (cells, bucket)) in self.buckets.iter_mut().enumerate() {
            let s = i * self.bucket_size;
            let t = s + cells.len();
            if t <= l || r <= s {
            } else if l <= s && t <= r {
                S::update_bucket(bucket, &x);
            } else {
                for cell in &mut cells[l.max(s) - s..r.min(t) - s] {
                    S::update_cell(bucket, cell, &x);
                }
            }
        }
    }
    pub fn fold(&self, l: usize, r: usize) -> S::T {
        let mut res = S::M::unit();
        for (i, (cells, bucket)) in self.buckets.iter().enumerate() {
            let s = i * self.bucket_size;
            let t = s + cells.len();
            if t <= l || r <= s {
            } else if l <= s && t <= r {
                <S::M as Magma>::operate_assign(&mut res, &S::fold_bucket(bucket));
            } else {
                for cell in &cells[l.max(s) - s..r.min(t) - s] {
                    <S::M as Magma>::operate_assign(&mut res, &S::fold_cell(bucket, cell));
                }
            }
        }
        res
    }
}
