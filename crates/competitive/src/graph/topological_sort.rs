use super::{DirectedGraph, VertexMap};

pub trait TopologicalSortExt: DirectedGraph {
    fn topological_sort(&self) -> Vec<Self::Vertex>
    where
        Self: VertexMap<usize>,
    {
        let mut indegree = self.construct_vmap(|| 0);
        for u in self.vertices() {
            for neighbor in self.neighbors(u) {
                *self.vmap_get_mut(&mut indegree, neighbor.to) += 1;
            }
        }
        let mut stack = self
            .vertices()
            .filter(|&u| *self.vmap_get(&indegree, u) == 0)
            .collect::<Vec<_>>();
        let mut order = Vec::with_capacity(self.vsize());
        while let Some(u) = stack.pop() {
            order.push(u);
            for neighbor in self.neighbors(u) {
                let indegree = self.vmap_get_mut(&mut indegree, neighbor.to);
                *indegree -= 1;
                if *indegree == 0 {
                    stack.push(neighbor.to);
                }
            }
        }
        order
    }
}

impl<G> TopologicalSortExt for G where G: DirectedGraph + ?Sized {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::{DirectedSparseGraph, UsizeGraph},
        rand,
        tools::Xorshift,
    };

    #[test]
    fn test_topological_sort() {
        const Q: usize = 1_000;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);
            let mut reachable = vec![vec![false; n]; n];
            for &(u, v) in &edges {
                reachable[u][v] = true;
            }
            for k in 0..n {
                for i in 0..n {
                    for j in 0..n {
                        if reachable[i][k] && reachable[k][j] {
                            reachable[i][j] = true;
                        }
                    }
                }
            }
            let acyclic = (0..n).all(|u| !reachable[u][u]);
            let sparse = DirectedSparseGraph::from_edges(n, edges.clone());
            let closure = UsizeGraph::new(n, |u| {
                edges
                    .iter()
                    .filter(move |&&(from, _)| from == u)
                    .map(|&(_, to)| (to, ()))
            });

            for order in [sparse.topological_sort(), closure.topological_sort()] {
                let mut position = vec![None; n];
                for (i, vertex) in order.into_iter().enumerate() {
                    assert!(position[vertex].replace(i).is_none());
                }
                for &(u, v) in &edges {
                    if let Some(v) = position[v] {
                        assert!(position[u].is_some_and(|u| u < v));
                    }
                }
                assert_eq!(position.iter().all(Option::is_some), acyclic);
            }
        }
    }
}
