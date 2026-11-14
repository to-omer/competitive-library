use super::{
    Bucket, BucketState, BucketValue, Buckets, LazyMapAct, LazyMapAgg, LazyMapMonoidWithKeyAct,
    MapValue, RangeBoundsExt, SequenceValue, sort_dedup_map,
};
use std::{
    borrow::Borrow,
    marker::PhantomData,
    mem,
    ops::{Range, RangeBounds},
};

struct LazyState<L, P>
where
    L: LazyMapMonoidWithKeyAct,
{
    agg: LazyMapAgg<L>,
    act: LazyMapAct<L>,
    _marker: PhantomData<fn() -> P>,
}
impl<T, L, P> BucketState<T> for LazyState<L, P>
where
    L: LazyMapMonoidWithKeyAct,
    P: BucketValue<T, Value = L::Key>,
{
    fn new(values: &[T]) -> Self {
        let agg = values.iter().fold(L::agg_unit(), |agg, value| {
            L::agg_operate(&agg, &L::single_agg(P::get(value)))
        });
        Self {
            agg,
            act: L::act_unit(),
            _marker: PhantomData,
        }
    }
    fn push(&mut self, values: &mut [T]) {
        if !L::is_act_unit(&self.act) {
            for value in values {
                let value = P::get_mut(value);
                *value = L::act_key(value, &self.act);
            }
            self.act = L::act_unit();
        }
    }
}

impl<T, L, P> Buckets<T, LazyState<L, P>>
where
    L: LazyMapMonoidWithKeyAct,
    P: BucketValue<T, Value = L::Key>,
{
    fn fold(&mut self, range: Range<usize>) -> LazyMapAgg<L> {
        let mut result = L::agg_unit();
        assert!(range.start <= range.end);
        if range.is_empty() {
            return result;
        }
        let mut offset = 0;
        for Bucket { values, state } in &mut self.buckets {
            let end = offset + values.len();
            if range.end <= offset {
                break;
            }
            if range.start <= offset && end <= range.end {
                L::agg_operate_assign(&mut result, &state.agg);
            } else if range.start < end {
                state.push(values);
                for value in
                    &values[range.start.saturating_sub(offset)..range.end.min(end) - offset]
                {
                    L::agg_operate_assign(&mut result, &L::single_agg(P::get(value)));
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
        let mut offset = 0;
        for Bucket { values, state } in &mut self.buckets {
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
                state.push(values);
                for value in
                    &mut values[range.start.saturating_sub(offset)..range.end.min(end) - offset]
                {
                    let value = P::get_mut(value);
                    *value = L::act_key(value, &act);
                }
                *state = LazyState::new(values);
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
    pub fn fold<R>(&mut self, range: R) -> LazyMapAgg<L>
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

    pub fn fold<Q, R>(&mut self, range: R) -> LazyMapAgg<L>
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
    use super::*;
    use crate::{
        algebra::{LazyMapMonoid, RangeSumRangeLinear},
        num::mint_basic::MInt998244353 as M,
        tools::{
            Xorshift,
            testutil::{ConcatUpdate, sequence_cases},
        },
    };
    use std::{collections::BTreeMap, ops::Bound};

    struct PartialConcat;
    impl LazyMapMonoid for PartialConcat {
        type Key = i64;
        type AggMonoid = <ConcatUpdate as LazyMapMonoid>::AggMonoid;
        type ActMonoid = <ConcatUpdate as LazyMapMonoid>::ActMonoid;
        fn single_agg(key: &i64) -> Vec<i64> {
            ConcatUpdate::single_agg(key)
        }
        fn act_agg(agg: &Vec<i64>, act: &Option<i64>) -> Option<Vec<i64>> {
            if agg.len() > 1 {
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
