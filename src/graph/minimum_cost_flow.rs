use crate::data_structure::Rev;
use cargo_snippet::snippet;

#[snippet("MinimumCostFlow")]
#[derive(Debug, Clone)]
pub struct RevEdge {
    pub to: usize,
    pub rev: usize,
    pub cap: u64,
    pub cost: i64,
}
#[snippet("MinimumCostFlow")]
impl RevEdge {
    pub fn new(to: usize, rev: usize, cap: u64, cost: i64) -> RevEdge {
        RevEdge {
            to: to,
            rev: rev,
            cap: cap,
            cost: cost,
        }
    }
}

#[snippet("MinimumCostFlow")]
#[derive(Debug)]
pub struct PrimalDual {
    n: usize,
    graph: Vec<Vec<RevEdge>>,
    potential: Vec<i64>,
    cost: Vec<i64>,
    prev_vertex: Vec<usize>,
    prev_edge: Vec<usize>,
}
#[snippet("MinimumCostFlow")]
impl PrimalDual {
    pub fn new(n: usize) -> PrimalDual {
        PrimalDual {
            n: n,
            graph: vec![vec![]; n],
            potential: vec![],
            cost: vec![],
            prev_vertex: vec![],
            prev_edge: vec![],
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize, cap: u64, cost: i64) {
        let e1 = RevEdge::new(to, self.graph[to].len(), cap, cost);
        let e2 = RevEdge::new(from, self.graph[from].len(), 0, -cost);
        self.graph[from].push(e1);
        self.graph[to].push(e2);
    }
    pub fn minimum_cost_flow(&mut self, s: usize, t: usize, f: u64) -> Option<i64> {
        use std::cmp::min;
        let mut res = 0;
        let mut f = f;
        self.potential = vec![0; self.n];
        self.prev_edge = vec![0; self.n];
        self.prev_vertex = vec![0; self.n];
        while f > 0 {
            self.dijkstra(s);
            if self.cost[t] == std::i64::MAX {
                return None;
            }
            for v in 0..self.n {
                self.potential[v] += self.cost[v];
            }
            let mut add_f = f;
            let mut v = t;
            while v != s {
                add_f = min(
                    add_f,
                    self.graph[self.prev_vertex[v]][self.prev_edge[v]].cap,
                );
                v = self.prev_vertex[v];
            }
            f -= add_f;
            res += add_f as i64 * self.potential[t];
            let mut v = t;
            while v != s {
                self.graph[self.prev_vertex[v]][self.prev_edge[v]].cap -= add_f;
                let r = self.graph[self.prev_vertex[v]][self.prev_edge[v]].rev;
                self.graph[v][r].cap += add_f;
                v = self.prev_vertex[v];
            }
        }
        Some(res)
    }
    fn dijkstra(&mut self, s: usize) {
        use std::collections::BinaryHeap;
        self.cost = vec![std::i64::MAX; self.n];
        let mut heap = BinaryHeap::new();
        self.cost[s] = 0;
        heap.push((Rev(0), s));
        while let Some((d, u)) = heap.pop() {
            let d = d.0;
            for i in 0..self.graph[u].len() {
                let e = &self.graph[u][i];
                let ncost = self.cost[u] + e.cost + self.potential[u] - self.potential[e.to];
                if e.cap > 0 && self.cost[e.to] > ncost {
                    self.cost[e.to] = ncost;
                    self.prev_vertex[e.to] = u;
                    self.prev_edge[e.to] = i;
                    heap.push((Rev(d + e.cost), e.to));
                }
            }
        }
    }
}
