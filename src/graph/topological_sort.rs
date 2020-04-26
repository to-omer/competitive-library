use super::*;
use cargo_snippet::snippet;

#[snippet("topological_sort")]
impl Graph {
    pub fn topological_sort(&self) -> Vec<usize> {
        let mut indeg = vec![0; self.vsize];
        let mut res = vec![];
        for a in self.vertices().flat_map(|u| self.adjacency(u)) {
            indeg[a.to] += 1;
        }
        let mut stack = self
            .vertices()
            .filter(|&u| indeg[u] == 0)
            .collect::<Vec<_>>();
        while let Some(u) = stack.pop() {
            res.push(u);
            for a in self.adjacency(u) {
                indeg[a.to] -= 1;
                if indeg[a.to] == 0 {
                    stack.push(a.to);
                }
            }
        }
        res
    }
}
