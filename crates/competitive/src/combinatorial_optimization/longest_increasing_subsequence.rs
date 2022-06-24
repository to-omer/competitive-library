#[derive(Debug, Clone)]
pub struct LongestIncreasingSubsequence<T> {
    pub dp: Vec<T>,
}

impl<T> Default for LongestIncreasingSubsequence<T> {
    fn default() -> Self {
        Self { dp: Vec::new() }
    }
}

impl<T> LongestIncreasingSubsequence<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn longest_length(&self) -> usize {
        self.dp.len()
    }
}

impl<T> LongestIncreasingSubsequence<T>
where
    T: Ord,
{
    pub fn insert(&mut self, x: T) {
        let i = self.dp.binary_search(&x).unwrap_or_else(|x| x);
        if let Some(t) = self.dp.get_mut(i) {
            *t = x;
        } else {
            self.dp.push(x);
        }
    }
}

impl<T> Extend<T> for LongestIncreasingSubsequence<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for x in iter.into_iter() {
            self.insert(x);
        }
    }
}
