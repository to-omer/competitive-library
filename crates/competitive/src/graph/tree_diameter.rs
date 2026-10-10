use super::{EdgeListGraph, XorLinkedRootedTree, Zero};

impl EdgeListGraph {
    /// `(diameter, vertex_path)` for a nonempty undirected tree; `weight` maps edge IDs to nonnegative values.
    pub fn tree_diameter<T>(&self, weight: impl Fn(&usize) -> T) -> (T, Vec<usize>)
    where
        T: Clone + Ord + Zero + std::ops::Add<Output = T>,
    {
        let n = self.vertices_size();
        let mut parent = vec![!0; n];
        let mut farthest: Vec<_> = (0..n).map(|u| (T::zero(), u)).collect();
        let (mut diameter, mut left, mut right, mut center) = (T::zero(), 0, 0, 0);
        XorLinkedRootedTree::builder(n)
            .with_eindexed()
            .run(0, self.edges().copied(), |u, p, e| {
                parent[u] = p;
                let distance = farthest[u].0.clone() + weight(&e);
                let candidate = distance.clone() + farthest[p].0.clone();
                if diameter < candidate {
                    diameter = candidate;
                    left = farthest[u].1;
                    right = farthest[p].1;
                    center = p;
                }
                if farthest[p].0 < distance {
                    farthest[p] = (distance, farthest[u].1);
                }
            });
        let mut path = Vec::new();
        while left != center {
            path.push(left);
            left = parent[left];
        }
        path.push(center);
        let middle = path.len();
        while right != center {
            path.push(right);
            right = parent[right];
        }
        path[middle..].reverse();
        (diameter, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::testutil::exhaustive_sequences;

    #[test]
    fn test_tree_diameter() {
        for n in 1..=5usize {
            for prufer in exhaustive_sequences(0..n, n.saturating_sub(2)..=n.saturating_sub(2)) {
                let mut degree = vec![1; n];
                for &u in &prufer {
                    degree[u] += 1;
                }
                let mut edges = Vec::new();
                for u in prufer {
                    let leaf = (0..n).find(|&v| degree[v] == 1).unwrap();
                    edges.push((leaf, u));
                    degree[leaf] -= 1;
                    degree[u] -= 1;
                }
                if n > 1 {
                    let last: Vec<_> = (0..n).filter(|&u| degree[u] == 1).collect();
                    edges.push((last[0], last[1]));
                }
                let graph = EdgeListGraph::from_edges(n, edges);
                let edges = graph.edges().as_slice();
                for weights in exhaustive_sequences(0..3u64, edges.len()..=edges.len())
                    .chain([vec![u64::MAX / edges.len().max(1) as u64; edges.len()]])
                {
                    let mut distance = vec![vec![u64::MAX; n]; n];
                    for (u, row) in distance.iter_mut().enumerate() {
                        row[u] = 0;
                    }
                    for (e, &(u, v)) in edges.iter().enumerate() {
                        distance[u][v] = weights[e];
                        distance[v][u] = weights[e];
                    }
                    for k in 0..n {
                        for u in 0..n {
                            for v in 0..n {
                                distance[u][v] = distance[u][v]
                                    .min(distance[u][k].saturating_add(distance[k][v]));
                            }
                        }
                    }
                    let (diameter, path) = graph.tree_diameter(|&e| weights[e]);
                    assert_eq!(diameter, *distance.iter().flatten().max().unwrap());
                    assert!(!path.is_empty());
                    assert_eq!(
                        path.iter()
                            .copied()
                            .collect::<std::collections::HashSet<_>>()
                            .len(),
                        path.len()
                    );
                    let actual: u64 = path
                        .windows(2)
                        .map(|w| {
                            let e = edges
                                .iter()
                                .position(|&(u, v)| {
                                    (u == w[0] && v == w[1]) || (v == w[0] && u == w[1])
                                })
                                .unwrap();
                            weights[e]
                        })
                        .sum();
                    assert_eq!(actual, diameter);
                }
            }
        }
    }
}
