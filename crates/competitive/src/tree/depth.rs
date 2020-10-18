use crate::algebra::Monoid;
use crate::graph::UndirectedSparseGraph;

#[codesnip::entry("tree_depth")]
impl UndirectedSparseGraph {
    fn depth_dfs(&self, u: usize, p: usize, d: u64, depth: &mut Vec<u64>) {
        depth[u] = d;
        for a in self.adjacencies(u).filter(|a| a.to != p) {
            self.depth_dfs(a.to, u, d + 1, depth);
        }
    }
    pub fn tree_depth(&self, root: usize) -> Vec<u64> {
        let mut depth = vec![0; self.vertices_size()];
        self.depth_dfs(root, self.vertices_size(), 0, &mut depth);
        depth
    }
}

#[codesnip::entry("tree_depth")]
impl UndirectedSparseGraph {
    fn weighted_depth_dfs<M: Monoid, F: Fn(usize) -> M::T>(
        &self,
        u: usize,
        p: usize,
        d: M::T,
        depth: &mut Vec<M::T>,
        weight: &F,
        monoid: &M,
    ) {
        for a in self.adjacencies(u).filter(|a| a.to != p) {
            let nd = monoid.operate(&d, &weight(a.id));
            self.weighted_depth_dfs(a.to, u, nd, depth, weight, monoid);
        }
        depth[u] = d;
    }
    pub fn weighted_tree_depth<M: Monoid, F: Fn(usize) -> M::T>(
        &self,
        root: usize,
        weight: F,
        monoid: M,
    ) -> Vec<M::T> {
        let mut depth = vec![monoid.unit(); self.vertices_size()];
        self.weighted_depth_dfs(
            root,
            std::usize::MAX,
            monoid.unit(),
            &mut depth,
            &weight,
            &monoid,
        );
        depth
    }
}

#[codesnip::entry("tree_size")]
impl UndirectedSparseGraph {
    fn size_dfs(&self, u: usize, p: usize, size: &mut Vec<u64>) {
        size[u] = 1;
        for a in self.adjacencies(u).filter(|a| a.to != p) {
            self.size_dfs(a.to, u, size);
            size[u] += size[a.to];
        }
    }
    pub fn tree_size(&self, root: usize) -> Vec<u64> {
        let mut size = vec![0; self.vertices_size()];
        self.size_dfs(root, std::usize::MAX, &mut size);
        size
    }
}
