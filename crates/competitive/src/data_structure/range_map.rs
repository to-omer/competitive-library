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
    use crate::tools::{NotEmptySegment as Nes, Xorshift};

    #[test]
    fn test_insert() {
        let mut map: RangeMap<usize, usize> = Default::default();
        map.insert((1, 3), 0);
        assert_eq!(map.get_range_value(&0), None);
        assert_eq!(map.get_range_value(&1), Some((&(1, 3), &0)));
        assert_eq!(map.get_range_value(&2), Some((&(1, 3), &0)));
        assert_eq!(map.get_range_value(&3), None);

        map.insert_with((2, 4), 1, |r, v| assert_eq!((r, v), ((2, 3), 0)));
        assert_eq!(map.get_range_value(&0), None);
        assert_eq!(map.get_range_value(&1), Some((&(1, 2), &0)));
        assert_eq!(map.get_range_value(&2), Some((&(2, 4), &1)));
        assert_eq!(map.get_range_value(&3), Some((&(2, 4), &1)));
        assert_eq!(map.get_range_value(&4), None);

        map.insert_with((2, 3), 2, |r, v| assert_eq!((r, v), ((2, 3), 1)));
        assert_eq!(map.get_range_value(&0), None);
        assert_eq!(map.get_range_value(&1), Some((&(1, 2), &0)));
        assert_eq!(map.get_range_value(&2), Some((&(2, 3), &2)));
        assert_eq!(map.get_range_value(&3), Some((&(3, 4), &1)));
        assert_eq!(map.get_range_value(&4), None);

        map.insert((1, 8), 3);
        map.insert((4, 6), 4);
        assert_eq!(map.get_range_value(&6), Some((&(6, 8), &3)));
    }

    #[test]
    fn test_range_map() {
        let mut rng = Xorshift::default();
        const N: usize = 200;
        const Q: usize = 5000;
        const A: i64 = 100;
        let mut map: RangeMap<usize, i64> = Default::default();
        let mut arr = vec![None; N];
        for _ in 0..Q {
            match rng.random(0..5) {
                0 => {
                    let key = rng.random(..N);
                    if let Some((r, &v)) = map.get_range_value(&key) {
                        arr[r.0..r.1].iter_mut().for_each(|a| {
                            assert_eq!(Some(v), *a);
                        });
                    };
                }
                1 => {
                    let range = rng.random(Nes(N));
                    map.drain_with(range, |r, v| {
                        arr[r.0.max(range.0)..r.1.min(range.1)]
                            .iter_mut()
                            .for_each(|a| {
                                assert_eq!(Some(v), *a);
                                *a = None;
                            });
                    });
                    arr[range.0..range.1]
                        .iter_mut()
                        .for_each(|a| assert_eq!(*a, None));
                }
                _ => {
                    let range = rng.random(Nes(N));
                    let value = rng.random(-A..=A);
                    map.insert_with(range, value, |r, v| {
                        arr[r.0.max(range.0)..r.1.min(range.1)]
                            .iter_mut()
                            .for_each(|a| {
                                assert_eq!(Some(v), *a);
                                *a = None;
                            });
                    });
                    arr[range.0..range.1].iter_mut().for_each(|a| {
                        assert_eq!(*a, None);
                        *a = Some(value);
                    });
                }
            }
            for (key, a) in arr.iter().enumerate() {
                assert_eq!(map.get(&key), a.as_ref());
            }
            for (key, (a, b)) in arr.iter().zip(arr.iter().skip(1)).enumerate() {
                assert_eq!(
                    map.get_range_value(&key) == map.get_range_value(&(key + 1)),
                    a == b
                );
            }
        }
    }

    #[test]
    fn test_range_set() {
        let mut rng = Xorshift::default();
        const N: usize = 200;
        const Q: usize = 5000;
        let mut set: RangeSet<usize> = Default::default();
        let mut arr = [false; N];
        for _ in 0..Q {
            match rng.random(0..5) {
                0 => {
                    let key = rng.random(..N);
                    if let Some(r) = set.get_range(&key) {
                        arr[r.0..r.1].iter_mut().for_each(|a| {
                            assert!(*a);
                        });
                    };
                }
                1 => {
                    let range = rng.random(Nes(N));
                    set.drain_with(range, |r| {
                        arr[r.0.max(range.0)..r.1.min(range.1)]
                            .iter_mut()
                            .for_each(|a| {
                                assert!(*a);
                                *a = false;
                            });
                    });
                    arr[range.0..range.1].iter_mut().for_each(|a| assert!(!*a));
                }
                _ => {
                    let range = rng.random(Nes(N));
                    set.insert_with(range, |r| {
                        arr[r.0.max(range.0)..r.1.min(range.1)]
                            .iter_mut()
                            .for_each(|a| {
                                assert!(*a);
                                *a = false;
                            });
                    });
                    arr[range.0..range.1].iter_mut().for_each(|a| {
                        assert!(!*a);
                        *a = true;
                    });
                }
            }
            for (key, a) in arr.iter().enumerate() {
                assert_eq!(set.contains(&key), *a);
            }
            for (key, (a, b)) in arr.iter().zip(arr.iter().skip(1)).enumerate() {
                assert_eq!(set.get_range(&key) == set.get_range(&(key + 1)), a == b,);
            }
        }
    }
}
