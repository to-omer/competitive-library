use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap, btree_map, hash_map},
    fmt::{self, Debug},
    hash::Hash,
    iter::{Extend, FromIterator},
    ops::RangeBounds,
};
#[cfg(verus_keep_ghost)]
use vstd::{
    laws_cmp::obeys_cmp,
    prelude::*,
    std_specs::{btree::key_obeys_cmp_spec, iter::IteratorSpec},
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

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
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
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T> Default for BTreeCounter<T>
where
    T: Ord,
{
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result@ == Map::<T, usize>::empty()))]
    fn default() -> Self {
        Self {
            map: BTreeMap::default(),
        }
    }
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T> BTreeCounter<T> {
    #[cfg_attr(verus_keep_ghost, verus_spec(result => requires key_obeys_cmp_spec::<T>(), ensures result == self@.len()))]
    pub fn len(&self) -> usize {
        self.map.len()
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == self@.is_empty()))]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires key_obeys_cmp_spec::<T>(),
        ensures result.remaining().unref().to_set() == self@.dom(),
            result.remaining().no_duplicates(), result.remaining().len() == self@.len(),
            result.decrease() is Some,
            vstd::std_specs::btree::increasing_seq(result.remaining()),
    ))]
    pub fn keys(&self) -> btree_map::Keys<'_, T, usize> {
        self.map.keys()
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires key_obeys_cmp_spec::<T>(),
        ensures result.remaining().unref().to_set() == self@.values(),
            result.remaining().len() == self@.len(), result.decrease() is Some,
            exists|keys: Seq<T>| vstd::std_specs::btree::increasing_seq(keys)
                && keys.to_set() == self@.dom() && keys.no_duplicates()
                && result.remaining() == keys.map(|i: int, k| &self@[k]),
    ))]
    pub fn values(&self) -> btree_map::Values<'_, T, usize> {
        self.map.values()
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires key_obeys_cmp_spec::<T>(),
        ensures result.remaining().unref().to_set() == self@.kv_pairs(),
            result.remaining().no_duplicates(), result.remaining().len() == self@.len(),
            result.decrease() is Some,
            vstd::std_specs::btree::increasing_seq(result.remaining().map_values(|kv: (&T, &usize)| *kv.0)),
    ))]
    pub fn iter(&self) -> btree_map::Iter<'_, T, usize> {
        self.map.iter()
    }
    #[cfg_attr(verus_keep_ghost, verifier::external)]
    pub fn range<Q, R>(&self, range: R) -> btree_map::Range<'_, T, usize>
    where
        Q: Ord,
        R: RangeBounds<Q>,
        T: Borrow<Q> + Ord,
    {
        self.map.range(range)
    }
}
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T> BTreeCounter<T>
where
    T: Ord,
{
    #[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result@ == Map::<T, usize>::empty()))]
    pub fn new() -> Self {
        Self::default()
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(ensures final(self)@ == Map::<T, usize>::empty()))]
    pub fn clear(&mut self) {
        self.map.clear();
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires obeys_cmp::<T>(), ensures result == self.count(*item),
    ))]
    pub fn get(&self, item: &T) -> usize {
        self.map.get(item).cloned().unwrap_or_default()
    }
    #[cfg_attr(verus_keep_ghost, verifier::external)]
    pub fn add(&mut self, item: T) {
        self.add_count(item, 1)
    }
    #[cfg_attr(verus_keep_ghost, verifier::external)]
    pub fn add_count(&mut self, item: T, count: usize) {
        *self.map.entry(item).or_default() += count;
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires obeys_cmp::<T>(),
        ensures result == (old(self).count(*item) > 0),
            final(self)@ == old(self).after_removal(*item, 1),
    ))]
    pub fn remove(&mut self, item: &T) -> bool {
        self.remove_count(item, 1) == 1
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires obeys_cmp::<T>(),
        ensures result == if old(self).count(*item) <= count { old(self).count(*item) } else { count as nat },
            final(self)@ == old(self).after_removal(*item, count as nat),
    ))]
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

#[cfg(verus_keep_ghost)]
verus! {

    impl<T> View for BTreeCounter<T> {
        type V = Map<T, usize>;
        closed spec fn view(&self) -> Map<T, usize> { self.map@ }
    }

    impl<T> BTreeCounter<T> {
        pub open spec fn count(&self, item: T) -> nat {
            if self@.contains_key(item) { self@[item] as nat } else { 0 }
        }

        pub open spec fn after_removal(&self, item: T, count: nat) -> Map<T, usize> {
            if self.count(item) <= count { self@.remove(item) }
            else { self@.insert(item, (self.count(item) - count) as usize) }
        }
    }
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
        fn removal_conserves_counts(counter: &mut BTreeCounter<u64>, key: u64, first: usize, second: usize) {
            let ghost before = *counter;
            let initial = counter.get(&key);
            let a = counter.remove_count(&key, first);
            let b = counter.remove_count(&key, second);
            let remaining = counter.get(&key);
            proof {
                #[verifier::proof_note("BTreeCounter::remove_count saturates at the stored count, returns exactly the removed multiplicity, removes an exhausted key including a stored zero, and preserves all other keys; any two removals conserve initial = removed + remaining without an overflow precondition. The standard lawful key-order model is required. Insertion, range, Clone, Extend, and FromIterator are not verified")]
                assert(initial as nat == a as nat + b as nat + remaining as nat);
                assert(forall|other: u64| other != key ==> #[trigger] counter@.get(other) == before@.get(other));
            }
        }

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
            let x = rng.random(..N);
            let c = rng.random(..N);
            assert_eq!(cnt.get(&x), arr[x]);
            match rng.random(0..4) {
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
            let x = rng.random(..N);
            let c = rng.random(..N);
            assert_eq!(cnt.get(&x), arr[x]);
            match rng.random(0..4) {
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
