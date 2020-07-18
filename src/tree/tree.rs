use crate::graph::Graph;

#[cargo_snippet::snippet("TreeRec")]
#[derive(Debug, Clone)]
pub struct TreeRec {
    pub n: usize,
}
#[cargo_snippet::snippet("TreeRec")]
impl TreeRec {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
    pub fn dfs(&mut self, u: usize, p: usize, graph: &Graph) {
        for a in graph.adjacency(u).iter().filter(|a| a.to != p) {
            self.dfs(a.to, u, graph);
        }
    }
}
