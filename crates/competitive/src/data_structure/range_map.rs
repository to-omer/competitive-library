use std::{
    collections::{BTreeMap, btree_map},
    iter::{Extend, FromIterator},
};

/// A map to control intervals that have same values.
#[derive(Debug, Clone)]
pub struct RangeMap<K, V> {
    map: BTreeMap<(K, K), V>,
}
impl<K, V> Default for RangeMap<K, V>
where
    K: Ord,
{
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}
impl<K, V> RangeMap<K, V> {
    /// Makes a new, empty `RangeMap`.
    pub fn new() -> Self
    where
        K: Ord,
    {
        Default::default()
    }
    /// Clears the map, removing all elements.
    pub fn clear(&mut self)
    where
        K: Ord,
    {
        self.map.clear();
    }
    /// Returns true if the map contains a value for the key.
    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Clone + Ord,
    {
        self.get(key).is_some()
    }
    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Clone + Ord,
    {
        self.get_range_value(key).map(|(_, v)| v)
    }
    /// Returns the range-value pair corresponding to the key.
    pub fn get_range_value(&self, key: &K) -> Option<(&(K, K), &V)>
    where
        K: Clone + Ord,
    {
        self.get_right_if(key, |r, _| key == &r.0)
            .or_else(|| self.get_left_if(key, |r, _| key < &r.1))
    }
    /// Inserts values into the specified range.
    pub fn insert(&mut self, range: (K, K), value: V)
    where
        K: Clone + Ord,
        V: Clone + Eq,
    {
        self.insert_with(range, value, |_, _| {});
    }
    /// Insert values and operate old range-value pairs.
    pub fn insert_with<F>(&mut self, range: (K, K), value: V, mut f: F)
    where
        K: Clone + Ord,
        V: Clone + Eq,
        F: FnMut((K, K), V),
    {
        if range.0 >= range.1 {
            return;
        }
        let mut ins_range = range.clone();
        if let Some((r, v)) = self.pop_left_if(&range.0, |r, v| {
            range.0 < r.1 || range.0 == r.1 && &value == v
        }) {
            if range.1 < r.1 {
                if value == v {
                    ins_range = r;
                } else {
                    self.map.insert((r.0, range.0.clone()), v.clone());
                    self.map.insert((range.1.clone(), r.1), v.clone());
                }
                f(range.clone(), v);
            } else {
                if value == v {
                    ins_range.0 = r.0;
                } else {
                    self.map.insert((r.0, range.0.clone()), v.clone());
                }
                if range.0 < r.1 {
                    f((range.0.clone(), r.1), v);
                }
            }
        }
        let mut wait = None;
        if let Some((r, _)) = self.pop_right_if(&range.1, |r, v| range.1 == r.0 && &value == v) {
            ins_range.1 = r.1;
        } else if let Some((r, v)) = self.pop_left_if(&range.1, |r, _| range.1 < r.1) {
            if value == v {
                ins_range.1 = r.1;
            } else {
                self.map.insert((range.1.clone(), r.1), v.clone());
            }
            wait = Some(((r.0, range.1.clone()), v));
        }
        let mut f = self.drain_with_inner(range, f);
        if let Some((r, v)) = wait {
            f(r, v);
        }
        self.map.insert(ins_range, value);
    }
    /// Remove values contained in the range.
    pub fn remove(&mut self, range: (K, K))
    where
        K: Clone + Ord,
        V: Clone,
    {
        self.drain_with(range, |_, _| {});
    }
    /// Get a left neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn get_left_if<F>(&self, key: &K, mut pred: F) -> Option<(&(K, K), &V)>
    where
        K: Clone + Ord,
        F: FnMut(&(K, K), &V) -> bool,
    {
        self.map
            .range(..(key.clone(), key.clone()))
            .next_back()
            .filter(|(r, v)| pred(r, v))
    }
    /// Get a right neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn get_right_if<F>(&self, key: &K, mut pred: F) -> Option<(&(K, K), &V)>
    where
        K: Clone + Ord,
        F: FnMut(&(K, K), &V) -> bool,
    {
        self.map
            .range((key.clone(), key.clone())..)
            .next()
            .filter(|(r, v)| pred(r, v))
    }
    /// Pop a left neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn pop_left_if<F>(&mut self, key: &K, pred: F) -> Option<((K, K), V)>
    where
        K: Clone + Ord,
        F: FnMut(&(K, K), &V) -> bool,
    {
        match self.get_left_if(key, pred) {
            Some((r, _)) => {
                let r = r.clone();
                let v = self.map.remove(&r).unwrap();
                Some((r, v))
            }
            None => None,
        }
    }
    /// Pop a right neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn pop_right_if<F>(&mut self, key: &K, pred: F) -> Option<((K, K), V)>
    where
        K: Clone + Ord,
        F: FnMut(&(K, K), &V) -> bool,
    {
        match self.get_right_if(key, pred) {
            Some((r, _)) => {
                let r = r.clone();
                let v = self.map.remove(&r).unwrap();
                Some((r, v))
            }
            None => None,
        }
    }
    /// Operate and consume range-value pairs in range when no overlapping.
    fn drain_with_inner<F>(&mut self, range: (K, K), mut f: F) -> F
    where
        K: Clone + Ord,
        F: FnMut((K, K), V),
    {
        while let Some((r, _)) = self
            .map
            .range((range.0.clone(), range.0.clone())..(range.1.clone(), range.1.clone()))
            .next()
        {
            let r = r.clone();
            let v = self.map.remove(&r).unwrap();
            f(r, v);
        }
        f
    }
    /// Operate and consume range-value pairs in range.
    pub fn drain_with<F>(&mut self, range: (K, K), mut f: F)
    where
        K: Clone + Ord,
        V: Clone,
        F: FnMut((K, K), V),
    {
        if range.0 >= range.1 {
            return;
        }
        if let Some((r, v)) = self.pop_left_if(&range.0, |r, _| range.0 < r.1) {
            if range.1 < r.1 {
                f(range.clone(), v.clone());
                self.map.insert((range.1.clone(), r.1), v.clone());
            } else {
                f((range.0.clone(), r.1), v.clone());
            }
            self.map.insert((r.0, range.0.clone()), v);
        }
        let mut wait = None;
        if let Some((r, v)) = self.pop_left_if(&range.1, |r, _| range.1 < r.1) {
            wait = Some(((r.0, range.1.clone()), v.clone()));
            self.map.insert((range.1.clone(), r.1), v);
        }
        let mut f = self.drain_with_inner(range, f);
        if let Some((r, v)) = wait {
            f(r, v);
        }
    }
    pub fn iter(&self) -> btree_map::Iter<'_, (K, K), V> {
        self.map.iter()
    }
    pub fn iter_mut(&mut self) -> btree_map::IterMut<'_, (K, K), V> {
        self.map.iter_mut()
    }
    pub fn keys(&self) -> btree_map::Keys<'_, (K, K), V> {
        self.map.keys()
    }
    pub fn values(&self) -> btree_map::Values<'_, (K, K), V> {
        self.map.values()
    }
    pub fn values_mut(&mut self) -> btree_map::ValuesMut<'_, (K, K), V> {
        self.map.values_mut()
    }
}
impl<K, V> Extend<((K, K), V)> for RangeMap<K, V>
where
    K: Clone + Ord,
    V: Clone + Eq,
{
    fn extend<T: IntoIterator<Item = ((K, K), V)>>(&mut self, iter: T) {
        for (range, value) in iter {
            self.insert(range, value);
        }
    }
}
impl<K, V> FromIterator<((K, K), V)> for RangeMap<K, V>
where
    K: Clone + Ord,
    V: Clone + Eq,
{
    fn from_iter<T: IntoIterator<Item = ((K, K), V)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

/// A set to control intervals.
#[derive(Debug, Clone)]
pub struct RangeSet<T> {
    map: RangeMap<T, ()>,
}
impl<T> Default for RangeSet<T>
where
    T: Ord,
{
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}
impl<T> RangeSet<T> {
    /// Makes a new, empty `RangeSet`.
    pub fn new() -> Self
    where
        T: Ord,
    {
        Default::default()
    }
    /// Clears the set, removing all elements.
    pub fn clear(&mut self)
    where
        T: Ord,
    {
        self.map.clear();
    }
    /// Returns true if the set contains a key.
    pub fn contains(&self, key: &T) -> bool
    where
        T: Clone + Ord,
    {
        self.get_range(key).is_some()
    }
    /// Returns the range corresponding to the key.
    pub fn get_range(&self, key: &T) -> Option<&(T, T)>
    where
        T: Clone + Ord,
    {
        self.map.get_range_value(key).map(|(r, _)| r)
    }
    /// Inserts into the specified range.
    pub fn insert(&mut self, range: (T, T))
    where
        T: Clone + Ord,
    {
        self.insert_with(range, |_| {});
    }
    /// Insert and operate old range.
    pub fn insert_with<F>(&mut self, range: (T, T), mut f: F)
    where
        T: Clone + Ord,
        F: FnMut((T, T)),
    {
        self.map.insert_with(range, (), |r, _| f(r))
    }
    /// Remove items contained in the range.
    pub fn remove(&mut self, range: (T, T))
    where
        T: Clone + Ord,
    {
        self.drain_with(range, |_| {});
    }
    /// Get a left neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn get_left_if<F>(&self, key: &T, mut pred: F) -> Option<&(T, T)>
    where
        T: Clone + Ord,
        F: FnMut(&(T, T)) -> bool,
    {
        self.map.get_left_if(key, |r, _| pred(r)).map(|(r, _)| r)
    }
    /// Get a right neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn get_right_if<F>(&self, key: &T, mut pred: F) -> Option<&(T, T)>
    where
        T: Clone + Ord,
        F: FnMut(&(T, T)) -> bool,
    {
        self.map.get_right_if(key, |r, _| pred(r)).map(|(r, _)| r)
    }
    /// Pop a left neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn pop_left_if<F>(&mut self, key: &T, mut pred: F) -> Option<(T, T)>
    where
        T: Clone + Ord,
        F: FnMut(&(T, T)) -> bool,
    {
        self.map.pop_left_if(key, |r, _| pred(r)).map(|(r, _)| r)
    }
    /// Pop a right neighboring range of `[key, key)` if the predicate is satisfied.
    pub fn pop_right_if<F>(&mut self, key: &T, mut pred: F) -> Option<(T, T)>
    where
        T: Clone + Ord,
        F: FnMut(&(T, T)) -> bool,
    {
        self.map.pop_right_if(key, |r, _| pred(r)).map(|(r, _)| r)
    }
    /// Operate and consume in range.
    pub fn drain_with<F>(&mut self, range: (T, T), mut f: F)
    where
        T: Clone + Ord,
        F: FnMut((T, T)),
    {
        self.map.drain_with(range, |r, _| f(r));
    }
    pub fn iter(&self) -> btree_map::Keys<'_, (T, T), ()> {
        self.map.keys()
    }
}
impl<K> Extend<(K, K)> for RangeSet<K>
where
    K: Clone + Ord,
{
    fn extend<T: IntoIterator<Item = (K, K)>>(&mut self, iter: T) {
        for range in iter {
            self.insert(range);
        }
    }
}
impl<K> FromIterator<(K, K)> for RangeSet<K>
where
    K: Clone + Ord,
{
    fn from_iter<T: IntoIterator<Item = (K, K)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        WithEmptySegment, Xorshift,
        testutil::{exhaustive_sequences, sample_usize},
    };

    fn model_ranges<T: Copy + Eq>(values: &[Option<T>]) -> Vec<((usize, usize), T)> {
        let mut start = 0;
        let mut ranges = Vec::new();
        for run in values.chunk_by(|a, b| a == b) {
            let end = start + run.len();
            if let Some(value) = run[0] {
                ranges.push(((start, end), value));
            }
            start = end;
        }
        ranges
    }

    fn check_map_operation(
        map: &mut RangeMap<usize, i64>,
        model: &mut [Option<i64>],
        range: (usize, usize),
        value: Option<i64>,
    ) {
        let expected: Vec<_> = model_ranges(&model[range.0..range.1])
            .into_iter()
            .map(|((l, r), value)| ((l + range.0, r + range.0), value))
            .collect();
        let mut notified = Vec::new();
        let mut plain = map.clone();
        if let Some(value) = value {
            map.insert_with(range, value, |r, v| notified.push((r, v)));
            plain.insert(range, value);
        } else {
            map.drain_with(range, |r, v| notified.push((r, v)));
            plain.remove(range);
        }
        notified.sort_unstable();
        assert_eq!(
            notified, expected,
            "range={range:?}, value={value:?}, before={model:?}"
        );
        model[range.0..range.1].fill(value);
        let expected = model_ranges(model);
        assert_eq!(
            map.iter().map(|(&r, &v)| (r, v)).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            plain.iter().map(|(&r, &v)| (r, v)).collect::<Vec<_>>(),
            expected
        );
        for key in 0..=model.len() {
            let interval = expected.iter().find(|((l, r), _)| *l <= key && key < *r);
            assert_eq!(
                map.get_range_value(&key),
                interval.map(|(r, v)| (r, v)),
                "key={key}, model={model:?}"
            );
            assert_eq!(map.get(&key).copied(), model.get(key).copied().flatten());
            assert_eq!(map.contains_key(&key), interval.is_some());
        }
    }

    #[test]
    fn test_range_map() {
        // Every state over {absent, 0, 1} and every interval operation through five cells.
        for n in 0..=5 {
            for model in exhaustive_sequences([None, Some(0), Some(1)], n..=n) {
                let mut base = RangeMap::new();
                for (i, value) in model.iter().enumerate() {
                    if let Some(value) = value {
                        base.insert((i, i + 1), *value);
                    }
                }
                for l in 0..=n {
                    for r in l..=n {
                        for value in [None, Some(0), Some(1)] {
                            check_map_operation(
                                &mut base.clone(),
                                &mut model.clone(),
                                (l, r),
                                value,
                            );
                        }
                    }
                }
            }
        }
        let mut rng = Xorshift::default();
        for n in sample_usize(&mut rng, 16, 0..=200, 30) {
            let mut map = RangeMap::new();
            let mut model = vec![None; n];
            for _ in 0..1000 {
                let range = rng.random(WithEmptySegment(n));
                let value = (rng.random(0..4) != 0).then(|| rng.random(-100..=100));
                check_map_operation(&mut map, &mut model, range, value);
            }
        }
    }

    fn check_set_operation(
        set: &mut RangeSet<usize>,
        model: &mut [Option<()>],
        range: (usize, usize),
        insert: bool,
    ) {
        let expected: Vec<_> = model_ranges(&model[range.0..range.1])
            .into_iter()
            .map(|((l, r), ())| (l + range.0, r + range.0))
            .collect();
        let mut notified = Vec::new();
        let mut plain = set.clone();
        if insert {
            set.insert_with(range, |r| notified.push(r));
            plain.insert(range);
        } else {
            set.drain_with(range, |r| notified.push(r));
            plain.remove(range);
        }
        notified.sort_unstable();
        assert_eq!(
            notified, expected,
            "range={range:?}, insert={insert}, before={model:?}"
        );
        model[range.0..range.1].fill(insert.then_some(()));
        let expected: Vec<_> = model_ranges(model).into_iter().map(|(r, ())| r).collect();
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), expected);
        assert_eq!(plain.iter().copied().collect::<Vec<_>>(), expected);
        for key in 0..=model.len() {
            let interval = expected.iter().find(|&&(l, r)| l <= key && key < r);
            assert_eq!(set.get_range(&key), interval, "key={key}, model={model:?}");
            assert_eq!(set.contains(&key), interval.is_some());
        }
    }

    #[test]
    fn test_range_set() {
        for n in 0..=8 {
            for model in exhaustive_sequences([None, Some(())], n..=n) {
                let mut base = RangeSet::new();
                for (i, value) in model.iter().enumerate() {
                    if value.is_some() {
                        base.insert((i, i + 1));
                    }
                }
                for l in 0..=n {
                    for r in l..=n {
                        for insert in [false, true] {
                            check_set_operation(
                                &mut base.clone(),
                                &mut model.clone(),
                                (l, r),
                                insert,
                            );
                        }
                    }
                }
            }
        }
        let mut rng = Xorshift::default();
        for n in sample_usize(&mut rng, 16, 0..=200, 30) {
            let mut set = RangeSet::new();
            let mut model = vec![None; n];
            for _ in 0..1000 {
                let range = rng.random(WithEmptySegment(n));
                let insert = rng.random(0..4) != 0;
                check_set_operation(&mut set, &mut model, range, insert);
            }
        }
    }
}
