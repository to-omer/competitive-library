use crate::graph::{Graph, UndirectedSparseGraph};

#[codesnip::entry("tree_center")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeCenter {
    One(usize),
    Two(usize, usize),
}
#[codesnip::entry("tree_center", include("SparseGraph"))]
impl UndirectedSparseGraph {
    /// tree center
    pub fn tree_center(&self) -> TreeCenter {
        let n = self.vertices_size();
        assert_ne!(n, 0);
        let mut deq = std::collections::VecDeque::with_capacity(n);
        let mut deg: Vec<_> = self.vertices().map(|u| self.neighbors(u).len()).collect();
        for u in self.vertices() {
            if self.neighbors(u).len() <= 1 {
                deq.push_back(u);
            }
        }
        let mut k = 0;
        let mut cnt = deq.len();
        if cnt < n {
            k = deq.len();
            'outer: while let Some(u) = deq.pop_front() {
                k -= 1;
                for a in self.neighbors(u) {
                    deg[a.to] -= 1;
                    if deg[a.to] == 1 {
                        deq.push_back(a.to);
                        cnt += 1;
                        if cnt == n {
                            break 'outer;
                        }
                    }
                }
                if k == 0 {
                    k = deq.len();
                }
            }
        }
        if deq.len() == k + 1 {
            TreeCenter::One(*deq.back().unwrap())
        } else {
            let u = deq.pop_back().unwrap();
            let v = deq.pop_back().unwrap();
            if u < v {
                TreeCenter::Two(u, v)
            } else {
                TreeCenter::Two(v, u)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::UndirectedSparseGraph,
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, sample_usize},
        },
        tree::{MixedTree, PathTree, PruferSequence, StarTree},
    };

    impl UndirectedSparseGraph {
        fn naive_tree_center(&self) -> TreeCenter {
            let mut md: Vec<_> = self
                .vertices()
                .map(|u| (self.tree_depth(u).into_iter().max().unwrap_or_default(), u))
                .collect();
            md.sort_unstable();
            if md.len() == 1 {
                TreeCenter::One(md[0].1)
            } else if md.len() >= 2 {
                if md[0].0 == md[1].0 {
                    TreeCenter::Two(md[0].1, md[1].1)
                } else {
                    TreeCenter::One(md[0].1)
                }
            } else {
                panic!("vertex size should be larger than one.");
            }
        }
    }

    #[test]
    fn test_center() {
        // Prüfer sequences enumerate every labelled tree through six vertices.
        let exhaustive = (1usize..=6).flat_map(|n| {
            let len = n.saturating_sub(2);
            exhaustive_sequences(0..n, len..=len).map(move |sequence| {
                let mut degrees = vec![1; n];
                for &v in &sequence {
                    degrees[v] += 1;
                }
                let mut edges = Vec::new();
                for v in sequence {
                    let leaf = degrees.iter().position(|&d| d == 1).unwrap();
                    edges.push((leaf, v));
                    degrees[leaf] -= 1;
                    degrees[v] -= 1;
                }
                let leaves: Vec<_> = (0..n).filter(|&v| degrees[v] == 1).collect();
                if let &[a, b] = leaves.as_slice() {
                    edges.push((a, b));
                }
                UndirectedSparseGraph::from_edges(n, edges)
            })
        });
        let mut rng = Xorshift::default();
        let random = sample_usize(&mut rng, 16, 1..=200, 200)
            .into_iter()
            .flat_map(|n| {
                [
                    rng.random(PathTree(n)),
                    rng.random(StarTree(n)),
                    rng.random(PruferSequence(n)),
                    rng.random(MixedTree(n)),
                ]
            });
        for graph in exhaustive.chain(random) {
            assert_eq!(
                graph.tree_center(),
                graph.naive_tree_center(),
                "graph={graph:?}"
            );
        }
    }
}
