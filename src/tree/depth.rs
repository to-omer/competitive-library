use crate::graph::Graph;
use cargo_snippet::snippet;

#[snippet("tree_depth")]
impl Graph {
    fn depth_dfs(&self, u: usize, p: usize, d: u64, depth: &mut Vec<u64>) {
        depth[u] = d;
        for a in self.adjacency(u) {
            if a.to != p {
                self.depth_dfs(a.to, u, d + 1, depth);
            }
        }
    }
    pub fn tree_depth(&self, root: usize) -> Vec<u64> {
        let n = self.vsize;
        let mut depth = vec![0; n];
        self.depth_dfs(root, n, 0, &mut depth);
        depth
    }
}

#[snippet("tree_size")]
impl Graph {
    fn size_dfs(&self, u: usize, p: usize, size: &mut Vec<u64>) {
        size[u] = 1;
        for a in self.adjacency(u) {
            if a.to != p {
                self.size_dfs(a.to, u, size);
                size[u] += size[a.to];
            }
        }
    }
    pub fn tree_size(&self, root: usize) -> Vec<u64> {
        let n = self.vsize;
        let mut size = vec![0; n];
        self.size_dfs(root, n, &mut size);
        size
    }
}
