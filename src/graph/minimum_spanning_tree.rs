use super::{Graph, RevGraph, StronglyConnectedComponent};
use crate::algebra::Group;
use crate::data_structure::UnionFind;

#[cargo_snippet::snippet("minimum_spanning_tree")]
impl Graph {
    pub fn minimum_spanning_tree<T: Ord, F: Fn(&usize) -> T>(&self, weight: F) -> Vec<bool> {
        let mut idx: Vec<_> = (0..self.esize).collect();
        idx.sort_by_key(weight);
        let mut uf = UnionFind::new(self.vsize);
        let cache = self.eid_cache();
        let mut res = vec![false; self.esize];
        for eid in idx.into_iter() {
            let (u, v) = cache.edge(eid);
            res[eid] = uf.unite(u, v);
        }
        res
    }
}

impl RevGraph {
    /// minimum_spanning_arborescence: O(|E||V|)
    pub fn chu_liu_edmond<G: Group>(
        &self,
        root: usize,
        group: G,
        weight: &[G::T],
        init: G::T,
    ) -> Option<G::T>
    where
        G::T: Ord,
    {
        let mut from = vec![0; self.vsize];
        let mut graph = RevGraph::new(self.vsize);
        for u in self.vertices().filter(|&u| u != root) {
            if let Some(a) = self.radjacency(u).into_iter().min_by_key(|a| &weight[a.id]) {
                graph.add_edge(a.to, u);
                from[u] = a.id;
            } else {
                return None;
            }
        }

        let scc = StronglyConnectedComponent::new(&graph);
        let mut acc = init;
        if scc.has_loop() {
            let comp = scc.components();
            let mut ngraph = RevGraph::new(scc.size());
            let mut nweight = vec![];
            for u in self.vertices().filter(|&u| u != root) {
                if comp[scc[u]].len() > 1 {
                    let c = &weight[from[u]];
                    acc = group.operate(&acc, c);
                    for a in self.radjacency(u).iter().filter(|a| scc[u] != scc[a.to]) {
                        nweight.push(group.operate(&weight[a.id], &group.inverse(&c)));
                        ngraph.add_edge(scc[a.to], scc[u]);
                    }
                } else {
                    for a in self.radjacency(u) {
                        nweight.push(weight[a.id].clone());
                        ngraph.add_edge(scc[a.to], scc[u]);
                    }
                }
            }
            ngraph.chu_liu_edmond(scc[root], group, &nweight, acc)
        } else {
            for u in self.vertices().filter(|&u| u != root) {
                acc = group.operate(&acc, &weight[from[u]]);
            }
            Some(acc)
        }
    }
}

#[cargo_snippet::snippet("minimum_spanning_arborescence")]
impl Graph {
    /// tarjan
    pub fn minimum_spanning_arborescence<G: Group, F: Fn(usize) -> G::T>(
        &self,
        root: usize,
        group: G,
        weight: F,
    ) -> Option<G::T>
    where
        G::T: Ord + std::fmt::Debug,
    {
        use std::{cmp::Reverse, collections::BinaryHeap};
        let mut uf = UnionFind::new(self.vsize);
        let mut from = vec![0; self.vsize];
        let mut cost = vec![group.unit(); self.vsize];
        let mut state = vec![0; self.vsize]; // 0: unprocessed, 1: in process, 2: completed
        state[root] = 2;
        let mut sub = vec![group.unit(); self.vsize];
        let mut out_edges: Vec<BinaryHeap<_>> =
            self.vertices().map(|_| Default::default()).collect();
        for u in self.vertices() {
            for a in self.adjacency(u) {
                out_edges[a.to].push((Reverse(weight(a.id)), u));
            }
        }
        let mut acc = group.unit();
        for mut u in self.vertices() {
            if state[u] != 0 {
                continue;
            }
            let mut path = vec![];
            while state[u] != 2 {
                path.push(u);
                state[u] = 1;
                if let Some((Reverse(w), v)) = out_edges[u].pop() {
                    let v = uf.find(v);
                    if u == v {
                        continue;
                    }
                    from[u] = v;
                    cost[u] = group.operate(&w, &sub[u]);
                    acc = group.operate(&acc, &cost[u]);
                    if state[v] == 1 {
                        let mut t = u;
                        loop {
                            if !out_edges[t].is_empty() {
                                sub[t] = group.operate(&sub[t], &group.inverse(&cost[t]));
                            }
                            if u != t {
                                if out_edges[u].len() < out_edges[t].len() {
                                    out_edges.swap(u, t);
                                    sub.swap(u, t);
                                }
                                let y = group.operate(&sub[t], &group.inverse(&sub[u]));
                                sub[t] = group.unit();
                                unsafe {
                                    let uedges = out_edges.as_mut_ptr().add(u);
                                    let tedges = out_edges.as_mut_ptr().add(t);
                                    (&mut *uedges).extend((&mut *tedges).drain().map(
                                        |(Reverse(ref w), z)| (Reverse(group.operate(w, &y)), z),
                                    ))
                                }
                                uf.unite_light(u, t);
                            }
                            t = uf.find(from[t]);
                            if u == t {
                                break;
                            }
                        }
                    } else {
                        u = v;
                    }
                } else {
                    return None;
                }
            }
            for u in path.into_iter() {
                state[u] = 2;
            }
        }
        Some(acc)
    }
}
