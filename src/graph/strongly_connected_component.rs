// use super::*;

#[cargo_snippet::snippet("StronglyConnectedComponent")]
#[derive(Debug)]
pub struct StronglyConnectedComponent {
    vsize: usize,
    graph: Vec<Vec<usize>>,
    rgraph: Vec<Vec<usize>>,
    ord: Vec<usize>,
    used: Vec<bool>,
    comp: Vec<usize>,
    csize: usize,
}
#[cargo_snippet::snippet("StronglyConnectedComponent")]
impl StronglyConnectedComponent {
    pub fn new(vsize: usize) -> Self {
        StronglyConnectedComponent {
            vsize: vsize,
            graph: vec![vec![]; vsize],
            rgraph: vec![vec![]; vsize],
            ord: vec![],
            used: vec![],
            comp: vec![0; vsize],
            csize: 0,
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.graph[from].push(to);
        self.rgraph[to].push(from);
    }
    pub fn dfs(&mut self, u: usize) {
        self.used[u] = true;
        for i in 0..self.graph[u].len() {
            let v = self.graph[u][i];
            if !self.used[v] {
                self.dfs(v);
            }
        }
        self.ord.push(u);
    }
    pub fn rdfs(&mut self, u: usize, k: usize) {
        self.used[u] = true;
        self.comp[u] = k;
        for i in 0..self.rgraph[u].len() {
            let v = self.rgraph[u][i];
            if !self.used[v] {
                self.rdfs(v, k);
            }
        }
    }
    pub fn build(&mut self) {
        self.used = vec![false; self.vsize];
        self.ord.clear();
        for u in 0..self.vsize {
            if !self.used[u] {
                self.dfs(u);
            }
        }
        self.used = vec![false; self.vsize];
        self.csize = 0;
        for i in (0..self.vsize).rev() {
            if !self.used[self.ord[i]] {
                let (v, k) = (self.ord[i], self.csize);
                self.rdfs(v, k);
                self.csize += 1;
            }
        }
    }
    pub fn gen_cgraph(&self) -> Vec<Vec<usize>> {
        let mut g = vec![vec![]; self.csize];
        for u in 0..self.vsize {
            for &v in self.graph[u].iter() {
                if self.comp[u] != self.comp[v] {
                    g[self.comp[u]].push(self.comp[v]);
                }
            }
        }
        g.into_iter()
            .map(|v| {
                v.into_iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
    pub fn component(&self) -> Vec<std::collections::HashSet<usize>> {
        let mut c = (0..self.csize)
            .map(|_| std::collections::HashSet::new())
            .collect::<Vec<_>>();
        for u in 0..self.vsize {
            c[self.comp[u]].insert(u);
        }
        c
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
    pub fn new(n: usize) -> Self {
        TwoSatisfiability {
            n: n,
            scc: StronglyConnectedComponent::new(n * 2),
        }
    }
    pub fn add_inner(&mut self, u: usize, v: usize) {
        self.scc.add_edge(u, v);
        self.scc.add_edge(v ^ 1, u ^ 1);
    }
    pub fn add_or(&mut self, x: usize, y: usize) {
        self.add_inner(x * 2 + 1, y * 2)
    }
    pub fn add_nand(&mut self, x: usize, y: usize) {
        self.add_inner(x * 2, y * 2 + 1)
    }
    pub fn set_true(&mut self, x: usize) {
        self.add_inner(x * 2 + 1, x * 2)
    }
    pub fn set_false(&mut self, x: usize) {
        self.add_inner(x * 2, x * 2 + 1)
    }
    pub fn build(&mut self) -> Option<Vec<bool>> {
        self.scc.build();
        let mut res = vec![false; self.n];
        for i in 0..self.n {
            if self.scc[i * 2] == self.scc[i * 2 + 1] {
                return None;
            }
            res[i] = self.scc[i * 2] > self.scc[i * 2 + 1];
        }
        Some(res)
    }
}
