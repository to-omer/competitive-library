use super::DirectedSparseGraph;

#[derive(Debug, Clone)]
pub struct StronglyConnectedComponent<'a> {
    graph: &'a DirectedSparseGraph,
    visited: Vec<usize>,
    csize: usize,
    low: Vec<usize>,
    ord: Vec<usize>,
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
        let mut self_ = Self {
            graph,
            csize: 0,
            visited: Vec::with_capacity(graph.vertices_size()),
            low: vec![0; graph.vertices_size()],
            ord: vec![usize::MAX; graph.vertices_size()],
            comp: vec![0; graph.vertices_size()],
        };
        for u in graph.vertices() {
            if self_.ord[u] == usize::MAX {
                self_.dfs(u, &mut now_ord);
            }
        }
        for x in self_.comp.iter_mut() {
            *x = self_.csize - 1 - *x;
        }
        self_
    }
}
impl StronglyConnectedComponent<'_> {
    fn dfs(&mut self, u: usize, now_ord: &mut usize) {
        self.low[u] = *now_ord;
        self.ord[u] = *now_ord;
        *now_ord += 1;
        self.visited.push(u);
        for a in self.graph.adjacencies(u) {
            if self.ord[a.to] == usize::MAX {
                self.dfs(a.to, now_ord);
                self.low[u] = self.low[u].min(self.low[a.to]);
            } else {
                self.low[u] = self.low[u].min(self.ord[a.to]);
            }
        }
        if self.low[u] == self.ord[u] {
            while let Some(v) = self.visited.pop() {
                self.ord[v] = self.graph.vertices_size();
                self.comp[v] = self.csize;
                if v == u {
                    break;
                }
            }
            self.csize += 1;
        }
    }
    pub fn gen_cgraph(&self) -> DirectedSparseGraph {
        let mut used = std::collections::HashSet::new();
        let mut edges = vec![];
        for u in self.graph.vertices() {
            for a in self.graph.adjacencies(u) {
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
        let mut groups = vec![vec![]; self.size()];
        for (g, c) in groups.iter_mut().zip(counts.into_iter()) {
            g.reserve(c);
        }
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
