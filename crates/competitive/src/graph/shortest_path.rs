use super::SparseGraph;
use crate::algebra::Monoid;

#[codesnip::entry("dijkstra", include("algebra", "SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn dijkstra<M>(
        &self,
        start: usize,
        monoid: M,
        weight: impl Fn(usize) -> M::T,
    ) -> Vec<Option<M::T>>
    where
        M: Monoid,
        M::T: Ord,
    {
        use std::cmp::Reverse;
        let mut cost = vec![None; self.vertices_size()];
        let mut heap = std::collections::BinaryHeap::new();
        cost[start] = Some(monoid.unit());
        heap.push((Reverse(monoid.unit()), start));
        while let Some((Reverse(d), u)) = heap.pop() {
            if cost[u].as_ref().unwrap() < &d {
                continue;
            }
            for a in self.adjacencies(u) {
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

#[codesnip::entry("bellman_ford", include("algebra", "SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn bellman_ford<M>(
        &self,
        start: usize,
        monoid: M,
        weight: impl Fn(usize) -> M::T,
    ) -> (Vec<Option<M::T>>, bool)
    where
        M: Monoid,
        M::T: Ord,
    {
        let mut cost = vec![None; self.vertices_size()];
        cost[start] = Some(monoid.unit());
        for i in 0..self.vertices_size() {
            for u in self.vertices() {
                if let Some(d) = cost[u].as_ref() {
                    let d = d.clone();
                    for a in self.adjacencies(u) {
                        let nd = monoid.operate(&d, &weight(a.id));
                        if cost[a.to].as_ref().map_or(true, |c| c > &nd) {
                            if i + 1 == self.vertices_size() {
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

#[codesnip::entry("warshall_floyd", include("algebra", "SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn warshall_floyd<M>(
        &self,
        monoid: M,
        weight: impl Fn(usize) -> M::T,
    ) -> Vec<Vec<Option<M::T>>>
    where
        M: Monoid,
        M::T: Ord,
    {
        let mut cost = vec![vec![None; self.vertices_size()]; self.vertices_size()];
        for i in self.vertices() {
            cost[i][i] = Some(monoid.unit());
        }
        for u in self.vertices() {
            for a in self.adjacencies(u) {
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
