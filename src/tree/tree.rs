use crate::graph::{AdjacencyGraphAbstraction, UndirectedSparseGraph};

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
    pub fn dfs(&mut self, u: usize, p: usize, graph: &UndirectedSparseGraph) {
        for a in graph.adjacencies(u).filter(|a| a.to != p) {
            self.dfs(a.to, u, graph);
        }
    }
}
