use super::{DirectedSparseGraph, StronglyConnectedComponent};

#[derive(Debug, Clone)]
pub struct TwoSatisfiability {
    vsize: usize,
    edges: Vec<(usize, usize)>,
}

impl TwoSatisfiability {
    pub fn new(vsize: usize) -> Self {
        Self {
            vsize,
            edges: Vec::new(),
        }
    }
    pub fn add_inner(&mut self, u: usize, v: usize) {
        self.edges.push((u, v));
        self.edges.push((v ^ 1, u ^ 1));
    }
    pub fn add_or(&mut self, x: usize, y: usize) {
        self.add_inner(x * 2 + 1, y * 2);
    }
    pub fn add_nand(&mut self, x: usize, y: usize) {
        self.add_inner(x * 2, y * 2 + 1);
    }
    pub fn set_true(&mut self, x: usize) {
        self.add_inner(x * 2 + 1, x * 2);
    }
    pub fn set_false(&mut self, x: usize) {
        self.add_inner(x * 2, x * 2 + 1);
    }
    pub fn two_satisfiability(self) -> Option<Vec<bool>> {
        let graph = DirectedSparseGraph::from_edges(self.vsize * 2, self.edges);
        let scc = StronglyConnectedComponent::new(&graph);
        let mut res = vec![false; self.vsize];
        for i in 0..self.vsize {
            if scc[i * 2] == scc[i * 2 + 1] {
                return None;
            }
            res[i] = scc[i * 2] > scc[i * 2 + 1];
        }
        Some(res)
    }
}
