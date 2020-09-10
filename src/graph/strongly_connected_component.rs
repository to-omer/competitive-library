use super::SparseGraph;

#[cargo_snippet::snippet("StronglyConnectedComponent")]
#[derive(Debug, Clone)]
pub struct StronglyConnectedComponent<'a> {
    graph: &'a SparseGraph,
    visited: Vec<usize>,
    csize: usize,
    low: Vec<usize>,
    ord: Vec<usize>,
    comp: Vec<usize>,
}
#[cargo_snippet::snippet("StronglyConnectedComponent")]
impl std::ops::Index<usize> for StronglyConnectedComponent<'_> {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.comp[index]
    }
}
#[cargo_snippet::snippet("StronglyConnectedComponent")]
impl<'a> StronglyConnectedComponent<'a> {
    pub fn new(graph: &'a SparseGraph) -> Self {
        let mut now_ord = 0;
        let mut self_ = Self {
            graph,
            csize: 0,
            visited: Vec::with_capacity(graph.vsize),
            low: vec![0; graph.vsize],
            ord: vec![std::usize::MAX; graph.vsize],
            comp: vec![0; graph.vsize],
        };
        for u in graph.vertices() {
            if self_.ord[u] == std::usize::MAX {
                self_.dfs(u, &mut now_ord);
            }
        }
        for x in self_.comp.iter_mut() {
            *x = self_.csize - 1 - *x;
        }
        self_
    }
}
#[cargo_snippet::snippet("StronglyConnectedComponent")]
impl StronglyConnectedComponent<'_> {
    fn dfs(&mut self, u: usize, now_ord: &mut usize) {
        self.low[u] = *now_ord;
        self.ord[u] = *now_ord;
        *now_ord += 1;
        self.visited.push(u);
        for &to in self.graph.adjacency(u) {
            if self.ord[to] == std::usize::MAX {
                self.dfs(to, now_ord);
                self.low[u] = self.low[u].min(self.low[to]);
            } else {
                self.low[u] = self.low[u].min(self.ord[to]);
            }
        }
        if self.low[u] == self.ord[u] {
            while let Some(v) = self.visited.pop() {
                self.ord[v] = self.graph.vsize;
                self.comp[v] = self.csize;
                if v == u {
                    break;
                }
            }
            self.csize += 1;
        }
    }
    pub fn gen_cgraph(&self) -> SparseGraph {
        let mut used = std::collections::HashSet::new();
        let mut edges = vec![];
        for u in self.graph.vertices() {
            for &to in self.graph.adjacency(u) {
                if self.comp[u] != self.comp[to] {
                    let (x, y) = (self.comp[u], self.comp[to]);
                    if !used.contains(&(x, y)) {
                        used.insert((x, y));
                        edges.push((x, y));
                    }
                }
            }
        }
        SparseGraph::from_edges(self.size(), edges.iter().cloned())
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
        self.graph.vsize != self.csize
    }
    pub fn size(&self) -> usize {
        self.csize
    }
}

#[cargo_snippet::snippet("TwoSatisfiability")]
#[derive(Debug, Clone)]
pub struct TwoSatisfiability {
    vsize: usize,
    edges: Vec<(usize, usize)>,
}
#[cargo_snippet::snippet("TwoSatisfiability")]
impl TwoSatisfiability {
    pub fn new(vsize: usize) -> Self {
        Self {
            vsize,
            edges: Vec::new(),
        }
    }
    pub fn add_inner(&mut self, u: usize, v: usize) {
        self.edges.push((u, v));
        self.edges.push((v ^ 1, u ^ 1));
    }
    pub fn add_or(&mut self, x: usize, y: usize) {
        self.add_inner(x * 2 + 1, y * 2);
    }
    pub fn add_nand(&mut self, x: usize, y: usize) {
        self.add_inner(x * 2, y * 2 + 1);
    }
    pub fn set_true(&mut self, x: usize) {
        self.add_inner(x * 2 + 1, x * 2);
    }
    pub fn set_false(&mut self, x: usize) {
        self.add_inner(x * 2, x * 2 + 1);
    }
    pub fn two_satisfiability(self) -> Option<Vec<bool>> {
        let graph = SparseGraph::from_edges(self.vsize, self.edges.iter().cloned());
        let scc = StronglyConnectedComponent::new(&graph);
        let mut res = vec![false; self.vsize / 2];
        for i in 0..self.vsize / 2 {
            if scc[i * 2] == scc[i * 2 + 1] {
                return None;
            }
            res[i] = scc[i * 2] > scc[i * 2 + 1];
        }
        Some(res)
    }
}
