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

#[cargo_snippet::snippet("minimum_spanning_arborescence")]
impl RevGraph {
    pub fn minimum_spanning_arborescence<G: Group>(
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
            ngraph.minimum_spanning_arborescence(scc[root], group, &nweight, acc)
        } else {
            for u in self.vertices().filter(|&u| u != root) {
                acc = group.operate(&acc, &weight[from[u]]);
            }
            Some(acc)
        }
    }
}
