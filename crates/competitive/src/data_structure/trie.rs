#[derive(Debug, Clone)]
pub struct Trie {
    child: Vec<usize>,
    node_size: usize,
    char_size: usize,
}
impl Trie {
    pub fn new(char_size: usize) -> Self {
        Self {
            child: vec![0; char_size],
            node_size: 1,
            char_size,
        }
    }
    pub fn with_capacity(char_size: usize, capacity: usize) -> Self {
        let mut child = Vec::with_capacity(capacity * char_size);
        child.resize_with(char_size, Default::default);
        Self {
            child,
            node_size: 1,
            char_size,
        }
    }
    pub fn insert_once_at(&mut self, node: usize, ch: usize) -> usize {
        let index = node * self.char_size + ch;
        if self.child[index] == 0 {
            self.child[index] = self.node_size;
            self.child
                .resize_with(self.child.len() + self.char_size, Default::default);
            self.node_size += 1;
        }
        self.child[index]
    }
    pub fn insert_at<I>(&mut self, mut node: usize, iter: I) -> Vec<usize>
    where
        I: IntoIterator<Item = usize>,
    {
        let mut path = Vec::new();
        for ch in iter.into_iter() {
            path.push(node);
            node = self.insert_once_at(node, ch);
        }
        path.push(node);
        path
    }
    pub fn insert<I>(&mut self, iter: I) -> Vec<usize>
    where
        I: IntoIterator<Item = usize>,
    {
        self.insert_at(0, iter)
    }
    pub fn find_at<I>(&self, mut node: usize, iter: I) -> Result<usize, usize>
    where
        I: IntoIterator<Item = usize>,
    {
        for ch in iter.into_iter() {
            if let Some(&nnode) = self.child.get(node * self.char_size + ch) {
                node = nnode;
            } else {
                return Err(node);
            }
        }
        Ok(node)
    }
    pub fn find<I>(&self, iter: I) -> Result<usize, usize>
    where
        I: IntoIterator<Item = usize>,
    {
        self.find_at(0, iter)
    }
    pub fn next_node(&self, node: usize, ch: usize) -> Option<usize> {
        let index = node * self.char_size + ch;
        if self.child[index] == 0 {
            None
        } else {
            Some(self.child[index])
        }
    }
    pub fn node_size(&self) -> usize {
        self.node_size
    }
    pub fn edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::with_capacity(self.node_size - 1);
        let mut stack = vec![0usize];
        while let Some(node) = stack.pop() {
            for ch in (0..self.char_size).rev() {
                if let Some(nnode) = self.next_node(node, ch) {
                    edges.push((node, nnode));
                    stack.push(nnode);
                }
            }
        }
        edges
    }
}
