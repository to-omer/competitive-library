use std::{
    borrow::Borrow,
    collections::{btree_map, hash_map, BTreeMap, HashMap},
    fmt::{self, Debug},
    hash::Hash,
    iter::{Extend, FromIterator},
    ops::RangeBounds,
};

#[derive(Clone)]
pub struct HashCounter<T> {
    map: HashMap<T, usize>,
}
impl<T> Debug for HashCounter<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
impl<T> Default for HashCounter<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}
impl<T> HashCounter<T> {
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn clear(&mut self) {
        self.map.clear();
    }
    pub fn keys(&self) -> hash_map::Keys<'_, T, usize> {
        self.map.keys()
    }
    pub fn values(&self) -> hash_map::Values<'_, T, usize> {
        self.map.values()
    }
    pub fn iter(&self) -> hash_map::Iter<'_, T, usize> {
        self.map.iter()
    }
    pub fn drain(&mut self) -> hash_map::Drain<'_, T, usize> {
        self.map.drain()
    }
}
impl<T> HashCounter<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }
    pub fn get(&self, item: &T) -> usize {
        self.map.get(item).cloned().unwrap_or_default()
    }
    pub fn add(&mut self, item: T) {
        self.add_count(item, 1)
    }
    pub fn add_count(&mut self, item: T, count: usize) {
        *self.map.entry(item).or_default() += count;
    }
    pub fn remove(&mut self, item: &T) -> bool {
        self.remove_count(item, 1) == 1
    }
    pub fn remove_count(&mut self, item: &T, count: usize) -> usize {
        if let Some(cnt) = self.map.get_mut(item) {
            if *cnt <= count {
                let cnt = *cnt;
                self.map.remove(item);
                cnt
            } else {
                *cnt -= count;
                count
            }
        } else {
            0
        }
    }
    pub fn append(&mut self, other: &mut Self) {
        if self.map.len() < other.map.len() {
            std::mem::swap(self, other);
        }
        self.extend(other.map.drain());
    }
}
impl<T> Extend<T> for HashCounter<T>
where
    T: Eq + Hash,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.add(item)
        }
    }
}
impl<T> Extend<(T, usize)> for HashCounter<T>
where
    T: Eq + Hash,
{
    fn extend<I: IntoIterator<Item = (T, usize)>>(&mut self, iter: I) {
        for (item, count) in iter {
            self.add_count(item, count)
        }
    }
}
impl<T> FromIterator<T> for HashCounter<T>
where
    T: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}
impl<T> FromIterator<(T, usize)> for HashCounter<T>
where
    T: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

#[derive(Clone)]
pub struct BTreeCounter<T> {
    map: BTreeMap<T, usize>,
}
impl<T> Debug for BTreeCounter<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
impl<T> Default for BTreeCounter<T>
where
    T: Ord,
{
    fn default() -> Self {
        Self {
            map: BTreeMap::default(),
        }
    }
}
impl<T> BTreeCounter<T> {
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn keys(&self) -> btree_map::Keys<'_, T, usize> {
        self.map.keys()
    }
    pub fn values(&self) -> btree_map::Values<'_, T, usize> {
        self.map.values()
    }
    pub fn iter(&self) -> btree_map::Iter<'_, T, usize> {
        self.map.iter()
    }
    pub fn range<Q, R>(&self, range: R) -> btree_map::Range<'_, T, usize>
    where
        Q: Ord,
        R: RangeBounds<Q>,
        T: Borrow<Q> + Ord,
    {
        self.map.range(range)
    }
}
impl<T> BTreeCounter<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        self.map.clear();
    }
    pub fn get(&self, item: &T) -> usize {
        self.map.get(item).cloned().unwrap_or_default()
    }
    pub fn add(&mut self, item: T) {
        self.add_count(item, 1)
    }
    pub fn add_count(&mut self, item: T, count: usize) {
        *self.map.entry(item).or_default() += count;
    }
    pub fn remove(&mut self, item: &T) -> bool {
        self.remove_count(item, 1) == 1
    }
    pub fn remove_count(&mut self, item: &T, count: usize) -> usize {
        if let Some(cnt) = self.map.get_mut(item) {
            if *cnt <= count {
                let cnt = *cnt;
                self.map.remove(item);
                cnt
            } else {
                *cnt -= count;
                count
            }
        } else {
            0
        }
    }
}
impl<T> Extend<T> for BTreeCounter<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.add(item)
        }
    }
}
impl<T> Extend<(T, usize)> for BTreeCounter<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = (T, usize)>>(&mut self, iter: I) {
        for (item, count) in iter {
            self.add_count(item, count)
        }
    }
}
impl<T> FromIterator<T> for BTreeCounter<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}
impl<T> FromIterator<(T, usize)> for BTreeCounter<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_hash_counter() {
        let mut rng = Xorshift::default();
        const N: usize = 100_000;
        const Q: usize = 100_000;
        let mut cnt = HashCounter::<usize>::new();
        let mut arr = vec![0usize; N];
        for _ in 0..Q {
            let x = rng.gen(..N);
            let c = rng.gen(..N);
            assert_eq!(cnt.get(&x), arr[x]);
            match rng.gen(0..4) {
                0 => {
                    cnt.add(x);
                    arr[x] += 1;
                }
                1 => {
                    assert_eq!(cnt.remove(&x), arr[x] > 0);
                    if arr[x] > 0 {
                        arr[x] -= 1;
                    }
                }
                2 => {
                    cnt.add_count(x, c);
                    arr[x] += c;
                }
                _ => {
                    assert_eq!(cnt.remove_count(&x, c), arr[x].min(c));
                    arr[x] -= arr[x].min(c);
                }
            }
            assert_eq!(cnt.get(&x), arr[x]);
        }
    }

    #[test]
    fn test_btree_counter() {
        let mut rng = Xorshift::default();
        const N: usize = 100_000;
        const Q: usize = 100_000;
        let mut cnt = BTreeCounter::<usize>::new();
        let mut arr = vec![0usize; N];
        for _ in 0..Q {
            let x = rng.gen(..N);
            let c = rng.gen(..N);
            assert_eq!(cnt.get(&x), arr[x]);
            match rng.gen(0..4) {
                0 => {
                    cnt.add(x);
                    arr[x] += 1;
                }
                1 => {
                    assert_eq!(cnt.remove(&x), arr[x] > 0);
                    if arr[x] > 0 {
                        arr[x] -= 1;
                    }
                }
                2 => {
                    cnt.add_count(x, c);
                    arr[x] += c;
                }
                _ => {
                    assert_eq!(cnt.remove_count(&x, c), arr[x].min(c));
                    arr[x] -= arr[x].min(c);
                }
            }
            assert_eq!(cnt.get(&x), arr[x]);
        }
    }
}
