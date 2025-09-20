use super::{EdgeListGraph, UnionFind};

impl EdgeListGraph {
    pub fn minimum_spanning_tree<T>(&self, weight: impl Fn(&usize) -> T) -> Vec<bool>
    where
        T: Ord,
    {
        let mut idx: Vec<_> = (0..self.edges_size()).collect();
        idx.sort_by_key(weight);
        let mut uf = UnionFind::new(self.vertices_size());
        let mut res = vec![false; self.edges_size()];
        for eid in idx.into_iter() {
            let (u, v) = self[eid];
            res[eid] = uf.unite(u, v);
        }
        res
    }
}
