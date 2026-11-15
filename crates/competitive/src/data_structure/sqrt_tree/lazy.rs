use super::{
    Bucket, BucketOrder, BucketState, BucketValue, Buckets, LazyMapAct, LazyMapMonoidWithKeyAct,
    MapAgg, MapValue, RangeBoundsExt, ReversibleBucketState, ReversibleMapMonoid, SequenceValue,
    sort_dedup_map,
};
use std::{
    borrow::Borrow,
    marker::PhantomData,
    mem,
    ops::{Range, RangeBounds},
};

struct LazyState<L, P, O = ()>
where
    L: LazyMapMonoidWithKeyAct,
{
    agg: MapAgg<L>,
    act: LazyMapAct<L>,
    order: O,
    _marker: PhantomData<fn() -> P>,
}
impl<T, L, P, O> BucketState<T> for LazyState<L, P, O>
where
    L: LazyMapMonoidWithKeyAct,
    P: BucketValue<T, Value = L::Key>,
    O: BucketOrder,
{
    fn new(values: &[T]) -> Self {
        let agg = P::fold::<L>(L::agg_unit(), values, false);
        Self {
            agg,
            act: L::act_unit(),
            order: O::default(),
            _marker: PhantomData,
        }
    }
    fn push(&mut self, values: &mut [T]) {
        self.order.push(values);
        if !L::is_act_unit(&self.act) {
            for value in values {
                let value = P::get_mut(value);
                *value = L::act_key(value, &self.act);
            }
            self.act = L::act_unit();
        }
    }
    fn is_reversed(&self) -> bool {
        self.order.is_reversed()
    }
}

impl<T, L, P> ReversibleBucketState<T> for LazyState<L, P, bool>
where
    L: LazyMapMonoidWithKeyAct + ReversibleMapMonoid,
    P: BucketValue<T, Value = L::Key>,
{
    fn toggle(&mut self) {
        L::toggle(&mut self.agg);
        self.order ^= true;
    }
}

impl<T, L, P, O> Buckets<T, LazyState<L, P, O>>
where
    L: LazyMapMonoidWithKeyAct,
    P: BucketValue<T, Value = L::Key>,
    O: BucketOrder,
{
    fn fold(&mut self, range: Range<usize>) -> MapAgg<L> {
        let mut result = L::agg_unit();
        assert!(range.start <= range.end);
        if range.is_empty() {
            return result;
        }
        let (start, index) = self.locate(range.start);
        let mut offset = range.start - index;
        for Bucket { values, state } in &mut self.buckets[start..] {
            let end = offset + values.len();
            if range.end <= offset {
                break;
            }
            if range.start <= offset && end <= range.end {
                L::agg_operate_assign(&mut result, &state.agg);
            } else if range.start < end {
                let l = range.start.saturating_sub(offset);
                let r = range.end.min(end) - offset;
                let fold = |initial, values: &[T], reversed| {
                    if reversed {
                        P::fold::<L>(initial, &values[values.len() - r..values.len() - l], true)
                    } else {
                        P::fold::<L>(initial, &values[l..r], false)
                    }
                };
                if L::is_act_unit(&state.act) {
                    result = fold(result, values, state.order.is_reversed());
                } else {
                    let agg = fold(L::agg_unit(), values, state.order.is_reversed());
                    if let Some(agg) = L::act_agg(&agg, &state.act) {
                        L::agg_operate_assign(&mut result, &agg);
                    } else {
                        state.push(values);
                        result = fold(result, values, false);
                    }
                }
            }
            offset = end;
        }
        result
    }
    fn update(&mut self, range: Range<usize>, act: LazyMapAct<L>) {
        assert!(range.start <= range.end);
        if range.is_empty() || L::is_act_unit(&act) {
            return;
        }
        let (start, index) = self.locate(range.start);
        let mut offset = range.start - index;
        for Bucket { values, state } in &mut self.buckets[start..] {
            let end = offset + values.len();
            if range.end <= offset {
                break;
            }
            if range.start <= offset
                && end <= range.end
                && let Some(agg) = L::act_agg(&state.agg, &act)
            {
                state.agg = agg;
                L::act_operate_assign(&mut state.act, &act);
                offset = end;
                continue;
            }
            if range.start < end {
                state.order.push(values);
                let l = range.start.saturating_sub(offset);
                let r = range.end.min(end) - offset;
                if L::is_act_unit(&state.act) {
                    for value in &mut values[l..r] {
                        let value = P::get_mut(value);
                        *value = L::act_key(value, &act);
                    }
                    *state = LazyState::new(values);
                    offset = end;
                    continue;
                }
                let combined = L::act_operate(&state.act, &act);
                let (prefix, rest) = values.split_at_mut(l);
                let (middle, suffix) = rest.split_at_mut(r - l);
                let mut agg = L::agg_unit();
                for (values, action) in [
                    (prefix, &state.act),
                    (middle, &combined),
                    (suffix, &state.act),
                ] {
                    if L::is_act_unit(action) {
                        for value in values {
                            agg = L::agg_operate_key(&agg, P::get(value));
                        }
                    } else {
                        let (chunks, rest) = values.as_chunks_mut::<4>();
                        for chunk in chunks {
                            for value in &mut *chunk {
                                let value = P::get_mut(value);
                                *value = L::act_key(value, action);
                            }
                            agg = P::fold::<L>(agg, chunk, false);
                        }
                        for value in rest {
                            let value = P::get_mut(value);
                            *value = L::act_key(value, action);
                            agg = L::agg_operate_key(&agg, value);
                        }
                    }
                }
                state.agg = agg;
                state.act = L::act_unit();
            }
            offset = end;
        }
    }
}

/// A dynamic sequence with ordered range aggregation and lazy range updates.
/// With O(1) monoid operations and successful `act_agg`, range operations take O(sqrt(N)).
/// An `act_agg` returning `None` materializes that bucket; updates can then take O(N).
pub struct SqrtLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    inner: Buckets<L::Key, LazyState<L, SequenceValue>>,
}
impl<L> SqrtLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
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
    pub fn get(&mut self, index: usize) -> Option<&L::Key> {
        if index >= self.len() {
            return None;
        }
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        Some(&values[j])
    }
    pub fn modify(&mut self, index: usize, f: impl FnOnce(&L::Key) -> L::Key) {
        assert!(index < self.len());
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        values[j] = f(&values[j]);
        *state = LazyState::new(values);
    }
    pub fn insert(&mut self, index: usize, value: L::Key) {
        assert!(index <= self.len());
        self.inner.insert(self.inner.locate(index), value);
    }
    pub fn remove(&mut self, index: usize) -> L::Key {
        assert!(index < self.len());
        self.inner.remove(self.inner.locate(index))
    }
    pub fn push(&mut self, value: L::Key) {
        self.inner.insert((self.inner.buckets.len(), 0), value);
    }
    pub fn pop(&mut self) -> Option<L::Key> {
        let i = self.inner.buckets.len().checked_sub(1)?;
        Some(
            self.inner
                .remove((i, self.inner.buckets[i].values.len() - 1)),
        )
    }
    /// Materializes pending updates before borrowing the elements.
    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = &L::Key> {
        for bucket in &mut self.inner.buckets {
            bucket.state.push(&mut bucket.values);
        }
        self.inner.buckets.iter().flat_map(|b| b.values.iter())
    }
    pub fn fold<R>(&mut self, range: R) -> MapAgg<L>
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.fold(range)
    }
    pub fn update<R>(&mut self, range: R, act: LazyMapAct<L>)
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.update(range, act);
    }
}
impl<L> Default for SqrtLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn default() -> Self {
        Self::new()
    }
}
impl<L> FromIterator<L::Key> for SqrtLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = L::Key>,
    {
        let values: Vec<_> = iter.into_iter().collect();
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}
impl<L> Extend<L::Key> for SqrtLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = L::Key>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

/// A sequence with range reversal, aggregation and lazy updates.
/// Reversal is amortized O(sqrt(N)) for O(1) `toggle` and monoid operations.
/// Aggregation and updates have the same costs and fallback as `SqrtLazyVec`.
pub struct SqrtReversibleLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    inner: Buckets<L::Key, LazyState<L, SequenceValue, bool>>,
}
impl<L> SqrtReversibleLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
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
    pub fn get(&mut self, index: usize) -> Option<&L::Key> {
        if index >= self.len() {
            return None;
        }
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        Some(&values[j])
    }
    pub fn modify(&mut self, index: usize, f: impl FnOnce(&L::Key) -> L::Key) {
        assert!(index < self.len());
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        values[j] = f(&values[j]);
        *state = LazyState::new(values);
    }
    pub fn insert(&mut self, index: usize, value: L::Key) {
        assert!(index <= self.len());
        self.inner.insert(self.inner.locate(index), value);
    }
    pub fn remove(&mut self, index: usize) -> L::Key {
        assert!(index < self.len());
        self.inner.remove(self.inner.locate(index))
    }
    pub fn push(&mut self, value: L::Key) {
        self.inner.insert((self.inner.buckets.len(), 0), value);
    }
    pub fn pop(&mut self) -> Option<L::Key> {
        let i = self.inner.buckets.len().checked_sub(1)?;
        Some(
            self.inner
                .remove((i, self.inner.buckets[i].values.len() - 1)),
        )
    }
    /// Materializes pending updates before borrowing the elements.
    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = &L::Key> {
        for bucket in &mut self.inner.buckets {
            bucket.state.push(&mut bucket.values);
        }
        self.inner.buckets.iter().flat_map(|b| b.values.iter())
    }
    pub fn reverse<R>(&mut self, range: R)
    where
        L: ReversibleMapMonoid,
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.reverse(range);
    }
    pub fn fold<R>(&mut self, range: R) -> MapAgg<L>
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.fold(range)
    }
    pub fn update<R>(&mut self, range: R, act: LazyMapAct<L>)
    where
        R: RangeBounds<usize>,
    {
        let range = range
            .to_range_bounded(0, self.len())
            .expect("invalid range");
        self.inner.update(range, act);
    }
}
impl<L> Default for SqrtReversibleLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn default() -> Self {
        Self::new()
    }
}
impl<L> FromIterator<L::Key> for SqrtReversibleLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = L::Key>,
    {
        let values: Vec<_> = iter.into_iter().collect();
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}
impl<L> Extend<L::Key> for SqrtReversibleLazyVec<L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = L::Key>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

/// An ordered map aggregating and updating values in key order, without changing keys.
/// Value access can materialize a bucket and takes O(sqrt(N)).
/// Range operations have the same costs and `act_agg` fallback as `SqrtLazyVec`.
pub struct SqrtLazyMap<K, L>
where
    L: LazyMapMonoidWithKeyAct,
{
    inner: Buckets<(K, L::Key), LazyState<L, MapValue>>,
}

impl<K, L> SqrtLazyMap<K, L>
where
    L: LazyMapMonoidWithKeyAct,
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

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let p = self.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner.at(p).is_some_and(|(k, _)| k.borrow() == key)
    }

    pub fn get<Q>(&mut self, key: &Q) -> Option<&L::Key>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }

    pub fn get_key_value<Q>(&mut self, key: &Q) -> Option<(&K, &L::Key)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (i, j) = self.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner.at((i, j)).filter(|(k, _)| k.borrow() == key)?;
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        let (k, v) = &values[j];
        Some((k, v))
    }

    pub fn get_key_value_at(&mut self, index: usize) -> Option<(&K, &L::Key)> {
        if index >= self.len() {
            return None;
        }
        let (i, j) = self.inner.locate(index);
        let Bucket { values, state } = &mut self.inner.buckets[i];
        state.push(values);
        let (k, v) = &values[j];
        Some((k, v))
    }

    pub fn insert(&mut self, key: K, value: L::Key) -> Option<L::Key>
    where
        K: Ord,
    {
        let (i, j) = self.inner.partition_point(|(k, _)| k < &key);
        if self.inner.at((i, j)).is_some_and(|(k, _)| k == &key) {
            let Bucket { values, state } = &mut self.inner.buckets[i];
            state.push(values);
            let old = mem::replace(&mut values[j].1, value);
            *state = LazyState::new(values);
            return Some(old);
        }
        self.inner.insert((i, j), (key, value));
        None
    }

    /// Returns whether the key was present.
    pub fn modify<Q>(&mut self, key: &Q, f: impl FnOnce(&L::Key) -> L::Key) -> bool
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
        state.push(values);
        values[j].1 = f(&values[j].1);
        *state = LazyState::new(values);
        true
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<L::Key>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, L::Key)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let p = self.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner.at(p).filter(|(k, _)| k.borrow() == key)?;
        Some(self.inner.remove(p))
    }

    pub fn remove_at(&mut self, index: usize) -> Option<(K, L::Key)> {
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

    /// Materializes pending updates before borrowing the entries.
    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = (&K, &L::Key)> {
        for bucket in &mut self.inner.buckets {
            bucket.state.push(&mut bucket.values);
        }
        self.inner
            .buckets
            .iter()
            .flat_map(|b| b.values.iter().map(|(k, v)| (k, v)))
    }

    pub fn fold<Q, R>(&mut self, range: R) -> MapAgg<L>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let (start, end) = self.inner.bounds(range, |(k, _)| k.borrow());
        let range = self.inner.position(start)..self.inner.position(end);
        self.inner.fold(range)
    }

    pub fn update<Q, R>(&mut self, range: R, act: LazyMapAct<L>)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let (start, end) = self.inner.bounds(range, |(k, _)| k.borrow());
        let range = self.inner.position(start)..self.inner.position(end);
        self.inner.update(range, act);
    }
}

impl<K, L> Default for SqrtLazyMap<K, L>
where
    L: LazyMapMonoidWithKeyAct,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, L> FromIterator<(K, L::Key)> for SqrtLazyMap<K, L>
where
    K: Ord,
    L: LazyMapMonoidWithKeyAct,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, L::Key)>,
    {
        let mut values: Vec<_> = iter.into_iter().collect();
        sort_dedup_map(&mut values);
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}

impl<K, L> Extend<(K, L::Key)> for SqrtLazyMap<K, L>
where
    K: Ord,
    L: LazyMapMonoidWithKeyAct,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, L::Key)>,
    {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::check_buckets;
    use super::super::{SqrtReversibleFoldVec, SqrtReversibleVec};
    use super::*;
    use crate::{
        algebra::{
            BidirectionalMap, ConcatenateOperation, LazyMapMonoid, MapMonoid, RangeSumRangeLinear,
            ReversibleMapMonoid,
        },
        num::mint_basic::MInt998244353 as M,
        tools::{
            Xorshift,
            testutil::{ConcatUpdate, sequence_cases},
        },
    };
    use std::{collections::BTreeMap, ops::Bound};

    struct PartialConcat;
    impl MapMonoid for PartialConcat {
        type Key = i64;
        type AggMonoid = <ConcatUpdate as MapMonoid>::AggMonoid;
        fn single_agg(key: &i64) -> Vec<i64> {
            ConcatUpdate::single_agg(key)
        }
    }
    impl ReversibleMapMonoid for PartialConcat {
        fn toggle(agg: &mut MapAgg<Self>) {
            agg.reverse();
        }
    }
    impl LazyMapMonoid for PartialConcat {
        type ActMonoid = <ConcatUpdate as LazyMapMonoid>::ActMonoid;

        fn act_agg(agg: &Vec<i64>, act: &Option<i64>) -> Option<Vec<i64>> {
            if agg.len() > 1 && !agg.len().is_power_of_two() {
                None
            } else {
                ConcatUpdate::act_agg(agg, act)
            }
        }
    }
    impl LazyMapMonoidWithKeyAct for PartialConcat {
        fn act_key(key: &i64, act: &Option<i64>) -> i64 {
            ConcatUpdate::act_key(key, act)
        }
    }

    #[test]
    fn reversible_sequence() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -10i64..=10, 0..=128).collect();
        for mut values in cases {
            let mut seq: SqrtReversibleLazyVec<ConcatUpdate> = values.iter().copied().collect();
            let mut fallback: SqrtReversibleLazyVec<PartialConcat> =
                values.iter().copied().collect();
            let mid = values.len() / 2;
            seq.update(..mid, Some(-11));
            fallback.update(..mid, Some(-11));
            values[..mid].fill(-11);
            let mut plain: SqrtReversibleVec<_> = values.iter().copied().collect();
            let mut fold: SqrtReversibleFoldVec<BidirectionalMap<ConcatenateOperation<i64>>> =
                values.iter().map(|&x| vec![x, -x]).collect();
            let n = values.len();
            let b = seq.inner.bucket_size.min(n);
            let inner = n.min(1)..n.saturating_sub(1).max(n.min(1));
            for range in [0..0, 0..n.min(1), 0..n, 0..b, b..n, inner] {
                seq.reverse(range.clone());
                fallback.reverse(range.clone());
                plain.reverse(range.clone());
                fold.reverse(range.clone());
                values[range].reverse();
                assert_eq!(seq.fold(..), values);
                assert_eq!(fallback.fold(..), values);
                assert!(plain.iter().eq(values.iter()));
                assert_eq!(
                    fold.fold(..).0,
                    values.iter().flat_map(|&x| [x, -x]).collect::<Vec<_>>()
                );
            }
            for step in 0..512 {
                let i = rng.random(0..=values.len());
                let r = rng.random(i..=values.len());
                let x = rng.random(-10..=10);
                match step % 8 {
                    0 => {
                        seq.update(i..r, Some(x));
                        fallback.update(i..r, Some(x));
                        for j in i..r {
                            *plain.get_mut(j).unwrap() = x;
                            fold.modify(j, |_| vec![x, -x]);
                        }
                        values[i..r].fill(x);
                    }
                    1 | 2 | 7 => {
                        let range = if step % 8 == 1 { 0..values.len() } else { i..r };
                        seq.reverse(range.clone());
                        fallback.reverse(range.clone());
                        plain.reverse(range.clone());
                        fold.reverse(range.clone());
                        values[range].reverse();
                    }
                    3 => {
                        seq.insert(i, x);
                        fallback.insert(i, x);
                        plain.insert(i, x);
                        fold.insert(i, vec![x, -x]);
                        values.insert(i, x);
                    }
                    4 if i < values.len() => {
                        let value = values.remove(i);
                        assert_eq!(seq.remove(i), value);
                        assert_eq!(fallback.remove(i), value);
                        assert_eq!(plain.remove(i), value);
                        assert_eq!(fold.remove(i), [value, -value]);
                    }
                    5 if i < values.len() => {
                        seq.modify(i, |_| x);
                        fallback.modify(i, |_| x);
                        *plain.get_mut(i).unwrap() = x;
                        fold.modify(i, |_| vec![x, -x]);
                        values[i] = x;
                    }
                    _ => {
                        assert_eq!(seq.fold(i..r), values[i..r]);
                        assert_eq!(fallback.fold(i..r), values[i..r]);
                        let (forward, backward) = fold.fold(i..r);
                        assert_eq!(
                            forward,
                            values[i..r]
                                .iter()
                                .flat_map(|&x| [x, -x])
                                .collect::<Vec<_>>()
                        );
                        assert_eq!(
                            backward,
                            values[i..r]
                                .iter()
                                .rev()
                                .flat_map(|&x| [x, -x])
                                .collect::<Vec<_>>()
                        );
                        assert_eq!(seq.get(i), values.get(i));
                        assert_eq!(plain.get(i), values.get(i));
                        assert_eq!(fold.get(i).map(|v| v[0]), values.get(i).copied());
                    }
                }
                check_buckets(&seq.inner);
                check_buckets(&fallback.inner);
                assert_eq!(plain.first(), values.first());
                assert_eq!(plain.last(), values.last());
                if step % 31 == 0 {
                    assert!(plain.iter().rev().eq(values.iter().rev()));
                    assert!(fold.iter().map(|v| &v[0]).eq(values.iter()));
                    assert!(seq.iter().eq(values.iter()));
                    assert!(fallback.iter().rev().eq(values.iter().rev()));
                }
            }
            seq.reverse(..);
            fallback.reverse(..);
            plain.reverse(..);
            fold.reverse(..);
            values.reverse();
            while let Some(x) = values.pop() {
                assert_eq!(seq.pop(), Some(x));
                assert_eq!(fallback.pop(), Some(x));
                assert_eq!(plain.pop(), Some(x));
                assert_eq!(fold.pop(), Some(vec![x, -x]));
            }
            assert_eq!(seq.pop(), None);
            assert_eq!(plain.pop(), None);
            assert_eq!(fold.pop(), None);
            seq.extend([1, 2]);
            plain.extend([1, 2]);
            fold.extend([vec![1, -1], vec![2, -2]]);
            assert_eq!(seq.len(), 2);
            assert_eq!(plain.len(), 2);
            assert_eq!(fold.len(), 2);
            seq.clear();
            plain.clear();
            fold.clear();
            assert!(seq.is_empty() && plain.is_empty() && fold.is_empty());
        }
        let mut values: Vec<_> = (0..1024).collect();
        let mut seq: SqrtReversibleLazyVec<ConcatUpdate> = values.iter().copied().collect();
        let mut rebuilt = false;
        for _ in 0..seq.inner.buckets.len() * 2 {
            let before = seq.inner.buckets.len();
            let first = seq.inner.sizes[0] - 1;
            let last = seq
                .inner
                .sizes
                .iter()
                .skip(1)
                .position(|&size| size >= seq.inner.bucket_size)
                .unwrap()
                + 1;
            let end = seq.inner.sizes[..=last].iter().sum();
            seq.reverse(first..end);
            values[first..end].reverse();
            rebuilt |= seq.inner.buckets.len() < before;
            check_buckets(&seq.inner);
            assert_eq!(seq.fold(..), values);
        }
        assert!(rebuilt);

        let mut fallback: SqrtReversibleLazyVec<PartialConcat> = (0..64).collect();
        fallback.update(.., Some(7));
        assert!(fallback.inner.buckets.iter().any(|b| b.state.act.is_some()));
        fallback.reverse(..);
        assert_eq!(fallback.fold(1..4), [7; 3]);

        struct Key(i32);
        let mut plain: SqrtReversibleVec<_> = (0..128).map(Key).collect();
        plain.reverse(..);
        for (i, key) in plain.iter_mut().enumerate() {
            assert_eq!(key.0, 127 - i as i32);
            key.0 += 1;
        }
        assert_eq!(plain.remove(0).0, 128);
    }

    #[test]
    fn lazy_sequence() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -10i64..=10, 0..=128).collect();
        for mut values in cases {
            let mut map: SqrtLazyMap<_, ConcatUpdate> =
                values.iter().copied().zip(0i64..).collect();
            let mut entries: BTreeMap<_, _> = values.iter().copied().zip(0i64..).collect();
            let mut seq: SqrtLazyVec<ConcatUpdate> = values.iter().copied().collect();
            let mut fallback: SqrtLazyVec<PartialConcat> = values.iter().copied().collect();
            let mut affine: SqrtLazyVec<RangeSumRangeLinear<M>> =
                values.iter().copied().map(M::from).collect();
            let mut numbers: Vec<M> = values.iter().copied().map(M::from).collect();
            for step in 0..512 {
                let i = rng.random(0..=values.len());
                let x = rng.random(-10..=10);
                let key = rng.random(-256..=256);
                match step % 8 {
                    0 => {
                        map.update(.., Some(x));
                        for value in entries.values_mut() {
                            *value = x;
                        }
                    }
                    1 => {
                        map.update((Bound::Excluded(key), Bound::Unbounded), Some(x));
                        for (_, value) in
                            entries.range_mut((Bound::Excluded(key), Bound::Unbounded))
                        {
                            *value = x;
                        }
                    }
                    2 => assert_eq!(map.insert(key, x), entries.insert(key, x)),
                    3 => assert_eq!(map.remove(&key), entries.remove(&key)),
                    4 => {
                        assert_eq!(map.modify(&key, |_| x), entries.contains_key(&key));
                        if let Some(value) = entries.get_mut(&key) {
                            *value = x;
                        }
                    }
                    5 => {
                        let j = rng.random(0..=entries.len());
                        let entry = entries.iter().nth(j).map(|(&k, &v)| (k, v));
                        assert_eq!(map.remove_at(j), entry);
                        if let Some((k, _)) = entry {
                            entries.remove(&k);
                        }
                    }
                    6 => {
                        assert_eq!(
                            map.fold(..=key),
                            entries.range(..=key).map(|(_, &v)| v).collect::<Vec<_>>()
                        );
                        assert_eq!(map.fold(key..key), Vec::<i64>::new());
                    }
                    _ => {
                        assert_eq!(map.get(&key), entries.get(&key));
                        let j = rng.random(0..=entries.len());
                        assert_eq!(map.get_key_value_at(j), entries.iter().nth(j));
                    }
                }
                assert_eq!(map.len(), entries.len());
                assert_eq!(map.contains_key(&key), entries.contains_key(&key));
                assert_eq!(map.rank(&key), entries.range(..key).count());
                assert_eq!(map.rank_le(&key), entries.range(..=key).count());
                check_buckets(&map.inner);
                match step % 8 {
                    0 | 1 => {
                        let (l, r) = if step % 8 == 0 {
                            (0, values.len())
                        } else {
                            (i, rng.random(i..=values.len()))
                        };
                        seq.update(l..r, Some(x));
                        fallback.update(l..r, Some(x));
                        values[l..r].fill(x);
                        let a = M::from(rng.random(-2i64..=2));
                        affine.update(l..r, (a, M::from(x)));
                        for v in &mut numbers[l..r] {
                            *v = a * *v + M::from(x);
                        }
                    }
                    2 => {
                        seq.insert(i, x);
                        fallback.insert(i, x);
                        values.insert(i, x);
                        affine.insert(i, M::from(x));
                        numbers.insert(i, M::from(x));
                    }
                    3 if i < values.len() => {
                        assert_eq!(seq.remove(i), values[i]);
                        assert_eq!(fallback.remove(i), values.remove(i));
                        assert_eq!(affine.remove(i), numbers.remove(i));
                    }
                    4 if i < values.len() => {
                        seq.modify(i, |_| x);
                        fallback.modify(i, |_| x);
                        values[i] = x;
                        affine.modify(i, |_| M::from(x));
                        numbers[i] = M::from(x);
                    }
                    _ => {
                        let r = rng.random(i..=values.len());
                        assert_eq!(seq.fold(i..r), values[i..r]);
                        assert_eq!(fallback.fold(i..r), values[i..r]);
                        assert_eq!(
                            affine.fold(i..r),
                            (numbers[i..r].iter().copied().sum(), M::from(r - i))
                        );
                    }
                }
                // Leave most buckets lazy across successive updates and structural changes.
                if step % 31 == 0 {
                    assert!(map.iter().eq(entries.iter()));
                    assert!(seq.iter().eq(values.iter()));
                    assert!(fallback.iter().eq(values.iter()));
                    assert!(affine.iter().eq(numbers.iter()));
                }
                check_buckets(&seq.inner);
            }
            assert_eq!(seq.fold(..), values);
            assert_eq!(fallback.fold(..), values);
            assert_eq!(map.fold(..), entries.into_values().collect::<Vec<_>>());
            while let Some(value) = values.pop() {
                assert_eq!(seq.pop(), Some(value));
            }
            assert!(seq.is_empty());
        }
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct Key(i32);
        let mut map: SqrtLazyMap<_, ConcatUpdate> =
            [(Key(1), 7), (Key(2), 9)].into_iter().collect();
        map.update(.., Some(3));
        assert_eq!(map.insert(Key(2), 4), Some(3));
        assert_eq!(map.fold(..), [3, 4]);
        assert_eq!(map.remove(&Key(1)), Some(3));
    }
}
