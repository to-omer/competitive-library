use super::BidirectionalSparseGraph;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DinicBuilder {
    vsize: usize,
    edges: Vec<(usize, usize)>,
    capacities: Vec<u64>,
}
impl DinicBuilder {
    pub fn new(vsize: usize, esize_expect: usize) -> Self {
        Self {
            vsize,
            edges: Vec::with_capacity(esize_expect),
            capacities: Vec::with_capacity(esize_expect * 2),
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize, cap: u64) {
        self.edges.push((from, to));
        self.capacities.push(cap);
        self.capacities.push(0);
    }
    pub fn gen_graph(&mut self) -> BidirectionalSparseGraph {
        let edges = std::mem::take(&mut self.edges);
        BidirectionalSparseGraph::from_edges(self.vsize, edges)
    }
    pub fn build(self, graph: &BidirectionalSparseGraph) -> Dinic<'_> {
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
impl Extend<(usize, usize, u64)> for DinicBuilder {
    fn extend<T: IntoIterator<Item = (usize, usize, u64)>>(&mut self, iter: T) {
        for (from, to, cap) in iter {
            self.add_edge(from, to, cap)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dinic<'a> {
    graph: &'a BidirectionalSparseGraph,
    capacities: Vec<u64>,
    iter: Vec<usize>,
    level: Vec<usize>,
    deq: VecDeque<usize>,
}
impl<'a> Dinic<'a> {
    pub fn builder(vsize: usize, esize_expect: usize) -> DinicBuilder {
        DinicBuilder::new(vsize, esize_expect)
    }
    fn bfs(&mut self, s: usize, t: usize) -> bool {
        use std::usize::MAX;
        self.level.clear();
        self.level.resize(self.graph.vertices_size(), MAX);
        self.level[s] = 0;
        self.deq.clear();
        self.deq.push_back(s);
        while let Some(u) = self.deq.pop_front() {
            for a in self.graph.adjacencies(u) {
                if self.capacities[a.id] > 0 && self.level[a.to] == MAX {
                    self.level[a.to] = self.level[u] + 1;
                    if a.to == t {
                        return false;
                    }
                    self.deq.push_back(a.to);
                }
            }
        }
        self.level[t] == MAX
    }
    fn dfs(&mut self, s: usize, u: usize, upper: u64) -> u64 {
        if u == s {
            return upper;
        }
        let mut res = 0;
        for a in self.graph.adjacencies(u).skip(self.iter[u]) {
            if self.level[u] > self.level[a.to] && self.capacities[a.id ^ 1] > 0 {
                let d = self.dfs(s, a.to, (upper - res).min(self.capacities[a.id ^ 1]));
                if d > 0 {
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
    pub fn maximum_flow_limited(&mut self, s: usize, t: usize, limit: u64) -> u64 {
        let mut flow = 0;
        while flow < limit {
            if self.bfs(s, t) {
                break;
            }
            self.iter.clear();
            self.iter.resize(self.graph.vertices_size(), 0);
            while flow < limit {
                let f = self.dfs(s, t, limit - flow);
                if f == 0 {
                    break;
                }
                flow += f;
            }
        }
        flow
    }
    pub fn maximum_flow(&mut self, s: usize, t: usize) -> u64 {
        self.maximum_flow_limited(s, t, std::u64::MAX)
    }
    pub fn minimum_cut(&mut self, s: usize) -> Vec<bool> {
        let mut visited = vec![false; self.graph.vertices_size()];
        visited[s] = true;
        self.deq.clear();
        self.deq.push_back(s);
        while let Some(u) = self.deq.pop_front() {
            for a in self.graph.adjacencies(u) {
                if self.capacities[a.id] > 0 && !visited[a.to] {
                    visited[a.to] = true;
                    self.deq.push_back(a.to);
                }
            }
        }
        visited
    }
    pub fn get_flow(&self, eid: usize) -> u64 {
        self.capacities[eid * 2 + 1]
    }
    pub fn change_edge(&mut self, eid: usize, cap: u64, flow: u64) {
        assert!(flow <= cap);
        self.capacities[eid * 2] = cap - flow;
        self.capacities[eid * 2 + 1] = flow;
    }
}
