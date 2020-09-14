#[cargo_snippet::snippet("Counter")]
#[derive(Clone, Debug)]
pub struct Counter<T: Eq + std::hash::Hash> {
    map: std::collections::HashMap<T, usize>,
}
#[cargo_snippet::snippet("Counter")]
impl<T: Eq + std::hash::Hash> Default for Counter<T> {
    #[inline]
    fn default() -> Self {
        Self {
            map: std::collections::HashMap::default(),
        }
    }
}
#[cargo_snippet::snippet("Counter")]
impl<T: Eq + std::hash::Hash> Counter<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn get(&self, item: &T) -> usize {
        self.map.get(item).cloned().unwrap_or_default()
    }
    #[inline]
    pub fn add(&mut self, item: T) {
        *self.map.entry(item).or_default() += 1usize;
    }
    #[inline]
    pub fn remove(&mut self, item: &T) -> bool {
        if let Some(cnt) = self.map.get_mut(item) {
            if *cnt <= 1 {
                let cnt = *cnt;
                self.map.remove(item);
                cnt == 1
            } else {
                *cnt -= 1usize;
                true
            }
        } else {
            false
        }
    }
    #[inline]
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, T, usize> {
        self.map.keys()
    }
    #[inline]
    pub fn values(&self) -> std::collections::hash_map::Values<'_, T, usize> {
        self.map.values()
    }
    #[inline]
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, T, usize> {
        self.map.iter()
    }
}
#[cargo_snippet::snippet("Counter")]
impl<T: Eq + std::hash::Hash> std::iter::Extend<T> for Counter<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.add(item)
        }
    }
}
#[cargo_snippet::snippet("Counter")]
impl<T: Eq + std::hash::Hash> std::iter::FromIterator<T> for Counter<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

#[test]
fn test_counter() {
    let mut counter: Counter<i32> = [0, 1, 1, 1, 2, 2].iter().copied().collect();
    assert_eq!(counter.get(&0), 1);
    assert_eq!(counter.get(&1), 3);
    assert_eq!(counter.get(&2), 2);
    assert_eq!(counter.get(&3), 0);

    assert!(counter.remove(&1));
    assert_eq!(counter.get(&1), 2);

    assert!(counter.remove(&1));
    assert_eq!(counter.get(&1), 1);

    assert!(counter.remove(&1));
    assert_eq!(counter.get(&1), 0);

    assert!(!counter.remove(&1));
    assert_eq!(counter.get(&1), 0);
}
