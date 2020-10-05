#[derive(Clone, Debug)]
pub struct KnuthMorrisPratt<T: Eq> {
    pattern: Vec<T>,
    table: Vec<usize>,
}
impl<T: Eq> KnuthMorrisPratt<T> {
    pub fn new(pattern: Vec<T>) -> Self {
        let mut table = vec![0; pattern.len() + 1];
        for i in 1..pattern.len() {
            let mut j = table[i - 1];
            while j > 0 && pattern[i] != pattern[j] {
                j = table[j - 1];
            }
            table[i] = j + (pattern[i] == pattern[j]) as usize;
        }
        Self { pattern, table }
    }
    pub fn search_all(&self, s: &[T]) -> Vec<usize> {
        let mut res = vec![];
        let mut j = 0;
        for (i, s) in s.iter().enumerate() {
            while j > 0 && s != &self.pattern[j] {
                j = self.table[j - 1];
            }
            if s == &self.pattern[j] {
                j += 1;
            }
            if j == self.pattern.len() {
                res.push(i - (j - 1));
                j = self.table[j - 1];
            }
        }
        res
    }
}
