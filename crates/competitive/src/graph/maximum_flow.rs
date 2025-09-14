use super::{BidirectionalSparseGraph, Bounded, Zero};
use std::{
    collections::VecDeque,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[derive(Debug, Clone)]
pub struct DinicBuilder<C> {
    vsize: usize,
    edges: Vec<(usize, usize)>,
    capacities: Vec<C>,
}
impl<C> DinicBuilder<C> {
    pub fn new(vsize: usize, esize_expect: usize) -> Self {
        Self {
            vsize,
            edges: Vec::with_capacity(esize_expect),
            capacities: Vec::with_capacity(esize_expect * 2),
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize, cap: C)
    where
        C: Zero + PartialOrd,
    {
        self.edges.push((from, to));
        assert!(cap >= C::zero());
        self.capacities.push(cap);
        self.capacities.push(C::zero());
    }
    pub fn gen_graph(&mut self) -> BidirectionalSparseGraph {
        let edges = std::mem::take(&mut self.edges);
        BidirectionalSparseGraph::from_edges(self.vsize, edges)
    }
    pub fn build(self, graph: &BidirectionalSparseGraph) -> Dinic<'_, C> {
        let DinicBuilder {
            vsize, capacities, ..
        } = self;
        Dinic {
            graph,
            capacities,
            iter: Vec::with_capacity(vsize),
            level: Vec::with_capacity(vsize),
            deq: VecDeque::with_capacity(vsize),
        }
    }
}
impl<C> Extend<(usize, usize, C)> for DinicBuilder<C>
where
    C: Zero + PartialOrd,
{
    fn extend<I: IntoIterator<Item = (usize, usize, C)>>(&mut self, iter: I) {
        for (from, to, cap) in iter {
            self.add_edge(from, to, cap)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dinic<'a, C> {
    graph: &'a BidirectionalSparseGraph,
    capacities: Vec<C>,
    iter: Vec<usize>,
    level: Vec<usize>,
    deq: VecDeque<usize>,
}
impl<'a, C> Dinic<'a, C>
where
    C: Copy + Zero + Ord + Bounded + Add<Output = C> + Sub<Output = C> + AddAssign + SubAssign,
{
    pub fn builder(vsize: usize, esize_expect: usize) -> DinicBuilder<C> {
        DinicBuilder::new(vsize, esize_expect)
    }
    fn bfs(&mut self, s: usize, t: usize) -> bool {
        self.level.clear();
        self.level.resize(self.graph.vertices_size(), usize::MAX);
        self.level[s] = 0;
        self.deq.clear();
        self.deq.push_back(s);
        while let Some(u) = self.deq.pop_front() {
            for a in self.graph.adjacencies(u) {
                if self.capacities[a.id] > C::zero() && self.level[a.to] == usize::MAX {
                    self.level[a.to] = self.level[u] + 1;
                    if a.to == t {
                        return false;
                    }
                    self.deq.push_back(a.to);
                }
            }
        }
        self.level[t] == usize::MAX
    }
    fn dfs(&mut self, s: usize, u: usize, upper: C) -> C {
        if u == s {
            return upper;
        }
        let mut res = C::zero();
        for a in self.graph.adjacencies(u).skip(self.iter[u]) {
            if self.level[u] > self.level[a.to] && self.capacities[a.id ^ 1] > C::zero() {
                let d = self.dfs(s, a.to, (upper - res).min(self.capacities[a.id ^ 1]));
                if d > C::zero() {
                    self.capacities[a.id ^ 1] -= d;
                    self.capacities[a.id] += d;
                    res += d;
                    if upper == res {
                        break;
                    }
                }
            }
            self.iter[u] += 1;
        }
        res
    }
    pub fn maximum_flow_limited(&mut self, s: usize, t: usize, limit: C) -> C {
        let mut flow = C::zero();
        while flow < limit {
            if self.bfs(s, t) {
                break;
            }
            self.iter.clear();
            self.iter.resize(self.graph.vertices_size(), 0);
            while flow < limit {
                let f = self.dfs(s, t, limit - flow);
                if f == C::zero() {
                    break;
                }
                flow += f;
            }
        }
        flow
    }
    pub fn maximum_flow(&mut self, s: usize, t: usize) -> C {
        self.maximum_flow_limited(s, t, C::maximum())
    }
    pub fn minimum_cut(&mut self, s: usize) -> Vec<bool> {
        let mut visited = vec![false; self.graph.vertices_size()];
        visited[s] = true;
        self.deq.clear();
        self.deq.push_back(s);
        while let Some(u) = self.deq.pop_front() {
            for a in self.graph.adjacencies(u) {
                if self.capacities[a.id] > C::zero() && !visited[a.to] {
                    visited[a.to] = true;
                    self.deq.push_back(a.to);
                }
            }
        }
        visited
    }
    pub fn get_flow(&self, eid: usize) -> C {
        self.capacities[eid * 2 + 1]
    }
    pub fn change_edge(&mut self, eid: usize, cap: C, flow: C) {
        assert!(flow <= cap);
        self.capacities[eid * 2] = cap - flow;
        self.capacities[eid * 2 + 1] = flow;
    }
}
