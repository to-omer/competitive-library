use crate::graph::SparseGraph;

#[cfg_attr(nightly, codesnip::entry("tree_order", include("SparseGraph")))]
impl<D> SparseGraph<D> {
    /// (order, parents)
    pub fn tree_order(&self, root: usize) -> (Vec<usize>, Vec<usize>) {
        let n = self.vertices_size();
        let mut order = Vec::with_capacity(n);
        let mut parents = vec![!0usize; n];
        let mut stack = Vec::with_capacity(n);
        stack.push(root);
        while let Some(u) = stack.pop() {
            order.push(u);
            for a in self.adjacencies(u).rev() {
                if a.to != parents[u] {
                    parents[a.to] = u;
                    stack.push(a.to);
                }
            }
        }
        (order, parents)
    }
}
