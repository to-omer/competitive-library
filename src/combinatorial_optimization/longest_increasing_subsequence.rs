#[cargo_snippet::snippet("LongestIncreasingSubsequence")]
pub struct LongestIncreasingSubsequence<T: Ord> {
    pub dp: Vec<T>,
}
#[cargo_snippet::snippet("LongestIncreasingSubsequence")]
impl<T: Ord> LongestIncreasingSubsequence<T> {
    pub fn new() -> Self {
        Self { dp: Vec::new() }
    }
    pub fn len(&self) -> usize {
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
