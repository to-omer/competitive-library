use super::RevGraph;

#[cargo_snippet::snippet("StronglyConnectedComponent")]
#[derive(Debug)]
pub struct StronglyConnectedComponent {
    vsize: usize,
    ord: Vec<usize>,
    used: Vec<bool>,
    comp: Vec<usize>,
    csize: usize,
}
#[cargo_snippet::snippet("StronglyConnectedComponent")]
impl StronglyConnectedComponent {
    pub fn new(graph: &RevGraph) -> Self {
        let mut self_ = Self {
            vsize: graph.vsize,
            ord: vec![],
            used: vec![false; graph.vsize],
            comp: vec![0; graph.vsize],
            csize: 0,
        };
        self_.build(graph);
        self_
    }
    fn dfs(&mut self, u: usize, graph: &RevGraph) {
        self.used[u] = true;
        for a in graph.adjacency(u) {
            if !self.used[a.to] {
                self.dfs(a.to, graph);
            }
        }
        self.ord.push(u);
    }
    fn rdfs(&mut self, u: usize, k: usize, graph: &RevGraph) {
        self.used[u] = true;
        self.comp[u] = k;
        for a in graph.radjacency(u) {
            if !self.used[a.to] {
                self.rdfs(a.to, k, graph);
            }
        }
    }
    fn build(&mut self, graph: &RevGraph) {
        for u in graph.vertices() {
            if !self.used[u] {
                self.dfs(u, graph);
            }
        }
        self.used = vec![false; self.vsize];
        for i in graph.vertices().rev() {
            if !self.used[self.ord[i]] {
                let (v, k) = (self.ord[i], self.csize);
                self.rdfs(v, k, graph);
                self.csize += 1;
            }
        }
    }
    pub fn gen_cgraph(&self, graph: &RevGraph) -> RevGraph {
        let mut g = RevGraph::new(self.csize);
        let mut used = std::collections::HashSet::new();
        for u in graph.vertices() {
            for a in graph.adjacency(u) {
                if self.comp[u] != self.comp[a.to] {
                    let (x, y) = (self.comp[u], self.comp[a.to]);
                    if !used.contains(&(x, y)) {
                        used.insert((x, y));
                        g.add_edge(x, y);
                    }
                }
            }
        }
        g
    }
    pub fn components(&self) -> Vec<Vec<usize>> {
        let mut c = vec![vec![]; self.csize];
        for u in 0..self.vsize {
            c[self.comp[u]].push(u);
        }
        c
    }
    pub fn has_loop(&self) -> bool {
        self.vsize != self.csize
    }
    pub fn size(&self) -> usize {
        self.csize
    }
}
#[cargo_snippet::snippet("StronglyConnectedComponent")]
impl std::ops::Index<usize> for StronglyConnectedComponent {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.comp[index]
    }
}

#[cargo_snippet::snippet("TwoSatisfiability")]
#[cargo_snippet::snippet(include = "StronglyConnectedComponent")]
#[derive(Debug)]
pub struct TwoSatisfiability {
    n: usize,
    scc: StronglyConnectedComponent,
}
#[cargo_snippet::snippet("TwoSatisfiability")]
impl TwoSatisfiability {
    pub fn add_inner(graph: &mut RevGraph, u: usize, v: usize) {
        graph.add_edge(u, v);
        graph.add_edge(v ^ 1, u ^ 1);
    }
    pub fn add_or(graph: &mut RevGraph, x: usize, y: usize) {
        Self::add_inner(graph, x * 2 + 1, y * 2)
    }
    pub fn add_nand(graph: &mut RevGraph, x: usize, y: usize) {
        Self::add_inner(graph, x * 2, y * 2 + 1)
    }
    pub fn set_true(graph: &mut RevGraph, x: usize) {
        Self::add_inner(graph, x * 2 + 1, x * 2)
    }
    pub fn set_false(graph: &mut RevGraph, x: usize) {
        Self::add_inner(graph, x * 2, x * 2 + 1)
    }
    pub fn build(n: usize, graph: &RevGraph) -> Option<Vec<bool>> {
        let scc = StronglyConnectedComponent::new(graph);
        let mut res = vec![false; n];
        for i in 0..n {
            if scc[i * 2] == scc[i * 2 + 1] {
                return None;
            }
            res[i] = scc[i * 2] > scc[i * 2 + 1];
        }
        Some(res)
    }
}
