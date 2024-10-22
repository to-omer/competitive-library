use std::{
    cmp::Ord,
    collections::{btree_map, hash_map, BTreeMap, HashMap},
    hash::Hash,
    iter::FusedIterator,
    marker::PhantomData,
};

pub trait ContainerFactory {
    type Container: Container;

    fn create_container(&self) -> Self::Container;
}

pub trait Container {
    type Key;
    type Value;
    type Entry<'a>: ContainerEntry<'a, Key = Self::Key, Value = Self::Value>
    where
        Self: 'a,
        Self::Key: 'a,
        Self::Value: 'a;
    type Iter<'a>: Iterator<Item = (&'a Self::Key, &'a Self::Value)>
    where
        Self: 'a,
        Self::Key: 'a,
        Self::Value: 'a;
    type Drain<'a>: Iterator<Item = (Self::Key, Self::Value)>
    where
        Self: 'a,
        Self::Key: 'a,
        Self::Value: 'a;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value>;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;
    fn entry(&mut self, key: Self::Key) -> Self::Entry<'_>;
    fn iter(&self) -> Self::Iter<'_>;
    fn drain(&mut self) -> Self::Drain<'_>;
}

pub trait ContainerEntry<'a> {
    type Key: 'a;
    type Value: 'a;

    fn or_default(self) -> &'a mut Self::Value
    where
        Self::Value: Default;
    fn or_insert(self, default: Self::Value) -> &'a mut Self::Value;
    fn or_insert_with<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce() -> Self::Value;
    fn or_insert_with_key<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce(&Self::Key) -> Self::Value;
    fn key(&self) -> &Self::Key;
    fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Self::Value);
}

impl<F> ContainerFactory for &F
where
    F: ContainerFactory,
{
    type Container = F::Container;

    fn create_container(&self) -> Self::Container {
        F::create_container(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BTreeMapFactory<K, V> {
    _marker: PhantomData<fn() -> (K, V)>,
}

impl<K, V> Default for BTreeMapFactory<K, V> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct BTreeMapDrain<'a, K, V> {
    inner: &'a mut BTreeMap<K, V>,
}

impl<K, V> ContainerFactory for BTreeMapFactory<K, V>
where
    K: Ord,
{
    type Container = BTreeMap<K, V>;

    fn create_container(&self) -> Self::Container {
        BTreeMap::new()
    }
}

impl<K, V> Container for BTreeMap<K, V>
where
    K: Ord,
{
    type Key = K;
    type Value = V;
    type Entry<'a>
        = btree_map::Entry<'a, K, V>
    where
        K: 'a,
        V: 'a;
    type Iter<'a>
        = btree_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a;
    type Drain<'a>
        = BTreeMapDrain<'a, K, V>
    where
        K: 'a,
        V: 'a;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        self.get(key)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        self.get_mut(key)
    }

    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        self.remove(key)
    }

    fn entry(&mut self, key: Self::Key) -> Self::Entry<'_> {
        self.entry(key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn drain(&mut self) -> Self::Drain<'_> {
        BTreeMapDrain { inner: self }
    }
}

impl<'a, K, V> Iterator for BTreeMapDrain<'a, K, V>
where
    K: Ord,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_first()
    }
}

impl<'a, K, V> DoubleEndedIterator for BTreeMapDrain<'a, K, V>
where
    K: Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop_last()
    }
}

impl<'a, K, V> ExactSizeIterator for BTreeMapDrain<'a, K, V>
where
    K: Ord,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K, V> FusedIterator for BTreeMapDrain<'a, K, V> where K: Ord {}

impl<'a, K, V> ContainerEntry<'a> for btree_map::Entry<'a, K, V>
where
    K: 'a + Ord,
    V: 'a,
{
    type Key = K;
    type Value = V;

    fn or_default(self) -> &'a mut Self::Value
    where
        Self::Value: Default,
    {
        self.or_default()
    }

    fn or_insert(self, default: Self::Value) -> &'a mut Self::Value {
        self.or_insert(default)
    }

    fn or_insert_with<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce() -> Self::Value,
    {
        self.or_insert_with(default)
    }

    fn or_insert_with_key<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce(&Self::Key) -> Self::Value,
    {
        self.or_insert_with_key(default)
    }

    fn key(&self) -> &Self::Key {
        self.key()
    }

    fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Self::Value),
    {
        self.and_modify(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashMapFactory<K, V> {
    _marker: PhantomData<fn() -> (K, V)>,
}

impl<K, V> Default for HashMapFactory<K, V> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashMapFactoryWithCapacity<K, V> {
    capacity: usize,
    _marker: PhantomData<fn() -> (K, V)>,
}

impl<K, V> Default for HashMapFactoryWithCapacity<K, V> {
    fn default() -> Self {
        Self {
            capacity: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<K, V> ContainerFactory for HashMapFactory<K, V>
where
    K: Eq + Hash,
{
    type Container = HashMap<K, V>;

    fn create_container(&self) -> Self::Container {
        HashMap::new()
    }
}

impl<K, V> ContainerFactory for HashMapFactoryWithCapacity<K, V>
where
    K: Eq + Hash,
{
    type Container = HashMap<K, V>;

    fn create_container(&self) -> Self::Container {
        HashMap::with_capacity(self.capacity)
    }
}

impl<K, V> Container for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Key = K;
    type Value = V;
    type Entry<'a>
        = hash_map::Entry<'a, K, V>
    where
        K: 'a,
        V: 'a;
    type Iter<'a>
        = hash_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a;
    type Drain<'a>
        = hash_map::Drain<'a, K, V>
    where
        K: 'a,
        V: 'a;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        self.get(key)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        self.get_mut(key)
    }

    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        self.remove(key)
    }

    fn entry(&mut self, key: Self::Key) -> Self::Entry<'_> {
        self.entry(key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn drain(&mut self) -> Self::Drain<'_> {
        self.drain()
    }
}

impl<'a, K, V> ContainerEntry<'a> for hash_map::Entry<'a, K, V>
where
    K: 'a + Eq + Hash,
    V: 'a,
{
    type Key = K;
    type Value = V;

    fn or_default(self) -> &'a mut Self::Value
    where
        Self::Value: Default,
    {
        self.or_default()
    }

    fn or_insert(self, default: Self::Value) -> &'a mut Self::Value {
        self.or_insert(default)
    }

    fn or_insert_with<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce() -> Self::Value,
    {
        self.or_insert_with(default)
    }

    fn or_insert_with_key<F>(self, default: F) -> &'a mut Self::Value
    where
        F: FnOnce(&Self::Key) -> Self::Value,
    {
        self.or_insert_with_key(default)
    }

    fn key(&self) -> &Self::Key {
        self.key()
    }

    fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Self::Value),
    {
        self.and_modify(f)
    }
}
