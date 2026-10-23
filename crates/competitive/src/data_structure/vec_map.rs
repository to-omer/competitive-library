use super::{Container, ContainerEntry};
use std::{
    fmt::{self, Debug},
    iter::{FilterMap, Flatten, Map, repeat_with},
    mem::replace,
    slice,
};

/// Stores keys at distinct indices supplied by `key_to_index`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecMap<const FIXED: bool, K, V, F> {
    pub data: Vec<Option<(K, V)>>,
    key_to_index: F,
}

impl<const FIXED: bool, K, V, F> Debug for VecMap<FIXED, K, V, F>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.data.iter().flatten().map(|(k, v): &(K, V)| (k, v)))
            .finish()
    }
}

#[derive(Debug)]
pub struct Entry<'a, K, V> {
    key: K,
    entry: &'a mut Option<(K, V)>,
}

impl<K, V, F> VecMap<false, K, V, F> {
    pub fn new(key_to_index: F) -> Self
    where
        F: Fn(&K) -> usize,
    {
        Self::with_capacity(0, key_to_index)
    }

    pub fn with_capacity(capacity: usize, key_to_index: F) -> Self
    where
        F: Fn(&K) -> usize,
    {
        Self {
            data: Vec::with_capacity(capacity),
            key_to_index,
        }
    }
}

impl<K, V, F> VecMap<true, K, V, F> {
    /// Fixed storage: every key's index must be in `0..len`.
    pub fn with_len(len: usize, key_to_index: F) -> Self
    where
        F: Fn(&K) -> usize,
    {
        Self {
            data: repeat_with(|| None).take(len).collect(),
            key_to_index,
        }
    }
}

impl<const FIXED: bool, K, V, F> Container for VecMap<FIXED, K, V, F>
where
    F: Fn(&K) -> usize,
{
    type Key = K;
    type Value = V;
    type Entry<'a>
        = Entry<'a, K, V>
    where
        Self: 'a,
        Self::Key: 'a,
        Self::Value: 'a;
    type Iter<'a>
        = Map<Flatten<slice::Iter<'a, Option<(K, V)>>>, fn(&(K, V)) -> (&K, &V)>
    where
        Self: 'a,
        Self::Key: 'a,
        Self::Value: 'a;
    type Drain<'a>
        = FilterMap<slice::IterMut<'a, Option<(K, V)>>, fn(&mut Option<(K, V)>) -> Option<(K, V)>>
    where
        Self: 'a,
        Self::Key: 'a,
        Self::Value: 'a;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        let index = (self.key_to_index)(key);
        self.data.get(index).and_then(|x| x.as_ref()).map(|x| &x.1)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        let index = (self.key_to_index)(key);
        self.data
            .get_mut(index)
            .and_then(|x| x.as_mut())
            .map(|x| &mut x.1)
    }

    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        let index = (self.key_to_index)(&key);
        let entry = if FIXED {
            self.data.get_mut(index).unwrap()
        } else {
            if index >= self.data.len() {
                self.data.resize_with(index + 1, Default::default);
            }
            unsafe { self.data.get_unchecked_mut(index) }
        };
        match entry {
            Some((_, v)) => Some(replace(v, value)),
            entry => {
                *entry = Some((key, value));
                None
            }
        }
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        let index = (self.key_to_index)(key);
        self.data.get_mut(index).and_then(|x| x.take()).map(|x| x.1)
    }

    fn entry(&mut self, key: Self::Key) -> Self::Entry<'_> {
        let index = (self.key_to_index)(&key);
        let entry = if FIXED {
            self.data.get_mut(index).unwrap()
        } else {
            if index >= self.data.len() {
                self.data.resize_with(index + 1, Default::default);
            }
            unsafe { self.data.get_unchecked_mut(index) }
        };
        Entry { key, entry }
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.data.iter().flatten().map(|(k, v)| (k, v))
    }

    fn drain(&mut self) -> Self::Drain<'_> {
        self.data.iter_mut().filter_map(Option::take)
    }
}

impl<'a, K, V> ContainerEntry<'a> for Entry<'a, K, V> {
    type Key = K;
    type Value = V;

    fn or_default(self) -> &'a mut Self::Value
    where
        Self::Value: Default,
    {
        match self.entry {
            Some((_, value)) => value,
            entry => {
                *entry = Some((self.key, Default::default()));
                unsafe { &mut entry.as_mut().unwrap_unchecked().1 }
            }
        }
    }

    fn or_insert(self, default: Self::Value) -> &'a mut Self::Value {
        match self.entry {
            Some((_, value)) => value,
            entry => {
                *entry = Some((self.key, default));
                unsafe { &mut entry.as_mut().unwrap_unchecked().1 }
            }
        }
    }

    fn or_insert_with<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce() -> Self::Value,
    {
        match self.entry {
            Some((_, value)) => value,
            entry => {
                *entry = Some((self.key, default()));
                unsafe { &mut entry.as_mut().unwrap_unchecked().1 }
            }
        }
    }

    fn or_insert_with_key<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce(&Self::Key) -> Self::Value,
    {
        match self.entry {
            Some((_, value)) => value,
            entry => {
                let default = default(&self.key);
                *entry = Some((self.key, default));
                unsafe { &mut entry.as_mut().unwrap_unchecked().1 }
            }
        }
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Self::Value),
    {
        if let Some((_, value)) = self.entry {
            f(value);
        }
        self
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_fixed_operations() {
        let initial: [Option<u64>; 4] = kani::any();
        let mut map = VecMap::with_len(4, |key: &u8| *key as usize);
        for (key, value) in initial.iter().enumerate() {
            if let Some(value) = *value {
                map.insert(key as u8, value);
            }
        }
        for (key, value) in initial.iter().enumerate() {
            kani::assert(
                map.get(&(key as u8)).copied() == *value,
                "Constructing any map on keys 0 through 3 preserves its occupied and vacant slots",
            );
        }
        let mut expected = initial;
        let (op, key, value): (u8, u8, u64) = kani::any();
        kani::assume(op < 4 && key < 4);
        let slot = &mut expected[key as usize];
        match op {
            0 => {
                kani::assert(
                    map.insert(key, value) == *slot,
                    "Insertion returns exactly the previous value",
                );
                *slot = Some(value);
            }
            1 => {
                kani::assert(
                    map.remove(&key) == *slot,
                    "Removal returns exactly the previous value",
                );
                *slot = None;
            }
            2 => {
                *map.entry(key).or_insert(value) ^= value;
                *slot = Some(slot.unwrap_or(value) ^ value);
            }
            _ => {
                let mut called = false;
                map.entry(key)
                    .and_modify(|x| *x ^= value)
                    .or_insert_with_key(|&k| {
                        called = true;
                        value ^ k as u64
                    });
                kani::assert(
                    called == slot.is_none(),
                    "The entry initializer runs exactly once for a vacant key and never for an occupied key",
                );
                *slot = Some(match *slot {
                    Some(old) => old ^ value,
                    None => value ^ key as u64,
                });
            }
        }
        for (key, value) in expected.iter().enumerate() {
            kani::assert(
                map.get(&(key as u8)).copied() == *value,
                "With fixed storage, every insert, remove, entry insertion, or entry modification from any map on keys 0 through 3 preserves all u64 values",
            );
        }
        let mut drained = [None; 4];
        for (key, value) in map.drain() {
            kani::assert(
                drained[key as usize].is_none(),
                "Drain yields each occupied key once",
            );
            drained[key as usize] = Some(value);
        }
        for key in 0..4 {
            kani::assert(
                drained[key] == expected[key] && map.get(&(key as u8)).is_none(),
                "Complete draining yields exactly the logical map and leaves every key vacant",
            );
        }
        kani::cover!(op == 3 && initial[key as usize].is_none());
        kani::cover!(op == 3 && initial[key as usize].is_some());
        kani::cover!(key == 3 && initial.iter().all(Option::is_none));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BTreeMap;

    #[test]
    fn test_vec_map() {
        let mut rng = Xorshift::default();
        for capacity in 0..100 {
            let mut map = VecMap::with_capacity(capacity, |&x: &usize| x);
            assert!(map.data.capacity() >= capacity);
            assert!(map.data.is_empty());
            let mut expected = BTreeMap::new();
            assert!(map.iter().eq(expected.iter()));
            for _ in 0..1000 {
                let key = rng.random(0..=100);
                let value = rng.random(-100..=100);
                match rng.random(0..5) {
                    0 => {
                        assert_eq!(map.insert(key, value), expected.insert(key, value));
                    }
                    1 => {
                        assert_eq!(map.remove(&key), expected.remove(&key));
                    }
                    2 => {
                        if let Some(x) = map.get_mut(&key) {
                            *x += value;
                        }
                        if let Some(x) = expected.get_mut(&key) {
                            *x += value;
                        }
                    }
                    3 => {
                        assert_eq!(
                            map.entry(key).or_insert(value),
                            expected.entry(key).or_insert(value)
                        );
                    }
                    _ => {
                        assert_eq!(
                            map.entry(key).and_modify(|x| *x += value).or_default(),
                            expected.entry(key).and_modify(|x| *x += value).or_default()
                        );
                    }
                }
                for key in 0..=100 {
                    assert_eq!(map.get(&key), expected.get(&key));
                }
                assert!(map.iter().eq(expected.iter()));
                assert!(map.iter().rev().eq(expected.iter().rev()));
            }
        }
    }
}
