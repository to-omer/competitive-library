use crate::algebra::base::Monoid;
use crate::graph::*;

#[cargo_snippet::snippet("ReRooting")]
#[derive(Clone, Debug)]
pub struct ReRooting<M: Monoid, F: Fn(&M::T, usize, Option<usize>) -> M::T> {
    n: usize,
    monoid: M,
    dp: Vec<M::T>,
    ep: Vec<M::T>,
    rooting: F,
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
    pub fn eidx(&self, u: usize, a: &Adjacent) -> usize {
        a.id + (self.n - 1) * (u > a.to) as usize
    }
    #[inline]
    pub fn reidx(&self, u: usize, a: &Adjacent) -> usize {
        a.id + (self.n - 1) * (u < a.to) as usize
    }
    #[inline]
    pub fn merge(&self, x: &M::T, y: &M::T) -> M::T {
        self.monoid.operate(x, y)
    }
    #[inline]
    pub fn add_subroot(&self, x: &M::T, vid: usize, eid: usize) -> M::T {
        (self.rooting)(x, vid, Some(eid))
    }
    #[inline]
    pub fn add_root(&self, x: &M::T, vid: usize) -> M::T {
        (self.rooting)(x, vid, None)
    }
    pub fn dfs(&mut self, pa: &Adjacent, p: usize, graph: &Graph) {
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
    pub fn efs(&mut self, u: usize, p: usize, graph: &Graph) {
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
