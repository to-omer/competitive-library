pub mod base;
pub mod maximum_flow;
pub mod minimum_cost_flow;
pub mod strongly_connected_component;
pub mod topological_sort;

pub use base::*;

// use cargo_snippet::snippet;

// #[snippet("DirectedAcyclicGraph")]
#[derive(Debug)]
pub struct DirectedAcyclicGraph {
    graph: Vec<Vec<usize>>,
    cost: Vec<Option<usize>>,
}
// #[snippet("DirectedAcyclicGraph")]
impl DirectedAcyclicGraph {
    pub fn new(n: usize) -> DirectedAcyclicGraph {
        DirectedAcyclicGraph {
            graph: vec![vec![]; n],
            cost: vec![None; n],
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.graph[u].push(v)
    }
    pub fn bfs(&mut self, s: usize, t: usize) -> Option<usize> {
        use std::collections::VecDeque;
        self.cost = vec![None; self.graph.len()];
        let mut deq = VecDeque::new();
        self.cost[s] = Some(0);
        deq.push_back(s);
        while let Some(u) = deq.pop_front() {
            if u == t {
                return self.cost[u];
            }
            let d = self.cost[u].unwrap();
            for &v in &self.graph[u] {
                if self.cost[v].is_none() {
                    self.cost[v] = Some(d + 1);
                    deq.push_back(v);
                }
            }
        }
        None
    }
}
