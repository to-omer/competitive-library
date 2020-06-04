use super::graph::Graph;
use crate::algebra::magma::Monoid;

#[cargo_snippet::snippet("dijkstra")]
impl Graph {
    pub fn dijkstra<M: Monoid, F: Fn(usize) -> M::T>(
        &self,
        start: usize,
        monoid: M,
        weight: F,
    ) -> Vec<Option<M::T>>
    where
        M::T: Ord,
    {
        use std::cmp::Reverse;
        let mut cost = vec![None; self.vsize];
        let mut heap = std::collections::BinaryHeap::new();
        cost[start] = Some(monoid.unit());
        heap.push((Reverse(monoid.unit()), start));
        while let Some((Reverse(d), u)) = heap.pop() {
            if cost[u].as_ref().unwrap() < &d {
                continue;
            }
            for a in self.adjacency(u) {
                let nd = monoid.operate(&d, &weight(a.id));
                if cost[a.to].as_ref().map_or(true, |c| c > &nd) {
                    cost[a.to] = Some(nd.clone());
                    heap.push((Reverse(nd), a.to));
                }
            }
        }
        cost
    }
}

#[cargo_snippet::snippet("bellman_ford")]
impl Graph {
    pub fn bellman_ford<M: Monoid, F: Fn(usize) -> M::T>(
        &self,
        start: usize,
        monoid: M,
        weight: F,
    ) -> (Vec<Option<M::T>>, bool)
    where
        M::T: Ord,
    {
        let mut cost = vec![None; self.vsize];
        cost[start] = Some(monoid.unit().clone());
        for i in 0..self.vsize {
            for u in self.vertices() {
                if let Some(d) = cost[u].as_ref() {
                    let d = d.clone();
                    for a in self.adjacency(u) {
                        let nd = monoid.operate(&d, &weight(a.id));
                        if cost[a.to].as_ref().map_or(true, |c| c > &nd) {
                            if i + 1 == self.vsize {
                                return (cost, true);
                            }
                            cost[a.to] = Some(nd);
                        }
                    }
                }
            }
        }
        (cost, false)
    }
}

#[cargo_snippet::snippet("warshall_floyd")]
impl Graph {
    pub fn warshall_floyd<M: Monoid, F: Fn(usize) -> M::T>(
        &self,
        monoid: M,
        weight: F,
    ) -> Vec<Vec<Option<M::T>>>
    where
        M::T: Ord,
    {
        let mut cost = vec![vec![None; self.vsize]; self.vsize];
        for i in self.vertices() {
            cost[i][i] = Some(monoid.unit());
        }
        for u in self.vertices() {
            for a in self.adjacency(u) {
                cost[u][a.to] = Some(weight(a.id));
            }
        }
        for k in self.vertices() {
            for i in self.vertices() {
                for j in self.vertices() {
                    if let Some(d1) = &cost[i][k] {
                        if let Some(d2) = &cost[k][j] {
                            let nd = monoid.operate(d1, d2);
                            if cost[i][j].as_ref().map_or(true, |c| c > &nd) {
                                cost[i][j] = Some(nd);
                            }
                        }
                    }
                }
            }
        }
        cost
    }
}
