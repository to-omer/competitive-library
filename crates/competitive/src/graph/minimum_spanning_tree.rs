use super::{EdgeListGraph, UnionFind};

impl EdgeListGraph {
    pub fn minimum_spanning_tree<T>(&self, weight: impl Fn(&usize) -> T) -> Vec<bool>
    where
        T: Ord,
    {
        let mut edges: Vec<_> = (0..self.edges_size()).collect();
        edges.sort_unstable_by_key(weight);
        self.minimum_spanning_tree_from_sorted_edges(edges)
    }

    /// Runs Kruskal's algorithm on edge IDs sorted by nondecreasing weight.
    pub fn minimum_spanning_tree_from_sorted_edges(
        &self,
        edges: impl IntoIterator<Item = usize>,
    ) -> Vec<bool> {
        let mut uf = UnionFind::new(self.vertices_size());
        let mut res = vec![false; self.edges_size()];
        let mut selected = 0;
        for eid in edges {
            let (u, v) = self[eid];
            res[eid] = uf.unite(u, v);
            if res[eid] {
                selected += 1;
                if selected + 1 == self.vertices_size() {
                    break;
                }
            }
        }
        res
    }
}
