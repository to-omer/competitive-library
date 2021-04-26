#[derive(Clone, Debug)]
pub struct HashCounter<T> {
    map: std::collections::HashMap<T, usize>,
}
impl<T> Default for HashCounter<T> {
    fn default() -> Self {
        Self {
            map: std::collections::HashMap::default(),
        }
    }
}
impl<T> HashCounter<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: std::collections::HashMap::with_capacity(capacity),
        }
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn clear(&mut self) {
        self.map.clear();
    }
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, T, usize> {
        self.map.keys()
    }
    pub fn values(&self) -> std::collections::hash_map::Values<'_, T, usize> {
        self.map.values()
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, T, usize> {
        self.map.iter()
    }
    pub fn drain(&mut self) -> std::collections::hash_map::Drain<'_, T, usize> {
        self.map.drain()
    }
}
impl<T> HashCounter<T>
where
    T: Eq + std::hash::Hash,
{
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
impl<T> std::iter::Extend<T> for HashCounter<T>
where
    T: Eq + std::hash::Hash,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.add(item)
        }
    }
}
impl<T> std::iter::Extend<(T, usize)> for HashCounter<T>
where
    T: Eq + std::hash::Hash,
{
    fn extend<I: IntoIterator<Item = (T, usize)>>(&mut self, iter: I) {
        for (item, count) in iter {
            self.add_count(item, count)
        }
    }
}
impl<T> std::iter::FromIterator<T> for HashCounter<T>
where
    T: Eq + std::hash::Hash,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}
impl<T> std::iter::FromIterator<(T, usize)> for HashCounter<T>
where
    T: Eq + std::hash::Hash,
{
    fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

#[derive(Clone, Debug)]
pub struct BTreeCounter<T> {
    map: std::collections::BTreeMap<T, usize>,
}
impl<T> Default for BTreeCounter<T>
where
    T: Ord,
{
    fn default() -> Self {
        Self {
            map: std::collections::BTreeMap::default(),
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
    pub fn keys(&self) -> std::collections::btree_map::Keys<'_, T, usize> {
        self.map.keys()
    }
    pub fn values(&self) -> std::collections::btree_map::Values<'_, T, usize> {
        self.map.values()
    }
    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, T, usize> {
        self.map.iter()
    }
    pub fn range<Q, R>(&self, range: R) -> std::collections::btree_map::Range<'_, T, usize>
    where
        Q: Ord,
        R: std::ops::RangeBounds<Q>,
        T: std::borrow::Borrow<Q> + Ord,
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
impl<T> std::iter::Extend<T> for BTreeCounter<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.add(item)
        }
    }
}
impl<T> std::iter::Extend<(T, usize)> for BTreeCounter<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = (T, usize)>>(&mut self, iter: I) {
        for (item, count) in iter {
            self.add_count(item, count)
        }
    }
}
impl<T> std::iter::FromIterator<T> for BTreeCounter<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}
impl<T> std::iter::FromIterator<(T, usize)> for BTreeCounter<T>
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
