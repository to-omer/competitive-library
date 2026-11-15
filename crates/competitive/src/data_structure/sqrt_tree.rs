use super::{
    EmptyActLazy, LazyMapAct, LazyMapMonoidWithKeyAct, MapAgg, MapMonoid, Monoid, RangeBoundsExt,
    ReversibleMapMonoid,
};
use std::{
    mem,
    ops::{Bound, Range, RangeBounds},
};

pub use self::fold::{SqrtFoldMap, SqrtFoldVec, SqrtReversibleFoldVec};
pub use self::lazy::{SqrtLazyMap, SqrtLazyVec, SqrtReversibleLazyVec};
pub use self::plain::{
    SqrtIntoIter, SqrtIter, SqrtMap, SqrtMultiset, SqrtReversibleVec, SqrtSet, SqrtVec,
};

mod fold;
mod lazy;
mod plain;

trait BucketState<T>: Sized {
    fn new(values: &[T]) -> Self;
    fn push(&mut self, values: &mut [T]);
    fn is_reversed(&self) -> bool {
        false
    }
}

impl<T> BucketState<T> for () {
    fn new(_: &[T]) -> Self {}
    fn push(&mut self, _: &mut [T]) {}
}

trait BucketOrder: Default {
    fn is_reversed(&self) -> bool;
    fn push<T>(&mut self, values: &mut [T]) {
        if self.is_reversed() {
            values.reverse();
            *self = Self::default();
        }
    }
}
impl BucketOrder for () {
    fn is_reversed(&self) -> bool {
        false
    }
}
impl BucketOrder for bool {
    fn is_reversed(&self) -> bool {
        *self
    }
}
impl<T> BucketState<T> for bool {
    fn new(_: &[T]) -> Self {
        false
    }
    fn push(&mut self, values: &mut [T]) {
        BucketOrder::push(self, values);
    }
    fn is_reversed(&self) -> bool {
        *self
    }
}

trait ReversibleBucketState<T>: BucketState<T> {
    fn toggle(&mut self);
}
impl<T> ReversibleBucketState<T> for bool {
    fn toggle(&mut self) {
        *self ^= true;
    }
}

#[derive(Clone, Debug)]
struct Bucket<T, S> {
    values: Vec<T>,
    state: S,
}

impl<T, S> Bucket<T, S>
where
    S: BucketState<T>,
{
    fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        let reversed = self.state.is_reversed();
        (0..self.values.len()).map(move |i| {
            &self.values[if reversed {
                self.values.len() - 1 - i
            } else {
                i
            }]
        })
    }
}

#[derive(Clone, Debug)]
struct Buckets<T, S> {
    buckets: Vec<Bucket<T, S>>,
    sizes: Vec<usize>,
    len: usize,
    base_len: usize,
    bucket_size: usize,
}

impl<T, S> Buckets<T, S>
where
    S: BucketState<T>,
{
    fn new() -> Self {
        Self {
            buckets: Vec::new(),
            sizes: Vec::new(),
            len: 0,
            base_len: 0,
            bucket_size: 1,
        }
    }

    fn build(len: usize, mut values: impl Iterator<Item = T>) -> Self {
        if len == 0 {
            return Self::new();
        }
        let bucket_size = (len - 1).isqrt() + 1;
        let mut buckets = Vec::with_capacity(len.div_ceil(bucket_size));
        let mut sizes = Vec::with_capacity(len.div_ceil(bucket_size));
        while let Some(first) = values.next() {
            let mut chunk = Vec::with_capacity(bucket_size);
            chunk.push(first);
            chunk.extend(values.by_ref().take(bucket_size - 1));
            sizes.push(chunk.len());
            let state = S::new(&chunk);
            buckets.push(Bucket {
                values: chunk,
                state,
            });
        }
        Self {
            buckets,
            sizes,
            len,
            base_len: len,
            bucket_size,
        }
    }

    fn locate(&self, mut index: usize) -> (usize, usize) {
        if index >= self.len / 2 {
            index = self.len - index;
            for (i, &size) in self.sizes.iter().enumerate().rev() {
                if index <= size {
                    return (i, size - index);
                }
                index -= size;
            }
            return (0, 0);
        }
        for (i, &size) in self.sizes.iter().enumerate() {
            if index < size {
                return (i, index);
            }
            index -= size;
        }
        (self.buckets.len(), 0)
    }

    fn insert(&mut self, (mut i, mut j): (usize, usize), value: T) {
        if self.len == 0 {
            *self = Self::build(1, std::iter::once(value));
            return;
        }
        if i == self.buckets.len() {
            i -= 1;
            j = self.buckets[i].values.len();
        }
        let Bucket { values, state } = &mut self.buckets[i];
        state.push(values);
        values.insert(j, value);
        self.sizes[i] += 1;
        if values.len() > 2 * self.bucket_size {
            let mid = values.len() / 2;
            self.split_bucket(i, mid);
        } else {
            *state = S::new(values);
        }
        self.len += 1;
        self.rebuild_if_needed();
    }

    fn remove(&mut self, (i, j): (usize, usize)) -> T {
        let Bucket { values, state } = &mut self.buckets[i];
        state.push(values);
        let value = values.remove(j);
        self.sizes[i] -= 1;
        if values.is_empty() {
            self.buckets.remove(i);
            self.sizes.remove(i);
        } else {
            *state = S::new(values);
        }
        self.len -= 1;
        self.rebuild_if_needed();
        value
    }

    fn rebuild_if_needed(&mut self) {
        // Size alone does not bound the number of sparse buckets under insert/remove churn.
        if self.len < self.base_len.div_ceil(2)
            || self.len > self.base_len.saturating_mul(2)
            || self.buckets.len() > 2 * self.base_len.div_ceil(self.bucket_size)
        {
            let values = mem::take(&mut self.buckets)
                .into_iter()
                .flat_map(|mut bucket| {
                    bucket.state.push(&mut bucket.values);
                    bucket.values
                });
            *self = Self::build(self.len, values);
        }
    }

    fn split_bucket(&mut self, i: usize, j: usize) {
        let Bucket { values, state } = &mut self.buckets[i];
        state.push(values);
        let right = values.split_off(j);
        *state = S::new(values);
        self.sizes[i] = j;
        self.sizes.insert(i + 1, right.len());
        let state = S::new(&right);
        self.buckets.insert(
            i + 1,
            Bucket {
                values: right,
                state,
            },
        );
    }

    fn partition_point(&self, mut before: impl FnMut(&T) -> bool) -> (usize, usize) {
        let i = self
            .buckets
            .partition_point(|b| before(b.values.last().unwrap()));
        let j = self
            .buckets
            .get(i)
            .map_or(0, |b| b.values.partition_point(before));
        (i, j)
    }

    fn position(&self, (i, j): (usize, usize)) -> usize {
        if i < self.buckets.len() / 2 {
            self.sizes[..i].iter().sum::<usize>() + j
        } else {
            self.len - self.sizes[i..].iter().sum::<usize>() + j
        }
    }

    fn at(&self, (i, j): (usize, usize)) -> Option<&T> {
        let bucket = self.buckets.get(i)?;
        let j = if bucket.state.is_reversed() {
            bucket.values.len().checked_sub(j + 1)?
        } else {
            j
        };
        bucket.values.get(j)
    }

    fn bounds<Q, R>(&self, range: R, key: impl Fn(&T) -> &Q) -> ((usize, usize), (usize, usize))
    where
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let start = range.start_bound();
        let end = range.end_bound();
        if let (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) =
            (start, end)
        {
            assert!(
                a <= b
                    && !(a == b
                        && matches!((start, end), (Bound::Excluded(_), Bound::Excluded(_)))),
                "invalid range"
            );
        }
        let front = match start {
            Bound::Unbounded => (0, 0),
            Bound::Included(x) => self.partition_point(|v| key(v) < x),
            Bound::Excluded(x) => self.partition_point(|v| key(v) <= x),
        };
        let back = match end {
            Bound::Unbounded => (self.buckets.len(), 0),
            Bound::Included(x) => self.partition_point(|v| key(v) <= x),
            Bound::Excluded(x) => self.partition_point(|v| key(v) < x),
        };
        (front, back)
    }
}

impl<T, S> Buckets<T, S>
where
    S: ReversibleBucketState<T>,
{
    fn reverse(&mut self, range: Range<usize>) {
        assert!(range.start <= range.end);
        if range.len() <= 1 {
            return;
        }
        let (i, l) = self.locate(range.start);
        let (mut j, mut r) = self.locate(range.end);
        if r == 0 {
            j -= 1;
            r = self.sizes[j];
        }
        if i == j {
            let Bucket { values, state } = &mut self.buckets[i];
            if l == 0 && r == values.len() {
                state.toggle();
            } else {
                state.push(values);
                values[l..r].reverse();
                *state = S::new(values);
            }
            return;
        }
        let (start, end) = if l == 0 && r == self.sizes[j] {
            (i, j + 1)
        } else {
            let (left, right) = self.buckets.split_at_mut(j);
            let left = &mut left[i];
            let right = &mut right[0];
            left.state.push(&mut left.values);
            right.state.push(&mut right.values);
            // [A B] ... [C D] becomes [A reverse(C)] ... [reverse(B) D].
            left.values[l..].reverse();
            right.values[..r].reverse();
            let n = left.values.len() - l;
            let m = n.min(r);
            left.values[l..l + m].swap_with_slice(&mut right.values[..m]);
            if n < r {
                left.values.extend(right.values.drain(n..r));
            } else {
                right.values.splice(r..r, left.values.drain(l + r..));
            }
            left.state = S::new(&left.values);
            right.state = S::new(&right.values);
            self.sizes[i] = left.values.len();
            self.sizes[j] = right.values.len();
            (i + 1, j)
        };
        self.buckets[start..end].reverse();
        self.sizes[start..end].reverse();
        for bucket in &mut self.buckets[start..end] {
            bucket.state.toggle();
        }
        for k in [j, i] {
            if self.sizes[k] > 2 * self.bucket_size {
                self.split_bucket(k, self.sizes[k] / 2);
            }
        }
        self.rebuild_if_needed();
    }
}

fn sort_dedup_map<K, V>(values: &mut Vec<(K, V)>)
where
    K: Ord,
{
    values.sort_by(|a, b| a.0.cmp(&b.0));
    // Keep the first key and last value, as repeated insertion does.
    values.dedup_by(|a, b| {
        if a.0 == b.0 {
            mem::swap(&mut a.1, &mut b.1);
            true
        } else {
            false
        }
    });
}

trait BucketValue<T> {
    type Value;
    fn get(value: &T) -> &Self::Value;
    fn get_mut(value: &mut T) -> &mut Self::Value;
    fn fold<M>(mut agg: MapAgg<M>, values: &[T], reversed: bool) -> MapAgg<M>
    where
        M: MapMonoid<Key = Self::Value>,
    {
        let combine = |agg, [a, b, c, d]: [&T; 4]| {
            let left = M::agg_operate_key(&M::single_agg(Self::get(a)), Self::get(b));
            let right = M::agg_operate_key(&M::single_agg(Self::get(c)), Self::get(d));
            M::agg_operate(&agg, &M::agg_operate(&left, &right))
        };
        if reversed {
            let (rest, chunks) = values.as_rchunks::<4>();
            for [a, b, c, d] in chunks.iter().rev() {
                agg = combine(agg, [d, c, b, a]);
            }
            rest.iter()
                .rev()
                .fold(agg, |agg, value| M::agg_operate_key(&agg, Self::get(value)))
        } else {
            let (chunks, rest) = values.as_chunks::<4>();
            for [a, b, c, d] in chunks {
                agg = combine(agg, [a, b, c, d]);
            }
            rest.iter()
                .fold(agg, |agg, value| M::agg_operate_key(&agg, Self::get(value)))
        }
    }
}
struct SequenceValue;
impl<T> BucketValue<T> for SequenceValue {
    type Value = T;
    fn get(value: &T) -> &T {
        value
    }
    fn get_mut(value: &mut T) -> &mut T {
        value
    }
}
struct MapValue;
impl<K, V> BucketValue<(K, V)> for MapValue {
    type Value = V;
    fn get(value: &(K, V)) -> &V {
        &value.1
    }
    fn get_mut(value: &mut (K, V)) -> &mut V {
        &mut value.1
    }
}

#[cfg(test)]
mod tests {
    use super::Buckets;

    pub fn check_buckets<T, S>(tree: &Buckets<T, S>) {
        assert_eq!(tree.len, tree.buckets.iter().map(|b| b.values.len()).sum());
        assert!(
            tree.sizes
                .iter()
                .copied()
                .eq(tree.buckets.iter().map(|b| b.values.len()))
        );
        assert!(
            tree.buckets
                .iter()
                .all(|b| !b.values.is_empty() && b.values.len() <= 2 * tree.bucket_size)
        );
        assert!(tree.buckets.len() <= 2 * tree.base_len.div_ceil(tree.bucket_size));
        assert!(tree.len >= tree.base_len.div_ceil(2) && tree.len <= 2 * tree.base_len);
    }
}
