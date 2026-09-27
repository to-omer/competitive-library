//! dynamic programming on all-rooted trees

use crate::algebra::{AbelianGroup, Monoid};
use crate::graph::{Graph, Neighbor, UndirectedSparseGraph};

#[codesnip::entry("ReRooting", include("algebra", "tree_order"))]
/// dynamic programming on all-rooted trees
///
/// Neighbors are merged in adjacency order before applying `rooting`.
#[derive(Clone, Debug)]
pub struct ReRooting<'a, M: Monoid, F: Fn(&M::T, usize, Option<usize>) -> M::T> {
    graph: &'a UndirectedSparseGraph,
    /// dp\[v\]: result of v-rooted tree
    pub dp: Vec<M::T>,
    /// ep\[e\]: result of e-subtree; e >= m denotes the reverse direction.
    pub ep: Vec<M::T>,
    /// rooting(data, vid, (Optional)eid): add root node(vid), result subtree is edge(eid)
    rooting: F,
}
#[codesnip::entry("ReRooting")]
impl<'a, M, F> ReRooting<'a, M, F>
where
    M: Monoid,
    F: Fn(&M::T, usize, Option<usize>) -> M::T,
{
    pub fn new(graph: &'a UndirectedSparseGraph, rooting: F) -> Self {
        Self::build(graph, rooting, None::<fn(&M::T, &M::T) -> M::T>)
    }

    pub fn new_with_inverse(graph: &'a UndirectedSparseGraph, rooting: F) -> Self
    where
        M: AbelianGroup,
    {
        Self::build(graph, rooting, Some(M::rinv_operate))
    }

    fn build<I>(graph: &'a UndirectedSparseGraph, rooting: F, inverse: Option<I>) -> Self
    where
        I: Fn(&M::T, &M::T) -> M::T,
    {
        let dp = vec![M::unit(); graph.vertices_size()];
        let ep = vec![M::unit(); graph.vertices_size() * 2];
        let mut self_ = Self {
            graph,
            dp,
            ep,
            rooting,
        };
        self_.rerooting(inverse);
        self_
    }
    #[inline]
    fn eidx(&self, u: usize, a: Neighbor<usize, usize>) -> usize {
        a.label + self.graph.edges_size() * (u > a.to) as usize
    }
    #[inline]
    fn reidx(&self, u: usize, a: Neighbor<usize, usize>) -> usize {
        a.label + self.graph.edges_size() * (u < a.to) as usize
    }
    #[inline]
    fn merge(&self, x: &M::T, y: &M::T) -> M::T {
        M::operate(x, y)
    }
    #[inline]
    fn add_subroot(&self, x: &M::T, vid: usize, eid: usize) -> M::T {
        (self.rooting)(x, vid, Some(eid))
    }
    #[inline]
    fn add_root(&self, x: &M::T, vid: usize) -> M::T {
        (self.rooting)(x, vid, None)
    }
    fn rerooting<I: Fn(&M::T, &M::T) -> M::T>(&mut self, inverse: Option<I>) {
        let (order, parents) = self.graph.tree_order(0);
        for &u in order.iter().skip(1).rev() {
            let mut sum = M::unit();
            let mut parent = None;
            for a in self.graph.neighbors(u) {
                if a.to == parents[u] {
                    parent = Some(a);
                } else {
                    sum = self.merge(&sum, &self.ep[self.eidx(u, a)]);
                }
            }
            let a = parent.unwrap();
            let i = self.reidx(u, a);
            self.ep[i] = self.add_subroot(&sum, u, a.label);
            if inverse.is_some() {
                self.dp[u] = sum;
            }
        }
        if let Some(inverse) = inverse {
            for u in order {
                let sum = if u == 0 {
                    self.graph.neighbors(u).fold(M::unit(), |sum, a| {
                        self.merge(&sum, &self.ep[self.eidx(u, a)])
                    })
                } else {
                    let a = self
                        .graph
                        .neighbors(u)
                        .find(|a| a.to == parents[u])
                        .unwrap();
                    self.merge(&self.dp[u], &self.ep[self.eidx(u, a)])
                };
                self.dp[u] = self.add_root(&sum, u);
                for a in self.graph.neighbors(u) {
                    if a.to != parents[u] {
                        let value = inverse(&sum, &self.ep[self.eidx(u, a)]);
                        let i = self.reidx(u, a);
                        self.ep[i] = self.add_subroot(&value, u, a.label);
                    }
                }
            }
            return;
        }
        let mut prefix = Vec::new();
        for u in order {
            prefix.clear();
            prefix.push(M::unit());
            for a in self.graph.neighbors(u) {
                prefix.push(self.merge(prefix.last().unwrap(), &self.ep[self.eidx(u, a)]));
            }
            self.dp[u] = self.add_root(prefix.last().unwrap(), u);
            let mut suffix = M::unit();
            for (k, a) in self.graph.neighbors(u).enumerate().rev() {
                if a.to != parents[u] {
                    let i = self.reidx(u, a);
                    self.ep[i] = self.add_subroot(&self.merge(&prefix[k], &suffix), u, a.label);
                }
                suffix = self.merge(&self.ep[self.eidx(u, a)], &suffix);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, ConcatenateOperation},
        tools::{Xorshift, testutil::exhaustive_sequences},
        tree::{MixedTree, PathTree, StarTree},
    };

    #[test]
    fn test_rerooting() {
        let mut rng = Xorshift::default();
        let mut graphs = Vec::new();
        for n in 1..=5 {
            for parents in exhaustive_sequences(0..n, n - 1..=n - 1) {
                if parents.iter().enumerate().all(|(i, &p)| p <= i) {
                    graphs.push(UndirectedSparseGraph::from_edges(
                        n,
                        parents
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| (p, i + 1))
                            .collect(),
                    ));
                }
            }
        }
        for n in 1..=32 {
            graphs.extend([
                rng.random(PathTree(n)),
                rng.random(StarTree(n)),
                rng.random(MixedTree(n)),
            ]);
        }
        for graph in graphs {
            let n = graph.vertices_size();
            let dp = ReRooting::<ConcatenateOperation<_>, _>::new(&graph, |xs, v, edge| {
                let mut xs = xs.clone();
                xs.push((v, edge));
                xs
            });
            let sums = ReRooting::<AdditiveOperation<i64>, _>::new_with_inverse(
                &graph,
                |&sum, v, edge| sum + v as i64 + edge.map_or(0, |e| e as i64),
            );
            for root in 0..n {
                for parent in std::iter::once(None).chain(graph.neighbors(root).map(Some)) {
                    let mut expected = Vec::new();
                    let mut stack = vec![(
                        root,
                        parent.map_or(n, |a| a.to),
                        parent.map(|a| a.label),
                        false,
                    )];
                    while let Some((u, p, edge, visited)) = stack.pop() {
                        if visited {
                            expected.push((u, edge));
                        } else {
                            stack.push((u, p, edge, true));
                            stack.extend(
                                graph
                                    .neighbors(u)
                                    .rev()
                                    .filter(|a| a.to != p)
                                    .map(|a| (a.to, u, Some(a.label), false)),
                            );
                        }
                    }
                    let actual = if let Some(parent) = parent {
                        &dp.ep[dp.reidx(root, parent)]
                    } else {
                        &dp.dp[root]
                    };
                    assert_eq!(*actual, expected);
                    let sum = if let Some(parent) = parent {
                        sums.ep[sums.reidx(root, parent)]
                    } else {
                        sums.dp[root]
                    };
                    assert_eq!(
                        sum,
                        expected
                            .iter()
                            .map(|&(v, e)| v as i64 + e.map_or(0, |e| e as i64))
                            .sum()
                    );
                }
            }
        }
    }
}
