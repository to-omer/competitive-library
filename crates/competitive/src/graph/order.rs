use super::SparseGraph;

#[codesnip::entry("bfs_order", include("SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn bfs_order(&self, root: usize) -> Vec<usize> {
        let mut visited = vec![false; self.vertices_size()];
        let mut ord = Vec::with_capacity(self.vertices_size());
        visited[root] = true;
        let mut deq = std::collections::VecDeque::new();
        deq.push_back(root);
        while let Some(u) = deq.pop_front() {
            ord.push(u);
            for a in self.adjacencies(u).rev() {
                if !visited[a.to] {
                    visited[a.to] = true;
                    deq.push_back(a.to);
                }
            }
        }
        ord
    }
}

#[codesnip::entry("dfs_order", include("SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn dfs_order(&self, root: usize) -> Vec<usize> {
        let mut visited = vec![false; self.vertices_size()];
        let mut ord = Vec::with_capacity(self.vertices_size());
        visited[root] = true;
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            ord.push(u);
            for a in self.adjacencies(u).rev() {
                if !visited[a.to] {
                    visited[a.to] = true;
                    stack.push(a.to);
                }
            }
        }
        ord
    }
}

#[codesnip::entry("dfs_tree", include("SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn dfs_tree(&self, root: usize) -> Vec<bool> {
        let mut visited = vec![false; self.vertices_size()];
        let mut used = vec![false; self.edges_size()];
        visited[root] = true;
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            for a in self.adjacencies(u).rev() {
                if !visited[a.to] {
                    visited[a.to] = true;
                    used[a.id] = true;
                    stack.push(a.to);
                }
            }
        }
        used
    }
}

#[codesnip::entry("for_each_connected_components", include("SparseGraph"))]
impl<D> SparseGraph<D> {
    /// f: |g, root, ord: [vertex, parent]| {}
    pub fn for_each_connected_components<F>(&self, mut f: F)
    where
        F: FnMut(&Self, usize, &[(usize, usize)]),
    {
        let mut visited = vec![false; self.vertices_size()];
        let mut ord = Vec::with_capacity(self.vertices_size());
        for u in self.vertices() {
            if !visited[u] {
                visited[u] = true;
                ord.push((u, !0));
                let mut i = 0;
                while i < ord.len() {
                    let u = ord[i].0;
                    for a in self.adjacencies(u).rev() {
                        if !visited[a.to] {
                            visited[a.to] = true;
                            ord.push((a.to, u));
                        }
                    }
                    i += 1;
                }
                f(self, u, &ord);
                ord.clear();
            }
        }
    }
}
