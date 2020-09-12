use super::AdjacencyGraphAbstraction;

#[cargo_snippet::snippet("topological_sort")]
pub trait AdjacencyGraphTopologicalSortExt<'a>: AdjacencyGraphAbstraction<'a> {
    fn topological_sort(&'a self) -> Vec<usize> {
        let mut indeg = vec![0; self.vertices_size()];
        let mut res = vec![];
        for a in self.vertices().flat_map(|u| self.adjacencies(u)) {
            indeg[a.to] += 1;
        }
        let mut stack = self
            .vertices()
            .filter(|&u| indeg[u] == 0)
            .collect::<Vec<_>>();
        while let Some(u) = stack.pop() {
            res.push(u);
            for a in self.adjacencies(u) {
                indeg[a.to] -= 1;
                if indeg[a.to] == 0 {
                    stack.push(a.to);
                }
            }
        }
        res
    }
}
#[cargo_snippet::snippet("topological_sort")]
impl<'a, G: AdjacencyGraphAbstraction<'a>> AdjacencyGraphTopologicalSortExt<'a> for G {}
