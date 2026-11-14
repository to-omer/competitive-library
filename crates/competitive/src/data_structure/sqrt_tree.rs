use super::{LazyMapAct, LazyMapAgg, LazyMapMonoidWithKeyAct, Monoid, RangeBoundsExt};
use std::{
    mem,
    ops::{Bound, RangeBounds},
};

pub use self::fold::{SqrtFoldMap, SqrtFoldVec};
pub use self::lazy::{SqrtLazyMap, SqrtLazyVec};
pub use self::plain::{SqrtIntoIter, SqrtIter, SqrtMap, SqrtMultiset, SqrtSet, SqrtVec};

mod fold;
mod lazy;
mod plain;

trait BucketState<T>: Sized {
    fn new(values: &[T]) -> Self;
    fn push(&mut self, values: &mut [T]);
}

impl<T> BucketState<T> for () {
    fn new(_: &[T]) -> Self {}
    fn push(&mut self, _: &mut [T]) {}
}

#[derive(Clone, Debug)]
struct Bucket<T, S> {
    values: Vec<T>,
    state: S,
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
            let right = values.split_off(values.len() / 2);
            self.sizes[i] = values.len();
            self.sizes.insert(i + 1, right.len());
            *state = S::new(values);
            let state = S::new(&right);
            self.buckets.insert(
                i + 1,
                Bucket {
                    values: right,
                    state,
                },
            );
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
        self.buckets.get(i).and_then(|b| b.values.get(j))
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
