use super::{Graph, UndirectedSparseGraph};

impl UndirectedSparseGraph {
    /// On failure, returns an odd cycle of vertex IDs without a repeated endpoint.
    pub fn bipartite_coloring(&self) -> Result<Vec<bool>, Vec<usize>> {
        let n = self.vertices_size();
        let mut colors = vec![false; n];
        let mut parents = vec![!0; n];
        let mut queue = Vec::with_capacity(n);
        let mut read = 0;
        for root in self.vertices() {
            if parents[root] != !0 {
                continue;
            }
            parents[root] = root;
            queue.push(root);
            while read < queue.len() {
                let u = queue[read];
                read += 1;
                for neighbor in self.neighbors(u) {
                    let v = neighbor.to;
                    if parents[v] == !0 {
                        parents[v] = u;
                        colors[v] = !colors[u];
                        queue.push(v);
                    } else if colors[u] == colors[v] {
                        // BFS endpoints of a same-color edge have equal depth.
                        let (mut u, mut v) = (u, v);
                        let mut cycle = vec![u];
                        let mut other = vec![v];
                        while u != v {
                            u = parents[u];
                            v = parents[v];
                            cycle.push(u);
                            other.push(v);
                        }
                        other.pop();
                        cycle.extend(other.into_iter().rev());
                        return Err(cycle);
                    }
                }
            }
        }
        Ok(colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;
    use std::collections::BTreeSet;

    fn assert_bipartite(n: usize, edges: &[(usize, usize)]) {
        let expected = (0..1usize << n).any(|mask| {
            edges
                .iter()
                .all(|&(u, v)| ((mask >> u) ^ (mask >> v)) & 1 != 0)
        });
        let graph = UndirectedSparseGraph::from_edges(n, edges.to_vec());
        match graph.bipartite_coloring() {
            Ok(colors) => {
                assert!(expected, "n={n}, edges={edges:?}");
                assert_eq!(colors.len(), n);
                assert!(edges.iter().all(|&(u, v)| colors[u] != colors[v]));
            }
            Err(cycle) => {
                assert!(!expected, "n={n}, edges={edges:?}");
                assert_eq!(cycle.len() & 1, 1);
                assert_eq!(
                    cycle.iter().copied().collect::<BTreeSet<_>>().len(),
                    cycle.len()
                );
                assert!(cycle.iter().all(|&u| u < n));
                for (&u, &v) in cycle.iter().zip(cycle.iter().cycle().skip(1)) {
                    assert!(
                        edges
                            .iter()
                            .any(|&(a, b)| (u, v) == (a, b) || (v, u) == (a, b))
                    );
                }
            }
        }
    }

    #[test]
    fn test_bipartite_coloring() {
        for n in 0..=6 {
            let choices: Vec<_> = (0..n)
                .flat_map(|u| (u + usize::from(n == 6)..n).map(move |v| (u, v)))
                .collect();
            for mask in 0..1usize << choices.len() {
                let edges: Vec<_> = choices
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| mask >> i & 1 != 0)
                    .map(|(_, &e)| e)
                    .collect();
                assert_bipartite(n, &edges);
                let edges: Vec<_> = edges
                    .iter()
                    .rev()
                    .flat_map(|&(u, v)| [(v, u), (v, u)])
                    .collect();
                assert_bipartite(n, &edges);
            }
        }
        let mut rng = Xorshift::default();
        for n in 1..=12 {
            for _ in 0..1000 {
                let m = rng.random(0..=2 * n * n);
                let edges: Vec<_> = (0..m)
                    .map(|_| (rng.random(0..n), rng.random(0..n)))
                    .collect();
                assert_bipartite(n, &edges);
            }
        }
    }
}
