#[cargo_snippet::snippet("KnuthMorrisPratt")]
#[derive(Clone, Debug)]
pub struct KnuthMorrisPratt<T: Eq> {
    pattern: Vec<T>,
    table: Vec<usize>,
}
#[cargo_snippet::snippet("KnuthMorrisPratt")]
impl<T: Eq> KnuthMorrisPratt<T> {
    pub fn new(pattern: Vec<T>) -> KnuthMorrisPratt<T> {
        let mut table = vec![0; pattern.len() + 1];
        for i in 1..pattern.len() {
            let mut j = table[i - 1];
            while j > 0 && pattern[i] != pattern[j] {
                j = table[j - 1];
            }
            table[i] = j + (pattern[i] == pattern[j]) as usize;
        }
        KnuthMorrisPratt {
            pattern: pattern,
            table: table,
        }
    }
    pub fn search_all(&self, s: &Vec<T>) -> Vec<usize> {
        let mut res = vec![];
        let mut j = 0;
        for i in 0..s.len() {
            while j > 0 && s[i] != self.pattern[j] {
                j = self.table[j - 1];
            }
            if s[i] == self.pattern[j] {
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
