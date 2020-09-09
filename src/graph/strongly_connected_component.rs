use super::Graph;

#[cargo_snippet::snippet("StronglyConnectedComponent")]
#[derive(Debug, Clone)]
pub struct StronglyConnectedComponent<'a> {
    graph: &'a Graph,
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
    pub fn new(graph: &'a Graph) -> Self {
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
        for a in self.graph.adjacency(u) {
            if self.ord[a.to] == std::usize::MAX {
                self.dfs(a.to, now_ord);
                self.low[u] = self.low[u].min(self.low[a.to]);
            } else {
                self.low[u] = self.low[u].min(self.ord[a.to]);
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
    pub fn gen_cgraph(&self) -> Graph {
        let mut g = Graph::new(self.csize);
        let mut used = std::collections::HashSet::new();
        for u in self.graph.vertices() {
            for a in self.graph.adjacency(u) {
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
#[cargo_snippet::snippet(include = "StronglyConnectedComponent")]
impl Graph {
    pub fn add_inner(&mut self, u: usize, v: usize) {
        self.add_edge(u, v);
        self.add_edge(v ^ 1, u ^ 1);
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
    pub fn two_satisfiability(&self) -> Option<Vec<bool>> {
        let scc = StronglyConnectedComponent::new(self);
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
