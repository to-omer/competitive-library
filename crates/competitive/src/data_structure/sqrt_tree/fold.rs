use super::{
    Bucket, BucketOrder, BucketState, BucketValue, Buckets, EmptyActLazy, MapAgg, MapMonoid,
    MapValue, Monoid, RangeBoundsExt, ReversibleBucketState, ReversibleMapMonoid, SequenceValue,
    sort_dedup_map,
};
use std::{
    borrow::Borrow,
    marker::PhantomData,
    mem,
    ops::{Range, RangeBounds},
};

struct FoldState<M, P, O = ()>
where
    M: MapMonoid,
{
    agg: MapAgg<M>,
    order: O,
    _marker: PhantomData<fn() -> P>,
}

impl<T, M, P, O> BucketState<T> for FoldState<M, P, O>
where
    M: MapMonoid,
    P: BucketValue<T, Value = M::Key>,
    O: BucketOrder,
{
    fn new(values: &[T]) -> Self {
        let agg = P::fold::<M>(M::agg_unit(), values, false);
        Self {
            agg,
            order: O::default(),
            _marker: PhantomData,
        }
    }
    fn push(&mut self, values: &mut [T]) {
        self.order.push(values);
    }
    fn is_reversed(&self) -> bool {
        self.order.is_reversed()
    }
}

impl<T, M, P> ReversibleBucketState<T> for FoldState<M, P, bool>
where
    M: ReversibleMapMonoid,
    P: BucketValue<T, Value = M::Key>,
{
    fn toggle(&mut self) {
        M::toggle(&mut self.agg);
        self.order ^= true;
    }
}

impl<T, M, P, O> Buckets<T, FoldState<M, P, O>>
where
    M: MapMonoid,
    P: BucketValue<T, Value = M::Key>,
    O: BucketOrder,
{
    fn fold(&self, range: Range<usize>) -> MapAgg<M> {
        assert!(range.start <= range.end);
        let mut result = M::agg_unit();
        let (start, index) = self.locate(range.start);
        let mut offset = range.start - index;
        for Bucket { values, state } in &self.buckets[start..] {
            let end = offset + values.len();
            if range.end <= offset {
                break;
            }
            if range.start <= offset && end <= range.end {
                result = M::agg_operate(&result, &state.agg);
            } else if range.start < end {
                let l = range.start.saturating_sub(offset);
                let r = range.end.min(end) - offset;
                if state.order.is_reversed() {
                    result =
                        P::fold::<M>(result, &values[values.len() - r..values.len() - l], true);
                } else {
                    result = P::fold::<M>(result, &values[l..r], false);
                }
            }
            offset = end;
        }
        result
    }
}

/// A sequence with ordered range aggregation and no lazy updates.
/// With O(1) monoid operations, folds and amortized updates take O(sqrt(N)).
pub struct SqrtFoldVec<M>
where
    M: Monoid,
{
    inner: Buckets<M::T, FoldState<EmptyActLazy<M>, SequenceValue>>,
}

impl<M> SqrtFoldVec<M>
where
    M: Monoid,
{
    pub fn new() -> Self {
        Self {
            inner: Buckets::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.inner.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn get(&self, index: usize) -> Option<&M::T> {
        if index >= self.len() {
            return None;
        }
        self.inner.at(self.inner.locate(index))
    }
    pub fn modify(&mut self, index: usize, f: impl FnOnce(&M::T) -> M::T) {
        assert!(index < self.len());
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        values[j] = f(&values[j]);
        *state = FoldState::new(values);
    }
    pub fn insert(&mut self, index: usize, value: M::T) {
        assert!(index <= self.len());
        self.inner.insert(self.inner.locate(index), value);
    }
    pub fn remove(&mut self, index: usize) -> M::T {
        assert!(index < self.len());
        self.inner.remove(self.inner.locate(index))
    }
    pub fn push(&mut self, value: M::T) {
        self.inner.insert((self.inner.buckets.len(), 0), value);
    }
    pub fn pop(&mut self) -> Option<M::T> {
        let i = self.inner.buckets.len().checked_sub(1)?;
        Some(
            self.inner
                .remove((i, self.inner.buckets[i].values.len() - 1)),
        )
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &M::T> {
        self.inner.buckets.iter().flat_map(|b| b.values.iter())
    }
    pub fn fold<R>(&self, range: R) -> M::T
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.fold(range)
    }
}

impl<M> Default for SqrtFoldVec<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M> FromIterator<M::T> for SqrtFoldVec<M>
where
    M: Monoid,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = M::T>,
    {
        let values: Vec<_> = iter.into_iter().collect();
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}

impl<M> Extend<M::T> for SqrtFoldVec<M>
where
    M: Monoid,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = M::T>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

/// A sequence with range reversal and aggregation, without lazy updates.
/// With O(1) monoid operations and `toggle`, range reversal is amortized O(sqrt(N)).
pub struct SqrtReversibleFoldVec<M>
where
    M: MapMonoid,
{
    inner: Buckets<M::Key, FoldState<M, SequenceValue, bool>>,
}

impl<M> SqrtReversibleFoldVec<M>
where
    M: MapMonoid,
{
    pub fn new() -> Self {
        Self {
            inner: Buckets::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.inner.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn get(&self, index: usize) -> Option<&M::Key> {
        if index >= self.len() {
            return None;
        }
        self.inner.at(self.inner.locate(index))
    }
    pub fn modify(&mut self, index: usize, f: impl FnOnce(&M::Key) -> M::Key) {
        assert!(index < self.len());
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        values[j] = f(&values[j]);
        *state = FoldState::new(values);
    }
    pub fn insert(&mut self, index: usize, value: M::Key) {
        assert!(index <= self.len());
        self.inner.insert(self.inner.locate(index), value);
    }
    pub fn remove(&mut self, index: usize) -> M::Key {
        assert!(index < self.len());
        self.inner.remove(self.inner.locate(index))
    }
    pub fn push(&mut self, value: M::Key) {
        self.inner.insert((self.inner.buckets.len(), 0), value);
    }
    pub fn pop(&mut self) -> Option<M::Key> {
        let i = self.inner.buckets.len().checked_sub(1)?;
        Some(
            self.inner
                .remove((i, self.inner.buckets[i].values.len() - 1)),
        )
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &M::Key> {
        self.inner.buckets.iter().flat_map(|b| b.iter())
    }
    pub fn reverse<R>(&mut self, range: R)
    where
        M: ReversibleMapMonoid,
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.reverse(range);
    }
    pub fn fold<R>(&self, range: R) -> MapAgg<M>
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.fold(range)
    }
}

impl<M> Default for SqrtReversibleFoldVec<M>
where
    M: MapMonoid,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M> FromIterator<M::Key> for SqrtReversibleFoldVec<M>
where
    M: MapMonoid,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = M::Key>,
    {
        let values: Vec<_> = iter.into_iter().collect();
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}

impl<M> Extend<M::Key> for SqrtReversibleFoldVec<M>
where
    M: MapMonoid,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = M::Key>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

/// An ordered map with value aggregation in key order and no lazy updates.
/// Key lookup takes O(log N); folds and amortized updates take O(sqrt(N)) for O(1) monoid operations.
pub struct SqrtFoldMap<K, M>
where
    M: Monoid,
{
    inner: Buckets<(K, M::T), FoldState<EmptyActLazy<M>, MapValue>>,
}

impl<K, M> SqrtFoldMap<K, M>
where
    M: Monoid,
{
    pub fn new() -> Self {
        Self {
            inner: Buckets::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.inner.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn get<Q>(&self, key: &Q) -> Option<&M::T>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &M::T)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let p = self.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner
            .at(p)
            .filter(|(k, _)| k.borrow() == key)
            .map(|(k, v)| (k, v))
    }
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }
    pub fn get_key_value_at(&self, index: usize) -> Option<(&K, &M::T)> {
        if index >= self.len() {
            return None;
        }
        self.inner.at(self.inner.locate(index)).map(|(k, v)| (k, v))
    }
    pub fn insert(&mut self, key: K, value: M::T) -> Option<M::T>
    where
        K: Ord,
    {
        let (i, j) = self.inner.partition_point(|(k, _)| k < &key);
        if self.inner.at((i, j)).is_some_and(|(k, _)| k == &key) {
            let Bucket { values, state } = &mut self.inner.buckets[i];
            let old = mem::replace(&mut values[j].1, value);
            *state = FoldState::new(values);
            return Some(old);
        }
        self.inner.insert((i, j), (key, value));
        None
    }
    /// Returns whether the key was present.
    pub fn modify<Q>(&mut self, key: &Q, f: impl FnOnce(&M::T) -> M::T) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (i, j) = self.inner.partition_point(|(k, _)| k.borrow() < key);
        if !self
            .inner
            .at((i, j))
            .is_some_and(|(k, _)| k.borrow() == key)
        {
            return false;
        }
        let Bucket { values, state } = &mut self.inner.buckets[i];
        values[j].1 = f(&values[j].1);
        *state = FoldState::new(values);
        true
    }
    pub fn remove<Q>(&mut self, key: &Q) -> Option<M::T>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, M::T)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let p = self.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner.at(p).filter(|(k, _)| k.borrow() == key)?;
        Some(self.inner.remove(p))
    }
    pub fn remove_at(&mut self, index: usize) -> Option<(K, M::T)> {
        (index < self.len()).then(|| self.inner.remove(self.inner.locate(index)))
    }
    /// Counts keys strictly less than `key`.
    pub fn rank<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .position(self.inner.partition_point(|(k, _)| k.borrow() < key))
    }
    /// Counts keys less than or equal to `key`.
    pub fn rank_le<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .position(self.inner.partition_point(|(k, _)| k.borrow() <= key))
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&K, &M::T)> {
        self.inner
            .buckets
            .iter()
            .flat_map(|b| b.values.iter().map(|(k, v)| (k, v)))
    }
    pub fn fold<Q, R>(&self, range: R) -> M::T
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let (start, end) = self.inner.bounds(range, |(k, _)| k.borrow());
        self.inner
            .fold(self.inner.position(start)..self.inner.position(end))
    }
}

impl<K, M> Default for SqrtFoldMap<K, M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, M> FromIterator<(K, M::T)> for SqrtFoldMap<K, M>
where
    K: Ord,
    M: Monoid,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, M::T)>,
    {
        let mut values: Vec<_> = iter.into_iter().collect();
        sort_dedup_map(&mut values);
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}

impl<K, M> Extend<(K, M::T)> for SqrtFoldMap<K, M>
where
    K: Ord,
    M: Monoid,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, M::T)>,
    {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}
