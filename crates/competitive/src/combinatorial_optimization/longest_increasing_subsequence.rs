#[snippet::entry("LongestIncreasingSubsequence")]
#[derive(Debug, Clone)]
pub struct LongestIncreasingSubsequence<T: Ord> {
    pub dp: Vec<T>,
}
#[snippet::entry("LongestIncreasingSubsequence")]
impl<T: Ord> Default for LongestIncreasingSubsequence<T> {
    fn default() -> Self {
        Self { dp: Vec::new() }
    }
}
#[snippet::entry("LongestIncreasingSubsequence")]
impl<T: Ord> LongestIncreasingSubsequence<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn longest_length(&self) -> usize {
        self.dp.len()
    }
    pub fn insert(&mut self, x: T) {
        let i = self.dp.binary_search(&x).unwrap_or_else(|x| x);
        if let Some(t) = self.dp.get_mut(i) {
            *t = x;
        } else {
            self.dp.push(x);
        }
    }
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for x in iter.into_iter() {
            self.insert(x);
        }
    }
}
