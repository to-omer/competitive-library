//! dynamic programming on all-rooted trees

use crate::algebra::magma::Monoid;
use crate::graph::*;

#[cargo_snippet::snippet("ReRooting")]
/// dynamic programming on all-rooted trees
///
/// caluculate all subtrees (hanging on the edge) in specific ordering,
/// each subtree calculated in the order of merge and rooting
#[derive(Clone, Debug)]
pub struct ReRooting<M: Monoid, F: Fn(&M::T, usize, Option<usize>) -> M::T> {
    /// vertex size
    pub n: usize,
    /// merge subtree
    pub monoid: M,
    /// dp\[v\]: result of v-rooted tree
    pub dp: Vec<M::T>,
    /// ep\[e\]: result of e-subtree, if e >= n then reversed-e-subtree
    pub ep: Vec<M::T>,
    /// rooting(data, vid, (Optional)eid): add root node(vid), result subtree is edge(eid)
    pub rooting: F,
}
#[cargo_snippet::snippet("ReRooting")]
impl<M: Monoid, F: Fn(&M::T, usize, Option<usize>) -> M::T> ReRooting<M, F> {
    pub fn new(n: usize, monoid: M, rooting: F) -> Self {
        let dp = vec![monoid.unit(); n];
        let ep = vec![monoid.unit(); n * 2];
        ReRooting {
            n: n,
            monoid: monoid,
            dp: dp,
            ep: ep,
            rooting: rooting,
        }
    }
    #[inline]
    fn eidx(&self, u: usize, a: &Adjacent) -> usize {
        a.id + (self.n - 1) * (u > a.to) as usize
    }
    #[inline]
    fn reidx(&self, u: usize, a: &Adjacent) -> usize {
        a.id + (self.n - 1) * (u < a.to) as usize
    }
    #[inline]
    fn merge(&self, x: &M::T, y: &M::T) -> M::T {
        self.monoid.operate(x, y)
    }
    #[inline]
    fn add_subroot(&self, x: &M::T, vid: usize, eid: usize) -> M::T {
        (self.rooting)(x, vid, Some(eid))
    }
    #[inline]
    fn add_root(&self, x: &M::T, vid: usize) -> M::T {
        (self.rooting)(x, vid, None)
    }
    fn dfs(&mut self, pa: &Adjacent, p: usize, graph: &Graph) {
        let u = pa.to;
        let pi = self.eidx(p, pa);
        for a in graph.adjacency(u) {
            let i = self.eidx(u, a);
            if a.to != p {
                self.dfs(a, u, graph);
                self.ep[pi] = self.merge(&self.ep[pi], &self.ep[i]);
            }
        }
        self.ep[pi] = self.add_subroot(&self.ep[pi], u, pa.id);
    }
    fn efs(&mut self, u: usize, p: usize, graph: &Graph) {
        let adjacency = graph.adjacency(u);
        let m = adjacency.len();
        let mut left = vec![self.monoid.unit(); m + 1];
        let mut right = vec![self.monoid.unit(); m + 1];
        for (k, a) in adjacency.iter().enumerate() {
            let i = self.eidx(u, a);
            left[k + 1] = self.merge(&left[k], &self.ep[i]);
        }
        for (k, a) in adjacency.iter().enumerate().rev() {
            let i = self.eidx(u, a);
            right[k] = self.merge(&right[k + 1], &self.ep[i]);
        }
        self.dp[u] = self.add_root(&left[m], u);
        for (k, a) in adjacency.iter().enumerate() {
            if a.to != p {
                let i = self.reidx(u, a);
                self.ep[i] = self.merge(&left[k], &right[k + 1]);
                self.ep[i] = self.add_subroot(&self.ep[i], u, a.id);
                self.efs(a.to, u, graph);
            }
        }
    }
    pub fn rerooting(&mut self, graph: &Graph) {
        let n = self.n;
        for a in graph.adjacency(0) {
            self.dfs(a, 0, graph);
        }
        self.efs(0, n, graph);
    }
}
