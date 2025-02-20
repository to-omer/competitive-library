use super::{Container, ContainerEntry, ContainerFactory};
use std::{
    fmt::{self, Debug},
    iter::{FilterMap, Map, repeat_with},
    marker::PhantomData,
    mem::replace,
    slice,
};

#[derive(Debug, Clone)]
pub struct VecMapFactory<K, V, F> {
    key_to_index: F,
    _marker: PhantomData<fn() -> (K, V)>,
}

#[derive(Debug, Clone)]
pub struct FixedVecMapFactory<K, V, F> {
    key_to_index: F,
    len: usize,
    _marker: PhantomData<fn() -> (K, V)>,
}

#[derive(Debug, Clone)]
pub struct VecMapFactoryWithCapacity<K, V, F> {
    key_to_index: F,
    capacity: usize,
    _marker: PhantomData<fn() -> (K, V)>,
}

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

impl<K, V, F> VecMapFactory<K, V, F> {
    pub fn new(key_to_index: F) -> Self {
        Self {
            key_to_index,
            _marker: PhantomData,
        }
    }
}

impl<K, V, F> ContainerFactory for VecMapFactory<K, V, F>
where
    F: Fn(&K) -> usize + Clone,
{
    type Container = VecMap<false, K, V, F>;

    fn create_container(&self) -> Self::Container {
        VecMap {
            data: Vec::new(),
            key_to_index: self.key_to_index.clone(),
        }
    }
}

impl<K, V, F> FixedVecMapFactory<K, V, F> {
    pub fn new(key_to_index: F, len: usize) -> Self {
        Self {
            key_to_index,
            len,
            _marker: PhantomData,
        }
    }
}

impl<K, V, F> ContainerFactory for FixedVecMapFactory<K, V, F>
where
    F: Fn(&K) -> usize + Clone,
{
    type Container = VecMap<true, K, V, F>;

    fn create_container(&self) -> Self::Container {
        VecMap {
            data: repeat_with(|| None).take(self.len).collect(),
            key_to_index: self.key_to_index.clone(),
        }
    }
}

impl<K, V, F> VecMapFactoryWithCapacity<K, V, F> {
    pub fn new(key_to_index: F, capacity: usize) -> Self {
        Self {
            key_to_index,
            capacity,
            _marker: PhantomData,
        }
    }
}

impl<K, V, F> ContainerFactory for VecMapFactoryWithCapacity<K, V, F>
where
    F: Fn(&K) -> usize + Clone,
{
    type Container = VecMap<false, K, V, F>;

    fn create_container(&self) -> Self::Container {
        VecMap {
            data: Vec::with_capacity(self.capacity),
            key_to_index: self.key_to_index.clone(),
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
        = Map<
        FilterMap<slice::Iter<'a, Option<(K, V)>>, fn(&Option<(K, V)>) -> Option<&(K, V)>>,
        fn(&(K, V)) -> (&K, &V),
    >
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
        self.data
            .iter()
            .filter_map::<_, fn(&Option<(K, V)>) -> Option<&(K, V)>>(Option::as_ref)
            .map::<_, fn(&(K, V)) -> (&K, &V)>(|(k, v)| (k, v))
    }

    fn drain(&mut self) -> Self::Drain<'_> {
        self.data
            .iter_mut()
            .filter_map::<_, fn(&mut Option<(K, V)>) -> Option<(K, V)>>(Option::take)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_map_get() {
        let mut map = VecMapFactory::new(|&x: &usize| x).create_container();
        map.insert(0, 0);
        assert_eq!(map.get(&0), Some(&0));
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn test_vec_map_get_mut() {
        let mut map = VecMapFactory::new(|&x: &usize| x).create_container();
        map.insert(0, 0);
        *map.get_mut(&0).unwrap() += 1;
        assert_eq!(map.get(&0), Some(&1));
    }

    #[test]
    fn test_vec_map_insert() {
        let mut map = VecMapFactory::new(|&x: &usize| x).create_container();
        assert_eq!(map.insert(0, 0), None);
        assert_eq!(map.insert(0, 1), Some(0));
        assert_eq!(map.get(&0), Some(&1));
    }

    #[test]
    fn test_vec_map_remove() {
        let mut map = VecMapFactory::new(|&x: &usize| x).create_container();
        map.insert(0, 0);
        assert_eq!(map.remove(&0), Some(0));
        assert_eq!(map.remove(&0), None);
    }

    #[test]
    fn test_vec_map_entry() {
        let mut map = VecMapFactory::new(|&x: &usize| x).create_container();
        map.entry(0).or_insert(0);
        assert_eq!(*map.entry(0).or_insert(1), 0);
        assert_eq!(*map.entry(1).and_modify(|e| *e += 1).or_default(), 0);
        assert_eq!(*map.entry(1).and_modify(|e| *e += 1).or_default(), 1);
    }
}
