use crate::algebra::Monoid;
use crate::graph::Graph;

#[cargo_snippet::snippet("HeavyLightDecomposition")]
pub struct HeavyLightDecomposition {
    pub par: Vec<usize>,
    size: Vec<usize>,
    head: Vec<usize>,
    pub vidx: Vec<usize>,
}
#[cargo_snippet::snippet("HeavyLightDecomposition")]
impl HeavyLightDecomposition {
    pub fn new(root: usize, graph: &mut Graph) -> Self {
        let vsize = graph.vsize;
        let mut self_ = Self {
            par: vec![0; vsize],
            size: vec![0; vsize],
            head: vec![0; vsize],
            vidx: vec![0; vsize],
        };
        self_.build(root, graph);
        self_
    }
    fn dfs_size(&mut self, u: usize, p: usize, graph: &mut Graph) {
        self.par[u] = p;
        self.size[u] = 1;
        if graph.graph[u].len() > 1 && graph.graph[u][0].to == p {
            graph.graph[u].swap(0, 1);
        }
        for i in 0..graph.graph[u].len() {
            let a = graph.graph[u][i];
            if a.to != p {
                self.dfs_size(a.to, u, graph);
                self.size[u] += self.size[a.to];
                if self.size[graph.graph[u][0].to] < self.size[a.to] {
                    graph.graph[u].swap(0, i);
                }
            }
        }
    }
    fn dfs_hld(&mut self, u: usize, p: usize, t: &mut usize, graph: &Graph) {
        self.vidx[u] = *t;
        *t += 1;
        for i in 0..graph.graph[u].len() {
            let a = graph.graph[u][i];
            if a.to != p {
                self.head[a.to] = if i == 0 { self.head[u] } else { a.to };
                self.dfs_hld(a.to, u, t, graph);
            }
        }
    }
    fn build(&mut self, root: usize, graph: &mut Graph) {
        self.head[root] = root;
        self.dfs_size(root, graph.vsize, graph);
        let mut t = 0;
        self.dfs_hld(root, graph.vsize, &mut t, graph);
    }
    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        loop {
            if self.vidx[u] > self.vidx[v] {
                std::mem::swap(&mut u, &mut v);
            }
            if self.head[u] == self.head[v] {
                return u;
            }
            v = self.par[self.head[v]];
        }
    }
    pub fn update<F: FnMut(usize, usize)>(
        &self,
        mut u: usize,
        mut v: usize,
        is_edge: bool,
        mut f: F,
    ) {
        loop {
            if self.vidx[u] > self.vidx[v] {
                std::mem::swap(&mut u, &mut v);
            }
            if self.head[u] == self.head[v] {
                break;
            }
            f(self.vidx[self.head[v]], self.vidx[v] + 1);
            v = self.par[self.head[v]];
        }
        f(self.vidx[u] + is_edge as usize, self.vidx[v] + 1);
    }
    pub fn query<M: Monoid, F: FnMut(usize, usize) -> M::T>(
        &self,
        mut u: usize,
        mut v: usize,
        is_edge: bool,
        mut f: F,
        monoid: &M,
    ) -> M::T {
        let (mut l, mut r) = (monoid.unit(), monoid.unit());
        loop {
            if self.vidx[u] > self.vidx[v] {
                std::mem::swap(&mut u, &mut v);
                std::mem::swap(&mut l, &mut r);
            }
            if self.head[u] == self.head[v] {
                break;
            }
            l = monoid.operate(&f(self.vidx[self.head[v]], self.vidx[v] + 1), &l);
            v = self.par[self.head[v]];
        }
        monoid.operate(
            &monoid.operate(&f(self.vidx[u] + is_edge as usize, self.vidx[v] + 1), &l),
            &r,
        )
    }
    pub fn query_noncom<
        M: Monoid,
        F1: FnMut(usize, usize) -> M::T,
        F2: FnMut(usize, usize) -> M::T,
    >(
        &self,
        mut u: usize,
        mut v: usize,
        is_edge: bool,
        mut f1: F1,
        mut f2: F2,
        monoid: &M,
    ) -> M::T {
        let (mut l, mut r) = (monoid.unit(), monoid.unit());
        while self.head[u] != self.head[v] {
            if self.vidx[u] > self.vidx[v] {
                l = monoid.operate(&l, &f2(self.vidx[self.head[u]], self.vidx[u] + 1));
                u = self.par[self.head[u]];
            } else {
                r = monoid.operate(&f1(self.vidx[self.head[v]], self.vidx[v] + 1), &r);
                v = self.par[self.head[v]];
            }
        }
        monoid.operate(
            &monoid.operate(
                &l,
                &if self.vidx[u] > self.vidx[v] {
                    f2(self.vidx[v] + is_edge as usize, self.vidx[u] + 1)
                } else {
                    f1(self.vidx[u] + is_edge as usize, self.vidx[v] + 1)
                },
            ),
            &r,
        )
    }
}
