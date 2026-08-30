use super::{EdgeMap, Graph, VertexMap};
use std::collections::VecDeque;

pub trait GraphOrderExt: Graph {
    fn bfs_order(&self, root: Self::Vertex) -> Vec<Self::Vertex>
    where
        Self: VertexMap<bool>,
    {
        let mut visited = self.construct_vmap(|| false);
        let mut order = Vec::with_capacity(self.vsize());
        *self.vmap_get_mut(&mut visited, root) = true;
        let mut queue = VecDeque::from([root]);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for neighbor in self.neighbors(u) {
                if !self.vmap_get(&visited, neighbor.to) {
                    *self.vmap_get_mut(&mut visited, neighbor.to) = true;
                    queue.push_back(neighbor.to);
                }
            }
        }
        order
    }

    fn dfs_order(&self, root: Self::Vertex) -> Vec<Self::Vertex>
    where
        Self: VertexMap<bool>,
    {
        let mut visited = self.construct_vmap(|| false);
        let mut order = Vec::with_capacity(self.vsize());
        *self.vmap_get_mut(&mut visited, root) = true;
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            order.push(u);
            for neighbor in self.neighbors(u) {
                if !self.vmap_get(&visited, neighbor.to) {
                    *self.vmap_get_mut(&mut visited, neighbor.to) = true;
                    stack.push(neighbor.to);
                }
            }
        }
        order
    }

    fn dfs_tree(&self, root: Self::Vertex) -> <Self as EdgeMap<bool>>::Emap
    where
        Self: EdgeMap<bool> + VertexMap<bool>,
    {
        let mut visited = self.construct_vmap(|| false);
        let mut used = self.construct_emap(|| false);
        *self.vmap_get_mut(&mut visited, root) = true;
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            for neighbor in self.neighbors(u) {
                if !self.vmap_get(&visited, neighbor.to) {
                    *self.vmap_get_mut(&mut visited, neighbor.to) = true;
                    self.emap_set(&mut used, neighbor.label, true);
                    stack.push(neighbor.to);
                }
            }
        }
        used
    }

    /// f: |g, root, ord: [vertex, parent]| {}
    fn for_each_connected_components<F>(&self, mut f: F)
    where
        Self: VertexMap<bool>,
        F: FnMut(&Self, Self::Vertex, &[(Self::Vertex, Option<Self::Vertex>)]),
    {
        let mut visited = self.construct_vmap(|| false);
        let mut order = Vec::with_capacity(self.vsize());
        for root in self.vertices() {
            if !self.vmap_get(&visited, root) {
                *self.vmap_get_mut(&mut visited, root) = true;
                order.push((root, None));
                let mut i = 0;
                while i < order.len() {
                    let u = order[i].0;
                    for neighbor in self.neighbors(u) {
                        if !self.vmap_get(&visited, neighbor.to) {
                            *self.vmap_get_mut(&mut visited, neighbor.to) = true;
                            order.push((neighbor.to, Some(u)));
                        }
                    }
                    i += 1;
                }
                f(self, root, &order);
                order.clear();
            }
        }
    }
}

impl<G> GraphOrderExt for G where G: Graph + ?Sized {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::{AdjacencyListGraph, UndirectedSparseGraph, UsizeGraph},
        rand,
        tools::Xorshift,
    };

    fn reachable(n: usize, edges: &[(usize, usize)], root: usize, selected: &[bool]) -> Vec<bool> {
        let mut reached = vec![false; n];
        reached[root] = true;
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            for (eid, &(from, to)) in edges.iter().enumerate() {
                if !selected[eid] {
                    continue;
                }
                let v = if from == u {
                    to
                } else if to == u {
                    from
                } else {
                    continue;
                };
                if !reached[v] {
                    reached[v] = true;
                    stack.push(v);
                }
            }
        }
        reached
    }

    #[test]
    fn test_bfs_order() {
        const Q: usize = 500;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);
            let sparse = UndirectedSparseGraph::from_edges(n, edges.clone());
            let closure = UsizeGraph::new(n, |u| {
                edges.iter().flat_map(move |&(from, to)| {
                    [
                        (from == u).then_some((to, ())),
                        (to == u).then_some((from, ())),
                    ]
                    .into_iter()
                    .flatten()
                })
            });
            let mut adjacency_list = AdjacencyListGraph::new(n);
            for &(u, v) in &edges {
                adjacency_list.add_undirected_edge(u, v);
            }
            let all = vec![true; m];

            for root in 0..n {
                let expected = reachable(n, &edges, root, &all);
                for order in [
                    sparse.bfs_order(root),
                    closure.bfs_order(root),
                    adjacency_list.bfs_order(root),
                ] {
                    let mut visited = vec![false; n];
                    for vertex in order {
                        assert!(!std::mem::replace(&mut visited[vertex], true));
                    }
                    assert_eq!(visited, expected);
                }
            }
        }
    }

    #[test]
    fn test_dfs_order() {
        const Q: usize = 500;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);
            let sparse = UndirectedSparseGraph::from_edges(n, edges.clone());
            let closure = UsizeGraph::new(n, |u| {
                edges.iter().flat_map(move |&(from, to)| {
                    [
                        (from == u).then_some((to, ())),
                        (to == u).then_some((from, ())),
                    ]
                    .into_iter()
                    .flatten()
                })
            });
            let mut adjacency_list = AdjacencyListGraph::new(n);
            for &(u, v) in &edges {
                adjacency_list.add_undirected_edge(u, v);
            }
            let all = vec![true; m];

            for root in 0..n {
                let expected = reachable(n, &edges, root, &all);
                for order in [
                    sparse.dfs_order(root),
                    closure.dfs_order(root),
                    adjacency_list.dfs_order(root),
                ] {
                    let mut visited = vec![false; n];
                    for vertex in order {
                        assert!(!std::mem::replace(&mut visited[vertex], true));
                    }
                    assert_eq!(visited, expected);
                }
            }
        }
    }

    #[test]
    fn test_dfs_tree() {
        const Q: usize = 500;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);
            let sparse = UndirectedSparseGraph::from_edges(n, edges.clone());
            let mut adjacency_list = AdjacencyListGraph::new(n);
            for &(u, v) in &edges {
                adjacency_list.add_undirected_edge(u, v);
            }
            let all = vec![true; m];

            for root in 0..n {
                let expected = reachable(n, &edges, root, &all);
                for selected in [sparse.dfs_tree(root), adjacency_list.dfs_tree(root)] {
                    assert_eq!(selected.len(), m);
                    assert_eq!(
                        selected.iter().filter(|&&selected| selected).count(),
                        expected.iter().filter(|&&reached| reached).count() - 1
                    );
                    assert_eq!(reachable(n, &edges, root, &selected), expected);
                }
            }
        }
    }

    #[test]
    fn test_for_each_connected_components() {
        const Q: usize = 500;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);
            let sparse = UndirectedSparseGraph::from_edges(n, edges.clone());
            let all = vec![true; m];
            let mut expected_components = Vec::new();
            let mut used = vec![false; n];
            for root in 0..n {
                if !used[root] {
                    let component: Vec<_> = reachable(n, &edges, root, &all)
                        .into_iter()
                        .enumerate()
                        .filter_map(|(vertex, reached)| reached.then_some(vertex))
                        .collect();
                    for &vertex in &component {
                        used[vertex] = true;
                    }
                    expected_components.push(component);
                }
            }
            let mut actual_components = Vec::new();
            sparse.for_each_connected_components(|_, root, order| {
                assert_eq!(order[0], (root, None));
                let mut component = Vec::with_capacity(order.len());
                for &(vertex, parent) in order {
                    if let Some(parent) = parent {
                        assert!(edges.iter().any(|&(u, v)| {
                            (u == vertex && v == parent) || (u == parent && v == vertex)
                        }));
                    }
                    component.push(vertex);
                }
                component.sort_unstable();
                actual_components.push(component);
            });
            assert_eq!(actual_components, expected_components);
        }
    }
}
