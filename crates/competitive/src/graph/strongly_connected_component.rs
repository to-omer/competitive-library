use super::DirectedSparseGraph;

/// Strongly connected components in topological order.
#[codesnip::entry("StronglyConnectedComponent")]
#[derive(Debug, Clone)]
pub struct StronglyConnectedComponent {
    vertices: Vec<usize>,
    // Vertices before starts[0] are singleton components removed by Kahn's algorithm.
    starts: Vec<usize>,
}

#[codesnip::entry("StronglyConnectedComponent")]
impl StronglyConnectedComponent {
    /// Decomposes a directed graph on `0..n` in O(V + E) time and space.
    pub fn from_edges(n: usize, edges: impl Iterator<Item = (usize, usize)>) -> Self {
        // Each block stores the previous cursor followed by neighbors in insertion order.
        const BLOCK: usize = 4;
        let mut degree = vec![0usize; n];
        let mut rows = vec![(n, n); n];
        let mut overflow = Vec::new();
        for (u, v) in edges {
            degree[v] += 1;
            let (first, rest) = rows[u];
            let rest = if rest == n {
                first
            } else if rest > n && !(rest - n).is_multiple_of(BLOCK) {
                overflow[rest - n] = first;
                rest + 1
            } else {
                let i = overflow.len();
                overflow.extend_from_slice(&[0; BLOCK]);
                overflow[i] = rest;
                overflow[i + 1] = first;
                n + i + 2
            };
            rows[u] = (v, rest);
        }
        // A cursor below n is the last neighbor, n is the end, and larger
        // values link to earlier edges of the same vertex.
        let next_edge = |cursor: usize| {
            if cursor <= n {
                (cursor, n)
            } else {
                let i = cursor - n - 1;
                let next = if i % BLOCK == 1 {
                    overflow[i - 1]
                } else {
                    cursor - 1
                };
                (overflow[i], next)
            }
        };
        let mut vertices = vec![0; n];
        let mut tail = 0;
        for (i, &d) in degree.iter().enumerate() {
            vertices[tail] = i;
            tail += usize::from(d == 0);
        }
        let mut head = 0;
        while head < tail {
            let u = vertices[head];
            head += 1;
            let (mut v, mut rest) = rows[u];
            while v < n {
                degree[v] -= 1;
                // This edge keeps its target out of the queue until it is processed.
                vertices[tail] = v;
                tail += usize::from(degree[v] == 0);
                (v, rest) = next_edge(rest);
            }
            rows[u].0 = usize::MAX;
        }
        let mut starts = if tail == n {
            Vec::new()
        } else {
            Self::decompose(&mut vertices, tail, &mut rows, degree, next_edge)
        };
        starts.push(n);
        Self { vertices, starts }
    }

    pub fn size(&self) -> usize {
        self.starts[0] + (self.starts.len() - 1)
    }

    /// Returns the vertex slices in topological order. The order within each slice is unspecified.
    pub fn components(&self) -> impl DoubleEndedIterator<Item = &[usize]> {
        let singletons = self.vertices[..self.starts[0]]
            .iter()
            .map(std::slice::from_ref);
        let groups = self.starts.windows(2).map(|r| &self.vertices[r[0]..r[1]]);
        singletons.chain(groups)
    }

    /// Maps each vertex to its zero-based component index in topological order.
    pub fn component_ids(&self) -> Vec<usize> {
        let mut ids = vec![0; self.vertices.len()];
        for (id, group) in self.components().enumerate() {
            for &v in group {
                ids[v] = id;
            }
        }
        ids
    }

    /// Returns whether a component contains more than one vertex.
    pub fn has_loop(&self) -> bool {
        self.vertices.len() != self.size()
    }

    fn decompose(
        vertices: &mut [usize],
        tail: usize,
        rows: &mut [(usize, usize)],
        mut roots: Vec<usize>,
        next_edge: impl Fn(usize) -> (usize, usize),
    ) -> Vec<usize> {
        let n = vertices.len();
        let mut starts = Vec::new();
        roots.clear();
        let mut branches = Vec::new();
        let mut open = tail;
        let mut end = n;
        for root in 0..n {
            if rows[root].0 > n {
                continue;
            }
            let mut v = root;
            loop {
                if v < n {
                    let (first, rest) = rows[v];
                    if first <= n {
                        // The first edge is consumed on discovery. Its slot then holds the
                        // complemented one-based stack position, or MAX for a closed SCC.
                        // These marks exceed n since rows holds two words per vertex.
                        vertices[open] = v;
                        roots.push(open);
                        open += 1;
                        rows[v].0 = !open;
                        if rest != n {
                            branches.push((open, rest));
                        }
                        v = first;
                        continue;
                    }
                    if first != usize::MAX {
                        let rank = !first;
                        while roots.last().is_some_and(|&i| i >= rank) {
                            roots.pop();
                        }
                    }
                }
                // Roots deeper than the nearest ancestor with unexamined edges
                // are complete. A vertex's last edge needs no return frame.
                let parent = branches.last().map_or(0, |&(rank, _)| rank);
                while roots.last().is_some_and(|&i| i >= parent) {
                    let first = roots.pop().unwrap();
                    let len = open - first;
                    end -= len;
                    vertices.copy_within(first..open, end);
                    for &u in &vertices[end..end + len] {
                        rows[u].0 = usize::MAX;
                    }
                    open = first;
                    starts.push(end);
                }
                let Some((_, cursor)) = branches.last_mut() else {
                    break;
                };
                let (to, rest) = next_edge(*cursor);
                if rest == n {
                    branches.pop();
                } else {
                    *cursor = rest;
                }
                v = to;
            }
        }
        starts.reverse();
        starts
    }
}

#[codesnip::entry(when("StronglyConnectedComponent", "SparseGraph"))]
impl StronglyConnectedComponent {
    /// Builds the condensation graph from the original directed edges.
    pub fn gen_cgraph(
        &self,
        edges: impl IntoIterator<Item = (usize, usize)>,
    ) -> DirectedSparseGraph {
        let ids = self.component_ids();
        let mut used = std::collections::HashSet::new();
        let edges = edges
            .into_iter()
            .filter_map(|(u, v)| {
                let u = ids[u];
                let v = ids[v];
                (u != v && used.insert((u, v))).then_some((u, v))
            })
            .collect();
        DirectedSparseGraph::from_edges(self.size(), edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Xorshift,
        testutil::{integer_cases, sequence_cases},
    };

    #[test]
    fn components() {
        let check = |n: usize, edges: &[(usize, usize)]| {
            let mut reachable: Vec<_> = (0..n).map(|v| 1u64 << v).collect();
            for &(u, v) in edges {
                reachable[u] |= 1 << v;
            }
            for k in 0..n {
                let destinations = reachable[k];
                for row in &mut reachable {
                    if *row >> k & 1 != 0 {
                        *row |= destinations;
                    }
                }
            }
            let scc = StronglyConnectedComponent::from_edges(n, edges.iter().copied());
            let mut ids = vec![usize::MAX; n];
            for (id, group) in scc.components().enumerate() {
                assert!(!group.is_empty());
                for &v in group {
                    assert_eq!(std::mem::replace(&mut ids[v], id), usize::MAX);
                }
            }
            assert!(!ids.contains(&usize::MAX));
            assert_eq!(ids, scc.component_ids());
            assert_eq!(scc.size(), scc.components().count());
            assert_eq!(
                scc.has_loop(),
                scc.components().any(|group| group.len() > 1)
            );
            for u in 0..n {
                for v in 0..n {
                    if reachable[u] >> v & 1 != 0 {
                        if reachable[v] >> u & 1 != 0 {
                            assert_eq!(ids[u], ids[v]);
                        } else {
                            assert!(ids[u] < ids[v]);
                        }
                    } else {
                        assert_ne!(ids[u], ids[v]);
                    }
                }
            }
            let mut expected: Vec<_> = edges
                .iter()
                .map(|&(u, v)| (ids[u], ids[v]))
                .filter(|&(u, v)| u != v)
                .collect();
            expected.sort_unstable();
            expected.dedup();
            let mut graph = scc.gen_cgraph(edges.iter().copied());
            assert_eq!(graph.vertices_size(), scc.size());
            graph.edges.sort_unstable();
            assert_eq!(graph.edges, expected);
        };

        for n in 0..=4 {
            for mask in 0..1u32 << (n * n) {
                let edges: Vec<_> = (0..n * n)
                    .filter(|&i| mask >> i & 1 != 0)
                    .map(|i| (i / n, i % n))
                    .collect();
                check(n, &edges);
            }
        }
        let mut rng = Xorshift::default();
        for n in integer_cases(1usize..=64) {
            for pairs in sequence_cases(&mut rng, 0..=n * n - 1, 0..=n * n) {
                let edges: Vec<_> = pairs.into_iter().map(|i| (i / n, i % n)).collect();
                check(n, &edges);
            }
        }
    }
}
