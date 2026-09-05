use super::{BidirectionalSparseGraph, Graph};
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Debug, Clone)]
pub struct PrimalDualBuilder {
    vsize: usize,
    edges: Vec<(usize, usize)>,
    capacities: Vec<u64>,
    costs: Vec<i64>,
    has_negedge: bool,
}
impl PrimalDualBuilder {
    pub fn new(vsize: usize, esize_expect: usize) -> Self {
        Self {
            vsize,
            edges: Vec::with_capacity(esize_expect),
            capacities: Vec::with_capacity(esize_expect * 2),
            costs: Vec::with_capacity(esize_expect * 2),
            has_negedge: false,
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize, cap: u64, cost: i64) {
        self.edges.push((from, to));
        self.capacities.push(cap);
        self.capacities.push(0);
        self.has_negedge |= cost < 0;
        self.costs.push(cost);
        self.costs.push(-cost);
    }
    pub fn gen_graph(&mut self) -> BidirectionalSparseGraph {
        let edges = std::mem::take(&mut self.edges);
        BidirectionalSparseGraph::from_edges(self.vsize, edges)
    }
    pub fn build(self, graph: &BidirectionalSparseGraph) -> PrimalDual<'_> {
        let PrimalDualBuilder {
            vsize,
            capacities,
            costs,
            has_negedge,
            ..
        } = self;
        PrimalDual {
            graph,
            capacities,
            costs,
            potential: std::iter::repeat_n(0, vsize).collect(),
            dist: Vec::with_capacity(vsize),
            prev_vertex: std::iter::repeat_n(0, vsize).collect(),
            prev_edge: std::iter::repeat_n(0, vsize).collect(),
            has_negedge,
            queue: BinaryHeap::new(),
        }
    }
}
impl Extend<(usize, usize, u64, i64)> for PrimalDualBuilder {
    fn extend<T: IntoIterator<Item = (usize, usize, u64, i64)>>(&mut self, iter: T) {
        for (from, to, cap, cost) in iter {
            self.add_edge(from, to, cap, cost)
        }
    }
}

#[derive(Debug)]
pub struct PrimalDual<'a> {
    graph: &'a BidirectionalSparseGraph,
    capacities: Vec<u64>,
    costs: Vec<i64>,
    potential: Vec<i64>,
    dist: Vec<i64>,
    prev_vertex: Vec<usize>,
    prev_edge: Vec<usize>,
    has_negedge: bool,
    queue: BinaryHeap<(Reverse<i64>, usize)>,
}
impl PrimalDual<'_> {
    pub fn builder(vsize: usize, esize_expect: usize) -> PrimalDualBuilder {
        PrimalDualBuilder::new(vsize, esize_expect)
    }
    fn bellman_ford(&mut self, s: usize) {
        self.potential.clear();
        self.potential.resize(self.graph.vertices_size(), i64::MAX);
        self.potential[s] = 0;
        for _ in 1..self.graph.vertices_size() {
            let mut end = true;
            for u in self.graph.vertices() {
                if self.potential[u] == i64::MAX {
                    continue;
                }
                for a in self.graph.neighbors(u) {
                    if self.capacities[a.label] == 0 {
                        continue;
                    }
                    let ncost = self.potential[u].saturating_add(self.costs[a.label]);
                    if self.potential[a.to] > ncost {
                        self.potential[a.to] = ncost;
                        end = false;
                    }
                }
            }
            if end {
                break;
            }
        }
    }
    fn dijkstra(&mut self, s: usize, t: usize) -> bool {
        self.dist.clear();
        self.dist.resize(self.graph.vertices_size(), i64::MAX);
        self.dist[s] = 0;
        self.queue.clear();
        self.queue.push((Reverse(0), s));
        while let Some((Reverse(d), u)) = self.queue.pop() {
            if d >= self.dist[t] {
                break;
            }
            if self.dist[u] < d {
                continue;
            }
            for a in self.graph.neighbors(u) {
                if self.capacities[a.label] == 0 {
                    continue;
                }
                let ncost = (d.saturating_add(self.costs[a.label]))
                    .saturating_add(self.potential[u].saturating_sub(self.potential[a.to]));
                if self.dist[a.to] > ncost {
                    debug_assert!(ncost >= d);
                    self.dist[a.to] = ncost;
                    self.prev_vertex[a.to] = u;
                    self.prev_edge[a.to] = a.label;
                    self.queue.push((Reverse(ncost), a.to));
                }
            }
        }
        self.dist[t] != i64::MAX
    }
    /// Return (flow, cost).
    pub fn minimum_cost_flow_limited(&mut self, s: usize, t: usize, limit: u64) -> (u64, i64) {
        let mut flow = 0;
        let mut cost = 0;
        if self.has_negedge {
            self.bellman_ford(s);
        }
        while flow < limit && self.dijkstra(s, t) {
            let shortest = self.dist[t];
            for (p, d) in self.potential.iter_mut().zip(self.dist.iter()) {
                *p = p.saturating_add((*d).min(shortest));
            }
            let mut f = limit - flow;
            let mut v = t;
            while v != s {
                f = f.min(self.capacities[self.prev_edge[v]]);
                v = self.prev_vertex[v];
            }
            flow += f;
            cost += f as i64 * (self.potential[t] - self.potential[s]);
            let mut v = t;
            while v != s {
                self.capacities[self.prev_edge[v]] -= f;
                self.capacities[self.prev_edge[v] ^ 1] += f;
                v = self.prev_vertex[v];
            }
        }
        (flow, cost)
    }
    /// Return (flow, cost).
    pub fn minimum_cost_flow(&mut self, s: usize, t: usize) -> (u64, i64) {
        self.minimum_cost_flow_limited(s, t, u64::MAX)
    }
    pub fn get_flow(&self, eid: usize) -> u64 {
        self.capacities[eid * 2 + 1]
    }
}
