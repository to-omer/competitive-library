use super::{EdgeListGraph, Group, MergingUnionFind, MonoidAct, PairingHeap, comparator::Less};

impl EdgeListGraph {
    /// tarjan
    pub fn minimum_spanning_arborescence<G, F>(
        &self,
        root: usize,
        weight: F,
    ) -> Option<(G::T, Vec<usize>)>
    where
        G: Group<T: Ord>,
        F: Fn(usize) -> G::T,
    {
        struct WeightAct<G>(std::marker::PhantomData<fn() -> G>);
        impl<G> MonoidAct for WeightAct<G>
        where
            G: Group,
        {
            type Key = (G::T, usize);
            type Act = G::T;
            type ActMonoid = G;

            fn act(x: &Self::Key, a: &Self::Act) -> Self::Key {
                (G::operate(&x.0, a), x.1)
            }

            fn act_assign(x: &mut Self::Key, a: &Self::Act) {
                x.0 = G::operate(&x.0, a);
            }
        }
        let mut uf = MergingUnionFind::new_with_merger(
            self.vertices_size(),
            |_| PairingHeap::<(G::T, usize), Less, WeightAct<G>>::default(),
            |x, y| x.append(y),
        );
        let mut state = vec![0; self.vertices_size()]; // 0: unprocessed, 1: in process, 2: completed
        state[root] = 2;
        for (id, &(_, to)) in self.edges().enumerate() {
            uf.merge_data_mut(to).push((weight(id), id));
        }
        let mut paredge = vec![0; self.edges_size()];
        let mut ord = vec![];
        let mut leaf = vec![self.edges_size(); self.vertices_size()];
        let mut cycle = 0usize;
        let mut acc = G::unit();
        for mut cur in self.vertices() {
            if state[cur] != 0 {
                continue;
            }
            let mut path = vec![];
            let mut ch = vec![];
            while state[cur] != 2 {
                path.push(cur);
                state[cur] = 1;
                let (w, eid) = {
                    match uf.merge_data_mut(cur).pop() {
                        Some((w, eid)) => (w, eid),
                        None => return None,
                    }
                };
                uf.merge_data_mut(cur).apply_all(G::inverse(&w));
                acc = G::operate(&acc, &w);
                ord.push(eid);
                let (u, v) = self[eid];
                if leaf[v] >= self.edges_size() {
                    leaf[v] = eid;
                }
                while cycle > 0 {
                    paredge[ch.pop().unwrap()] = eid;
                    cycle -= 1;
                }
                ch.push(eid);
                if state[uf.find_root(u)] == 1 {
                    while let Some(t) = path.pop() {
                        state[t] = 2;
                        cycle += 1;
                        if !uf.unite(u, t) {
                            break;
                        }
                    }
                    state[uf.find_root(u)] = 1;
                }
                cur = uf.find_root(u);
            }
            for u in path.into_iter() {
                state[u] = 2;
            }
        }
        let mut tree = vec![root; self.vertices_size()];
        let mut used = vec![false; self.edges_size()];
        for eid in ord.into_iter().rev() {
            if !used[eid] {
                let (u, v) = self[eid];
                tree[v] = u;
                let mut x = leaf[v];
                while x != eid {
                    used[x] = true;
                    x = paredge[x];
                }
            }
        }
        Some((acc, tree))
    }
}
