use crate::data_structure::Rev;

#[cargo_snippet::snippet("WeightedGraph")]
#[derive(Clone, Debug)]
pub struct WeightedGraph<T> {
    graph: Vec<Vec<(usize, T)>>,
}
#[cargo_snippet::snippet("WeightedGraph")]
impl<T: Clone> WeightedGraph<T> {
    pub fn new(n: usize) -> WeightedGraph<T> {
        WeightedGraph {
            graph: vec![vec![]; n],
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize, c: T) {
        self.graph[u].push((v, c))
    }
}

#[cargo_snippet::snippet("dijkstra")]
#[cargo_snippet::snippet(include = "WeightedGraph")]
impl WeightedGraph<usize> {
    pub fn dijkstra(&self, s: usize) -> Vec<usize> {
        use std::collections::BinaryHeap;
        const INF: usize = std::usize::MAX;
        let mut cost = vec![INF; self.graph.len()];
        let mut heap = BinaryHeap::new();
        cost[s] = 0;
        heap.push((Rev(0), s));
        while let Some((d, u)) = heap.pop() {
            let d = d.0;
            for &(v, c) in self.graph[u].iter() {
                if cost[v] > d + c {
                    cost[v] = d + c;
                    heap.push((Rev(d + c), v));
                }
            }
        }
        cost
    }
}

#[cargo_snippet::snippet("bellman_ford")]
#[cargo_snippet::snippet(include = "WeightedGraph")]
impl WeightedGraph<i64> {
    pub fn bellman_ford(&self, s: usize) -> (Vec<i64>, bool) {
        const INF: i64 = std::i64::MAX;
        let n = self.graph.len();
        let mut cost = vec![INF; n];
        cost[s] = 0;
        for i in 0..n {
            for u in 0..n {
                if cost[u] == INF {
                    continue;
                }
                for &(v, c) in self.graph[u].iter() {
                    if cost[v] > cost[u] + c {
                        if i == n - 1 {
                            return (cost, true);
                        }
                        cost[v] = cost[u] + c;
                    }
                }
            }
        }
        (cost, false)
    }
}

#[derive(Debug)]
struct EdgeGraph {
    n: usize,
    edges: Vec<(usize, usize, i64)>,
}
impl EdgeGraph {
    fn new(n: usize) -> EdgeGraph {
        EdgeGraph {
            n: n,
            edges: vec![],
        }
    }
    fn add_edge(&mut self, u: usize, v: usize, c: i64) {
        self.edges.push((u, v, c))
    }
    fn bellman_ford(&self, s: usize) -> (Vec<i64>, bool) {
        const INF: i64 = std::i64::MAX;
        let mut cost = vec![INF; self.n];
        cost[s] = 0;
        for i in 0..self.n {
            for &(u, v, c) in self.edges.iter() {
                if cost[u] != INF && cost[v] > cost[u] + c {
                    if i == self.n - 1 {
                        return (cost, true);
                    }
                    cost[v] = cost[u] + c;
                }
            }
        }
        (cost, false)
    }
}
