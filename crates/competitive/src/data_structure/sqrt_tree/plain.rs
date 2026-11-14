use super::{Bucket, Buckets, sort_dedup_map};
use std::{
    borrow::Borrow,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem,
    ops::RangeBounds,
};

impl<T> Buckets<T, ()> {
    fn range<Q, R>(&self, range: R, key: impl Fn(&T) -> &Q) -> SqrtIter<'_, T>
    where
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let (front, back) = self.bounds(range, key);
        SqrtIter {
            buckets: &self.buckets,
            front,
            back,
        }
    }
}

/// A sequence with amortized O(sqrt(N)) insertion/removal and O(sqrt(N)) indexing.
/// An individual insertion or removal can take O(N) when rebuilding.
#[derive(Clone)]
pub struct SqrtVec<T> {
    inner: Buckets<T, ()>,
}

impl<T> SqrtVec<T> {
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

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        self.inner.at(self.inner.locate(index))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        let (i, j) = self.inner.locate(index);
        Some(&mut self.inner.buckets[i].values[j])
    }

    pub fn first(&self) -> Option<&T> {
        self.inner.buckets.first()?.values.first()
    }
    pub fn last(&self) -> Option<&T> {
        self.inner.buckets.last()?.values.last()
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());
        self.inner.insert(self.inner.locate(index), value);
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        self.inner.remove(self.inner.locate(index))
    }

    pub fn push(&mut self, value: T) {
        self.inner.insert((self.inner.buckets.len(), 0), value);
    }

    pub fn pop(&mut self) -> Option<T> {
        let i = self.inner.buckets.len().checked_sub(1)?;
        Some(
            self.inner
                .remove((i, self.inner.buckets[i].values.len() - 1)),
        )
    }

    pub fn iter(&self) -> SqrtIter<'_, T> {
        SqrtIter {
            buckets: &self.inner.buckets,
            front: (0, 0),
            back: (self.inner.buckets.len(), 0),
        }
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> {
        self.inner
            .buckets
            .iter_mut()
            .flat_map(|b| b.values.iter_mut())
    }
}

impl<T> Default for SqrtVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for SqrtVec<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let values: Vec<_> = iter.into_iter().collect();
        Self {
            inner: Buckets::build(values.len(), values.into_iter()),
        }
    }
}

impl<T> Extend<T> for SqrtVec<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T> Debug for SqrtVec<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> PartialEq for SqrtVec<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<T> Eq for SqrtVec<T> where T: Eq {}

impl<T> IntoIterator for SqrtVec<T> {
    type Item = T;
    type IntoIter = SqrtIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        SqrtIntoIter {
            buckets: self.inner.buckets.into_iter(),
            front: Vec::new().into_iter(),
            back: Vec::new().into_iter(),
            remaining: self.inner.len,
        }
    }
}
impl<'a, T> IntoIterator for &'a SqrtVec<T> {
    type Item = &'a T;
    type IntoIter = SqrtIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct SqrtIter<'a, T> {
    buckets: &'a [Bucket<T, ()>],
    front: (usize, usize),
    back: (usize, usize),
}
impl<T> Clone for SqrtIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets,
            front: self.front,
            back: self.back,
        }
    }
}
impl<'a, T> Iterator for SqrtIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let (i, j) = self.front;
        let values = &self.buckets[i].values;
        self.front = if j + 1 == values.len() {
            (i + 1, 0)
        } else {
            (i, j + 1)
        };
        Some(&values[j])
    }
}
impl<T> DoubleEndedIterator for SqrtIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        if self.back.1 == 0 {
            self.back.0 -= 1;
            self.back.1 = self.buckets[self.back.0].values.len();
        }
        self.back.1 -= 1;
        Some(&self.buckets[self.back.0].values[self.back.1])
    }
}
impl<T> FusedIterator for SqrtIter<'_, T> {}

#[derive(Clone, Debug)]
pub struct SqrtIntoIter<T> {
    buckets: std::vec::IntoIter<Bucket<T, ()>>,
    front: std::vec::IntoIter<T>,
    back: std::vec::IntoIter<T>,
    remaining: usize,
}
impl<T> Iterator for SqrtIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let value = loop {
            if let Some(value) = self.front.next() {
                break Some(value);
            }
            if let Some(bucket) = self.buckets.next() {
                self.front = bucket.values.into_iter();
            } else {
                break self.back.next();
            }
        };
        self.remaining -= usize::from(value.is_some());
        value
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<T> DoubleEndedIterator for SqrtIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        let value = loop {
            if let Some(value) = self.back.next_back() {
                break Some(value);
            }
            if let Some(bucket) = self.buckets.next_back() {
                self.back = bucket.values.into_iter();
            } else {
                break self.front.next_back();
            }
        };
        self.remaining -= usize::from(value.is_some());
        value
    }
}
impl<T> ExactSizeIterator for SqrtIntoIter<T> {}
impl<T> FusedIterator for SqrtIntoIter<T> {}

macro_rules! sqrt_set {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name<T> {
            inner: SqrtVec<T>,
        }
        impl<T> $name<T> {
            pub fn new() -> Self {
                Self {
                    inner: SqrtVec::new(),
                }
            }
            pub fn len(&self) -> usize {
                self.inner.len()
            }
            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }
            pub fn clear(&mut self) {
                self.inner.clear();
            }
            pub fn iter(&self) -> SqrtIter<'_, T> {
                self.inner.iter()
            }
            pub fn first(&self) -> Option<&T> {
                self.inner.first()
            }
            pub fn last(&self) -> Option<&T> {
                self.inner.last()
            }
            pub fn get_at(&self, index: usize) -> Option<&T> {
                self.inner.get(index)
            }
            pub fn remove_at(&mut self, index: usize) -> Option<T> {
                (index < self.len()).then(|| self.inner.remove(index))
            }
            pub fn get<Q>(&self, key: &Q) -> Option<&T>
            where
                T: Borrow<Q>,
                Q: Ord + ?Sized,
            {
                let p = self.inner.inner.partition_point(|v| v.borrow() < key);
                self.inner
                    .inner
                    .at(p)
                    .filter(|v| <T as Borrow<Q>>::borrow(v) == key)
            }
            pub fn contains<Q>(&self, key: &Q) -> bool
            where
                T: Borrow<Q>,
                Q: Ord + ?Sized,
            {
                self.get(key).is_some()
            }
            /// Removes one occurrence and returns whether it was present.
            pub fn remove<Q>(&mut self, key: &Q) -> bool
            where
                T: Borrow<Q>,
                Q: Ord + ?Sized,
            {
                let p = self.inner.inner.partition_point(|v| v.borrow() < key);
                if self.inner.inner.at(p).is_some_and(|v| v.borrow() == key) {
                    self.inner.inner.remove(p);
                    true
                } else {
                    false
                }
            }
            /// Counts elements strictly less than `key`.
            pub fn rank<Q>(&self, key: &Q) -> usize
            where
                T: Borrow<Q>,
                Q: Ord + ?Sized,
            {
                self.inner
                    .inner
                    .position(self.inner.inner.partition_point(|v| v.borrow() < key))
            }
            /// Counts elements less than or equal to `key`.
            pub fn rank_le<Q>(&self, key: &Q) -> usize
            where
                T: Borrow<Q>,
                Q: Ord + ?Sized,
            {
                self.inner
                    .inner
                    .position(self.inner.inner.partition_point(|v| v.borrow() <= key))
            }
            pub fn range<Q, R>(&self, range: R) -> SqrtIter<'_, T>
            where
                T: Borrow<Q>,
                Q: Ord + ?Sized,
                R: RangeBounds<Q>,
            {
                self.inner.inner.range(range, |v| v.borrow())
            }
        }
        impl<T> Default for $name<T> {
            fn default() -> Self {
                Self::new()
            }
        }
        impl<T> Extend<T> for $name<T>
        where
            T: Ord,
        {
            fn extend<I>(&mut self, iter: I)
            where
                I: IntoIterator<Item = T>,
            {
                for value in iter {
                    self.insert(value);
                }
            }
        }
        impl<T> IntoIterator for $name<T> {
            type Item = T;
            type IntoIter = SqrtIntoIter<T>;
            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }
        impl<'a, T> IntoIterator for &'a $name<T> {
            type Item = &'a T;
            type IntoIter = SqrtIter<'a, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
    };
}

sqrt_set!(
    SqrtSet,
    "An ordered set with O(log N) key lookup and amortized O(sqrt(N)) updates. Rank and indexing take O(sqrt(N))."
);
sqrt_set!(
    SqrtMultiset,
    "An ordered multiset. Length, rank and indexing count duplicate elements. Complexity matches `SqrtSet`."
);

impl<T> SqrtSet<T> {
    pub fn insert(&mut self, value: T) -> bool
    where
        T: Ord,
    {
        let p = self.inner.inner.partition_point(|v| v < &value);
        if self.inner.inner.at(p) == Some(&value) {
            return false;
        }
        self.inner.inner.insert(p, value);
        true
    }
}
impl<T> FromIterator<T> for SqrtSet<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut values: Vec<_> = iter.into_iter().collect();
        values.sort_unstable();
        values.dedup();
        Self {
            inner: SqrtVec {
                inner: Buckets::build(values.len(), values.into_iter()),
            },
        }
    }
}
impl<T> SqrtMultiset<T> {
    pub fn insert(&mut self, value: T)
    where
        T: Ord,
    {
        let p = self.inner.inner.partition_point(|v| v <= &value);
        self.inner.inner.insert(p, value);
    }
    pub fn count<Q>(&self, key: &Q) -> usize
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank_le(key) - self.rank(key)
    }
}
impl<T> FromIterator<T> for SqrtMultiset<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut values: Vec<_> = iter.into_iter().collect();
        values.sort_unstable();
        Self {
            inner: SqrtVec {
                inner: Buckets::build(values.len(), values.into_iter()),
            },
        }
    }
}

/// An ordered map with O(log N) key lookup and amortized O(sqrt(N)) updates.
/// Rank and indexing take O(sqrt(N)).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqrtMap<K, V> {
    inner: SqrtVec<(K, V)>,
}

impl<K, V> SqrtMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: SqrtVec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        self.inner.iter().map(|(k, v)| (k, v))
    }
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (&K, &mut V)> {
        self.inner.iter_mut().map(|(k, v)| (&*k, v))
    }
    pub fn get_key_value_at(&self, index: usize) -> Option<(&K, &V)> {
        self.inner.get(index).map(|(k, v)| (k, v))
    }
    pub fn remove_at(&mut self, index: usize) -> Option<(K, V)> {
        (index < self.len()).then(|| self.inner.remove(index))
    }
    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.inner.first().map(|(k, v)| (k, v))
    }
    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        self.inner.last().map(|(k, v)| (k, v))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let p = self.inner.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner
            .inner
            .at(p)
            .filter(|(k, _)| k.borrow() == key)
            .map(|(k, v)| (k, v))
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (i, j) = self.inner.inner.partition_point(|(k, _)| k.borrow() < key);
        let (k, v) = self.inner.inner.buckets.get_mut(i)?.values.get_mut(j)?;
        (<K as Borrow<Q>>::borrow(k) == key).then_some(v)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let (i, j) = self.inner.inner.partition_point(|(k, _)| k < &key);
        if self.inner.inner.at((i, j)).is_some_and(|(k, _)| k == &key) {
            return Some(mem::replace(
                &mut self.inner.inner.buckets[i].values[j].1,
                value,
            ));
        }
        self.inner.inner.insert((i, j), (key, value));
        None
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let p = self.inner.inner.partition_point(|(k, _)| k.borrow() < key);
        self.inner.inner.at(p).filter(|(k, _)| k.borrow() == key)?;
        Some(self.inner.inner.remove(p))
    }

    /// Counts keys strictly less than `key`.
    pub fn rank<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .inner
            .position(self.inner.inner.partition_point(|(k, _)| k.borrow() < key))
    }

    /// Counts keys less than or equal to `key`.
    pub fn rank_le<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .inner
            .position(self.inner.inner.partition_point(|(k, _)| k.borrow() <= key))
    }

    pub fn range<Q, R>(&self, range: R) -> impl DoubleEndedIterator<Item = (&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        self.inner
            .inner
            .range(range, |(k, _)| k.borrow())
            .map(|(k, v)| (k, v))
    }
}
impl<K, V> Default for SqrtMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V> FromIterator<(K, V)> for SqrtMap<K, V>
where
    K: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut values: Vec<_> = iter.into_iter().collect();
        sort_dedup_map(&mut values);
        Self {
            inner: SqrtVec {
                inner: Buckets::build(values.len(), values.into_iter()),
            },
        }
    }
}
impl<K, V> Extend<(K, V)> for SqrtMap<K, V>
where
    K: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}
impl<K, V> IntoIterator for SqrtMap<K, V> {
    type Item = (K, V);
    type IntoIter = SqrtIntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{SqrtFoldMap, SqrtFoldVec, tests::check_buckets};
    use super::*;
    use crate::{
        algebra::ConcatenateOperation,
        tools::{Xorshift, testutil::sequence_cases},
    };
    use std::{
        collections::{BTreeMap, BTreeSet, VecDeque},
        ops::Bound,
    };

    #[test]
    fn sets() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -256i64..=256, 0..=256).collect();
        for mut values in cases {
            let mut set: SqrtSet<_> = values.iter().copied().collect();
            let mut expected: BTreeSet<_> = values.iter().copied().collect();
            let mut multi: SqrtMultiset<_> = values.iter().copied().collect();
            values.sort_unstable();
            for step in 0..512 {
                let x = if step == 0 {
                    i64::MIN
                } else if step == 1 {
                    i64::MAX
                } else {
                    rng.random(-256..=256)
                };
                match rng.random(0..4) {
                    0 | 1 => {
                        assert_eq!(set.insert(x), expected.insert(x));
                        multi.insert(x);
                        values.push(x);
                        values.sort_unstable();
                    }
                    2 => {
                        assert_eq!(set.remove(&x), expected.remove(&x));
                        let pos = values.iter().position(|v| *v == x);
                        assert_eq!(multi.remove(&x), pos.is_some());
                        if let Some(i) = pos {
                            values.remove(i);
                        }
                    }
                    _ => {
                        let i = rng.random(0..=set.len());
                        let x = expected.iter().nth(i).copied();
                        assert_eq!(set.remove_at(i), x);
                        if let Some(x) = x {
                            expected.remove(&x);
                        }
                        let i = rng.random(0..=values.len());
                        assert_eq!(
                            multi.remove_at(i),
                            (i < values.len()).then(|| values.remove(i))
                        );
                    }
                }
                assert_eq!(
                    set.iter().copied().collect::<Vec<_>>(),
                    expected.iter().copied().collect::<Vec<_>>()
                );
                assert_eq!(multi.iter().copied().collect::<Vec<_>>(), values);
                assert_eq!(set.get(&x).copied(), expected.get(&x).copied());
                assert_eq!(set.contains(&x), expected.contains(&x));
                assert_eq!(multi.contains(&x), values.contains(&x));
                assert_eq!(set.rank(&x), expected.iter().filter(|&&v| v < x).count());
                assert_eq!(
                    set.rank_le(&x),
                    expected.iter().filter(|&&v| v <= x).count()
                );
                assert_eq!(multi.rank(&x), values.iter().filter(|&&v| v < x).count());
                assert_eq!(
                    multi.rank_le(&x),
                    values.iter().filter(|&&v| v <= x).count()
                );
                assert_eq!(multi.count(&x), values.iter().filter(|&&v| v == x).count());
                let i = rng.random(0..=values.len());
                assert_eq!(multi.get_at(i), values.get(i));
                assert_eq!(set.get_at(i), expected.iter().nth(i));
                let y = rng.random(-256..=256);
                let (lo, hi) = (x.min(y), x.max(y));
                let start = match step % 3 {
                    0 => Bound::Included(lo),
                    1 => Bound::Excluded(lo),
                    _ => Bound::Unbounded,
                };
                let end = match step / 3 % 3 {
                    0 => Bound::Included(hi),
                    1 if lo != hi => Bound::Excluded(hi),
                    _ => Bound::Unbounded,
                };
                assert_eq!(
                    set.range((start, end)).collect::<Vec<_>>(),
                    expected.range((start, end)).collect::<Vec<_>>()
                );
                let mut iter = multi.range((start, end));
                let mut range: VecDeque<_> =
                    values.iter().filter(|v| (start, end).contains(v)).collect();
                while !range.is_empty() {
                    if rng.random(0..2) == 0 {
                        assert_eq!(iter.next(), range.pop_front());
                    } else {
                        assert_eq!(iter.next_back(), range.pop_back());
                    }
                }
                assert_eq!(iter.next(), None);
                assert_eq!(iter.next_back(), None);
                check_buckets(&set.inner.inner);
                check_buckets(&multi.inner.inner);
            }
            assert_eq!(set.first(), expected.first());
            assert_eq!(set.last(), expected.last());
            assert_eq!(multi.first(), values.first());
            assert_eq!(multi.last(), values.last());
            assert_eq!(multi, values.iter().copied().collect());
            assert_eq!(
                multi.into_iter().rev().collect::<Vec<_>>(),
                values.into_iter().rev().collect::<Vec<_>>()
            );
            set.clear();
            assert!(set.is_empty());
        }
        let mut strings: SqrtSet<_> = ["b".to_owned(), "a".to_owned()].into_iter().collect();
        assert!(strings.contains("b"));
        assert!(strings.remove("a"));
        assert_eq!(
            strings
                .range::<str, _>((Bound::Included("a"), Bound::Unbounded))
                .next()
                .map(String::as_str),
            Some("b")
        );
    }

    #[test]
    fn map() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -128i64..=128, 0..=256).collect();
        for values in cases {
            let mut map: SqrtMap<_, _> = values.iter().copied().zip(0i64..).collect();
            let mut folded: SqrtFoldMap<_, ConcatenateOperation<i64>> = values
                .iter()
                .copied()
                .zip(0i64..)
                .map(|(k, v)| (k, vec![v]))
                .collect();
            let mut expected: BTreeMap<_, _> = values.into_iter().zip(0i64..).collect();
            for step in 0..512 {
                let key = rng.random(-256..=256);
                match rng.random(0..5) {
                    0 | 1 => {
                        let old = expected.insert(key, step);
                        assert_eq!(map.insert(key, step), old);
                        assert_eq!(folded.insert(key, vec![step]), old.map(|v| vec![v]));
                    }
                    2 => {
                        let old = expected.remove(&key);
                        assert_eq!(map.remove(&key), old);
                        assert_eq!(folded.remove(&key), old.map(|v| vec![v]));
                    }
                    3 => {
                        let i = rng.random(0..=map.len());
                        let entry = expected.iter().nth(i).map(|(&k, &v)| (k, v));
                        assert_eq!(map.remove_at(i), entry);
                        assert_eq!(folded.remove_at(i), entry.map(|(k, v)| (k, vec![v])));
                        if let Some((k, _)) = entry {
                            expected.remove(&k);
                        }
                    }
                    _ => {
                        assert_eq!(
                            folded.modify(&key, |v| vec![v[0] + 1]),
                            expected.contains_key(&key)
                        );
                        if let Some(v) = map.get_mut(&key) {
                            *v += 1;
                        }
                        if let Some(v) = expected.get_mut(&key) {
                            *v += 1;
                        }
                    }
                }
                assert_eq!(map.len(), expected.len());
                assert!(map.iter().eq(expected.iter()));
                assert_eq!(map.get(&key), expected.get(&key));
                assert_eq!(folded.get(&key).map(|v| &v[0]), expected.get(&key));
                assert_eq!(map.contains_key(&key), expected.contains_key(&key));
                assert_eq!(map.rank(&key), expected.range(..key).count());
                assert_eq!(map.rank_le(&key), expected.range(..=key).count());
                assert_eq!(folded.rank(&key), map.rank(&key));
                assert_eq!(folded.rank_le(&key), map.rank_le(&key));
                assert_eq!(
                    folded.fold(..=key),
                    expected.range(..=key).map(|(_, &v)| v).collect::<Vec<_>>()
                );
                let i = rng.random(0..=map.len());
                assert_eq!(map.get_key_value_at(i), expected.iter().nth(i));
                assert_eq!(
                    folded.get_key_value_at(i).map(|(k, v)| (k, &v[0])),
                    expected.iter().nth(i)
                );
                assert!(map.range(..=key).rev().eq(expected.range(..=key).rev()));
                check_buckets(&map.inner.inner);
            }
            assert!(folded.iter().map(|(k, v)| (k, &v[0])).eq(expected.iter()));
            for (_, v) in map.iter_mut() {
                *v += 1;
            }
            for v in expected.values_mut() {
                *v += 1;
            }
            assert_eq!(map.first_key_value(), expected.first_key_value());
            assert_eq!(map.last_key_value(), expected.last_key_value());
            assert_eq!(map, expected.clone().into_iter().collect());
            assert!(map.into_iter().eq(expected));
        }
        struct Value(i32);
        let mut map = SqrtMap::new();
        assert!(map.insert("key".to_owned(), Value(1)).is_none());
        map.get_mut("key").unwrap().0 += 1;
        assert_eq!(map.remove("key").unwrap().0, 2);
    }

    #[test]
    fn sequence() {
        let mut rng = Xorshift::default();
        let cases: Vec<_> = sequence_cases(&mut rng, -256i64..=256, 0..=256).collect();
        for mut values in cases {
            let mut seq: SqrtVec<_> = values.iter().copied().collect();
            let mut folded: SqrtFoldVec<ConcatenateOperation<i64>> =
                values.iter().map(|&v| vec![v]).collect();
            for _ in 0..512 {
                let i = rng.random(0..=values.len());
                let x = rng.random(-256..=256);
                match rng.random(0..5) {
                    0 => {
                        seq.insert(i, x);
                        folded.insert(i, vec![x]);
                        values.insert(i, x);
                    }
                    1 if i < values.len() => {
                        let old = values.remove(i);
                        assert_eq!(seq.remove(i), old);
                        assert_eq!(folded.remove(i), [old]);
                    }
                    2 => {
                        seq.push(x);
                        folded.push(vec![x]);
                        values.push(x);
                    }
                    3 => {
                        let old = values.pop();
                        assert_eq!(seq.pop(), old);
                        assert_eq!(folded.pop(), old.map(|v| vec![v]));
                    }
                    _ => {
                        assert_eq!(seq.get(i), values.get(i));
                        assert_eq!(folded.get(i).map(|v| &v[0]), values.get(i));
                        if let Some(v) = seq.get_mut(i) {
                            *v = x;
                        }
                        if let Some(v) = values.get_mut(i) {
                            *v = x;
                            folded.modify(i, |_| vec![x]);
                        }
                    }
                }
                assert_eq!(seq.len(), values.len());
                assert!(seq.iter().eq(values.iter()));
                let l = i.min(values.len());
                let r = rng.random(l..=values.len());
                assert_eq!(folded.fold(l..r), values[l..r]);
                check_buckets(&seq.inner);
            }
            assert!(folded.iter().map(|v| &v[0]).eq(values.iter()));
            for x in seq.iter_mut() {
                *x += 1;
            }
            for x in &mut values {
                *x += 1;
            }
            assert_eq!(seq, values.iter().copied().collect());
            let mut iter = seq.into_iter();
            let mut expected = VecDeque::from(values);
            while !expected.is_empty() {
                assert_eq!(iter.len(), expected.len());
                if rng.random(0..2) == 0 {
                    assert_eq!(iter.next(), expected.pop_front());
                } else {
                    assert_eq!(iter.next_back(), expected.pop_back());
                }
            }
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }
        // Retain sparse buckets while transferring elements to the tail at constant length.
        let mut seq: SqrtVec<_> = (0..256).collect();
        let mut expected: Vec<_> = (0..256).collect();
        for x in 0..4096 {
            let mut index = 0;
            for bucket in &seq.inner.buckets {
                if bucket.values.len() > 1 {
                    break;
                }
                index += bucket.values.len();
            }
            assert_eq!(seq.remove(index), expected.remove(index));
            seq.push(x);
            expected.push(x);
            assert!(seq.iter().eq(expected.iter()));
            check_buckets(&seq.inner);
        }
        while let Some(x) = expected.pop() {
            assert_eq!(seq.pop(), Some(x));
            check_buckets(&seq.inner);
        }
        struct Value;
        let mut seq = SqrtVec::new();
        seq.push(Value);
        seq.insert(0, Value);
        seq.remove(1);
        assert!(seq.get_mut(0).is_some());
        assert_eq!(seq.iter().clone().count(), 1);
    }
}
