use crate::graph::Graph;
use cargo_snippet::snippet;

#[snippet("TreeRec")]
#[derive(Debug)]
pub struct TreeRec {
    pub n: usize,
}
#[snippet("TreeRec")]
impl TreeRec {
    pub fn new(n: usize) -> TreeRec {
        TreeRec { n: n }
    }
    pub fn dfs(&mut self, u: usize, p: usize, graph: &Graph) {
        for a in graph.adjacency(u) {
            if a.to != p {
                self.dfs(a.to, u, graph);
            }
        }
    }
}
