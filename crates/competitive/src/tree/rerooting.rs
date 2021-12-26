//! dynamic programming on all-rooted trees

use crate::algebra::Monoid;
use crate::graph::{Adjacency, UndirectedSparseGraph};

#[cfg_attr(
    nightly,
    codesnip::entry("ReRooting", include("algebra", "SparseGraph"))
)]
/// dynamic programming on all-rooted trees
///
/// caluculate all subtrees (hanging on the edge) in specific ordering,
/// each subtree calculated in the order of merge and rooting
#[derive(Clone, Debug)]
pub struct ReRooting<'a, M: Monoid, F: Fn(&M::T, usize, Option<usize>) -> M::T> {
    graph: &'a UndirectedSparseGraph,
    /// dp\[v\]: result of v-rooted tree
    pub dp: Vec<M::T>,
    /// ep\[e\]: result of e-subtree, if e >= n then reversed-e-subtree
    pub ep: Vec<M::T>,
    /// rooting(data, vid, (Optional)eid): add root node(vid), result subtree is edge(eid)
    rooting: F,
}
#[cfg_attr(nightly, codesnip::entry("ReRooting"))]
impl<'a, M, F> ReRooting<'a, M, F>
where
    M: Monoid,
    F: Fn(&M::T, usize, Option<usize>) -> M::T,
{
    pub fn new(graph: &'a UndirectedSparseGraph, rooting: F) -> Self {
        let dp = vec![M::unit(); graph.vertices_size()];
        let ep = vec![M::unit(); graph.vertices_size() * 2];
        let mut self_ = Self {
            graph,
            dp,
            ep,
            rooting,
        };
        self_.rerooting();
        self_
    }
    #[inline]
    fn eidx(&self, u: usize, a: &Adjacency) -> usize {
        a.id + self.graph.edges_size() * (u > a.to) as usize
    }
    #[inline]
    fn reidx(&self, u: usize, a: &Adjacency) -> usize {
        a.id + self.graph.edges_size() * (u < a.to) as usize
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
    fn dfs(&mut self, pa: &Adjacency, p: usize) {
        let u = pa.to;
        let pi = self.eidx(p, pa);
        for a in self.graph.adjacencies(u).filter(|a| a.to != p) {
            let i = self.eidx(u, a);
            self.dfs(a, u);
            self.ep[pi] = self.merge(&self.ep[pi], &self.ep[i]);
        }
        self.ep[pi] = self.add_subroot(&self.ep[pi], u, pa.id);
    }
    fn efs(&mut self, u: usize, p: usize) {
        let m = self.graph.adjacencies(u).len();
        let mut left = vec![M::unit(); m + 1];
        let mut right = vec![M::unit(); m + 1];
        for (k, a) in self.graph.adjacencies(u).enumerate() {
            let i = self.eidx(u, a);
            left[k + 1] = self.merge(&left[k], &self.ep[i]);
        }
        for (k, a) in self.graph.adjacencies(u).enumerate().rev() {
            let i = self.eidx(u, a);
            right[k] = self.merge(&right[k + 1], &self.ep[i]);
        }
        self.dp[u] = self.add_root(&left[m], u);
        for (k, a) in self.graph.adjacencies(u).enumerate() {
            if a.to != p {
                let i = self.reidx(u, a);
                self.ep[i] = self.merge(&left[k], &right[k + 1]);
                self.ep[i] = self.add_subroot(&self.ep[i], u, a.id);
                self.efs(a.to, u);
            }
        }
    }
    fn rerooting(&mut self) {
        for a in self.graph.adjacencies(0) {
            self.dfs(a, 0);
        }
        self.efs(0, std::usize::MAX);
    }
}
