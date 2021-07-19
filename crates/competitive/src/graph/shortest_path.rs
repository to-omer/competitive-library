use super::SparseGraph;
use crate::algebra::Monoid;

#[codesnip::entry("dijkstra", include("algebra", "SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn dijkstra<M, F>(&self, start: usize, weight: F) -> Vec<Option<M::T>>
    where
        M: Monoid,
        M::T: Ord,
        F: Fn(usize) -> M::T,
    {
        let mut cost = vec![None; self.vertices_size()];
        let mut heap = std::collections::BinaryHeap::new();
        cost[start] = Some(M::unit());
        heap.push((std::cmp::Reverse(M::unit()), start));
        while let Some((std::cmp::Reverse(d), u)) = heap.pop() {
            if cost[u].as_ref().unwrap() < &d {
                continue;
            }
            for a in self.adjacencies(u) {
                let nd = M::operate(&d, &weight(a.id));
                if cost[a.to].as_ref().map_or(true, |c| c > &nd) {
                    cost[a.to] = Some(nd.clone());
                    heap.push((std::cmp::Reverse(nd), a.to));
                }
            }
        }
        cost
    }
}

#[codesnip::entry("bellman_ford", include("algebra", "SparseGraph"))]
impl<D> SparseGraph<D> {
    pub fn bellman_ford<M, F>(&self, start: usize, weight: F) -> (Vec<Option<M::T>>, bool)
    where
        M: Monoid,
        M::T: Ord,
        F: Fn(usize) -> M::T,
    {
        let mut cost = vec![None; self.vertices_size()];
        cost[start] = Some(M::unit());
        for i in 0..self.vertices_size() {
            for u in self.vertices() {
                if let Some(d) = cost[u].as_ref() {
                    let d = d.clone();
                    for a in self.adjacencies(u) {
                        let nd = M::operate(&d, &weight(a.id));
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
    pub fn warshall_floyd<M, F>(&self, weight: F) -> Vec<Vec<Option<M::T>>>
    where
        M: Monoid,
        M::T: Ord,
        F: Fn(usize) -> M::T,
    {
        let mut cost = vec![vec![None; self.vertices_size()]; self.vertices_size()];
        for i in self.vertices() {
            cost[i][i] = Some(M::unit());
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
                            let nd = M::operate(d1, d2);
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
