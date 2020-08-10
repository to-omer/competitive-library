#[cargo_snippet::snippet("Trie")]
pub struct Trie {
    child: Vec<Vec<usize>>,
    accept: Vec<usize>,
    char_size: usize,
}
#[cargo_snippet::snippet("Trie")]
impl Trie {
    pub fn new(char_size: usize) -> Self {
        Self {
            child: vec![vec![0; char_size]],
            accept: vec![0],
            char_size,
        }
    }
    pub fn insert_at(
        &mut self,
        mut node: usize,
        iter: impl IntoIterator<Item = usize>,
    ) -> Vec<usize> {
        let mut path = Vec::new();
        for ch in iter.into_iter() {
            path.push(node);
            if self.child[node][ch] == 0 {
                self.child[node][ch] = self.child.len();
                self.child.push(vec![0; self.char_size]);
                self.accept.push(0);
            }
            node = self.child[node][ch];
        }
        path.push(node);
        self.accept[node] += 1;
        path
    }
    pub fn insert(&mut self, iter: impl IntoIterator<Item = usize>) -> Vec<usize> {
        self.insert_at(0, iter)
    }
    pub fn find_at(
        &self,
        mut node: usize,
        iter: impl IntoIterator<Item = usize>,
    ) -> Result<usize, usize> {
        for ch in iter.into_iter() {
            if let Some(v) = self.child.get(node) {
                node = v[ch];
            } else {
                return Err(node);
            }
        }
        Ok(node)
    }
    pub fn find(&self, iter: impl IntoIterator<Item = usize>) -> Result<usize, usize> {
        self.find_at(0, iter)
    }
    pub fn next_node(&self, node: usize, ch: usize) -> Option<usize> {
        if self.child[node][ch] == 0 {
            None
        } else {
            Some(self.child[node][ch])
        }
    }
    pub fn count(&self, node: usize) -> usize {
        self.accept[node]
    }
    pub fn next_count(&self, node: usize, ch: usize) -> usize {
        if let Some(node) = self.next_node(node, ch) {
            self.count(node)
        } else {
            0
        }
    }
}
