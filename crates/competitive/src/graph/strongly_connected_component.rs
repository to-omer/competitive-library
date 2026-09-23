use super::{DirectedSparseGraph, Graph};

#[derive(Debug, Clone)]
pub struct StronglyConnectedComponent<'a> {
    graph: &'a DirectedSparseGraph,
    csize: usize,
    comp: Vec<usize>,
}
impl std::ops::Index<usize> for StronglyConnectedComponent<'_> {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.comp[index]
    }
}
impl<'a> StronglyConnectedComponent<'a> {
    pub fn new(graph: &'a DirectedSparseGraph) -> Self {
        let mut now_ord = 0;
        let mut visited = Vec::with_capacity(graph.vertices_size());
        let mut ord = vec![usize::MAX; graph.vertices_size()];
        let mut stack = Vec::new();
        let mut self_ = Self {
            graph,
            csize: 0,
            comp: vec![0; graph.vertices_size()],
        };
        for root in graph.vertices() {
            if ord[root] != usize::MAX {
                continue;
            }
            ord[root] = now_ord;
            now_ord += 1;
            visited.push(root);
            stack.push((root, ord[root], graph.neighbors(root)));
            while let Some((u, low, neighbors)) = stack.last_mut() {
                let u = *u;
                if let Some(a) = neighbors.next() {
                    if ord[a.to] == usize::MAX {
                        ord[a.to] = now_ord;
                        now_ord += 1;
                        visited.push(a.to);
                        stack.push((a.to, ord[a.to], graph.neighbors(a.to)));
                    } else {
                        *low = (*low).min(ord[a.to]);
                    }
                } else {
                    let low = *low;
                    stack.pop();
                    if low == ord[u] {
                        while let Some(v) = visited.pop() {
                            ord[v] = graph.vertices_size();
                            self_.comp[v] = self_.csize;
                            if v == u {
                                break;
                            }
                        }
                        self_.csize += 1;
                    }
                    if let Some((_, parent_low, _)) = stack.last_mut() {
                        *parent_low = (*parent_low).min(low);
                    }
                }
            }
        }
        for x in self_.comp.iter_mut() {
            *x = self_.csize - 1 - *x;
        }
        self_
    }
}
impl StronglyConnectedComponent<'_> {
    pub fn gen_cgraph(&self) -> DirectedSparseGraph {
        let mut used = std::collections::HashSet::new();
        let mut edges = vec![];
        for u in self.graph.vertices() {
            for a in self.graph.neighbors(u) {
                if self.comp[u] != self.comp[a.to] {
                    let (x, y) = (self.comp[u], self.comp[a.to]);
                    if !used.contains(&(x, y)) {
                        used.insert((x, y));
                        edges.push((x, y));
                    }
                }
            }
        }
        DirectedSparseGraph::from_edges(self.size(), edges)
    }
    pub fn components(&self) -> Vec<Vec<usize>> {
        let mut counts = vec![0; self.size()];
        for &x in self.comp.iter() {
            counts[x] += 1;
        }
        let mut groups: Vec<_> = counts.into_iter().map(Vec::with_capacity).collect();
        for u in self.graph.vertices() {
            groups[self[u]].push(u);
        }
        groups
    }
    pub fn has_loop(&self) -> bool {
        self.graph.vertices_size() != self.csize
    }
    pub fn size(&self) -> usize {
        self.csize
    }
}
