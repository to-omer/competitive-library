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
    pub fn get(&self, key: &T) -> usize {
        self.map.get(key).cloned().unwrap_or_default()
    }
    #[inline]
    pub fn add(&mut self, key: T) {
        *self.map.entry(key).or_default() += 1usize;
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
impl<T: Eq + std::hash::Hash> std::iter::FromIterator<T> for Counter<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Self::default();
        for key in iter {
            map.add(key);
        }
        map
    }
}
