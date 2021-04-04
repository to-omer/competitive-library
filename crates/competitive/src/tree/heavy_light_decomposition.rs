use crate::algebra::Monoid;
use crate::graph::UndirectedSparseGraph;

#[codesnip::entry("HeavyLightDecomposition", include("algebra", "SparseGraph"))]
pub struct HeavyLightDecomposition {
    pub par: Vec<usize>,
    size: Vec<usize>,
    head: Vec<usize>,
    pub vidx: Vec<usize>,
}
#[codesnip::entry("HeavyLightDecomposition")]
impl HeavyLightDecomposition {
    pub fn new(root: usize, graph: &mut UndirectedSparseGraph) -> Self {
        let mut self_ = Self {
            par: vec![0; graph.vertices_size()],
            size: vec![0; graph.vertices_size()],
            head: vec![0; graph.vertices_size()],
            vidx: vec![0; graph.vertices_size()],
        };
        self_.build(root, graph);
        self_
    }
    fn dfs_size(&mut self, u: usize, p: usize, graph: &mut UndirectedSparseGraph) {
        self.par[u] = p;
        self.size[u] = 1;
        let base = graph.start[u];
        if graph.adjacencies(u).len() > 1 && graph.adjacencies(u).next().unwrap().to == p {
            graph.elist.swap(base, base + 1);
        }
        for i in base..graph.start[u + 1] {
            let a = graph.elist[i];
            if a.to != p {
                self.dfs_size(a.to, u, graph);
                self.size[u] += self.size[a.to];
                if self.size[graph.elist[base].to] < self.size[a.to] {
                    graph.elist.swap(base, i);
                }
            }
        }
    }
    fn dfs_hld(&mut self, u: usize, p: usize, t: &mut usize, graph: &UndirectedSparseGraph) {
        self.vidx[u] = *t;
        *t += 1;
        let mut adjacencies = graph.adjacencies(u).filter(|a| a.to != p);
        if let Some(a) = adjacencies.next() {
            self.head[a.to] = self.head[u];
            self.dfs_hld(a.to, u, t, graph);
        }
        for a in adjacencies {
            self.head[a.to] = a.to;
            self.dfs_hld(a.to, u, t, graph);
        }
    }
    fn build(&mut self, root: usize, graph: &mut UndirectedSparseGraph) {
        self.head[root] = root;
        self.dfs_size(root, graph.vertices_size(), graph);
        let mut t = 0;
        self.dfs_hld(root, graph.vertices_size(), &mut t, graph);
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
    ) -> M::T {
        let (mut l, mut r) = (M::unit(), M::unit());
        loop {
            if self.vidx[u] > self.vidx[v] {
                std::mem::swap(&mut u, &mut v);
                std::mem::swap(&mut l, &mut r);
            }
            if self.head[u] == self.head[v] {
                break;
            }
            l = M::operate(&f(self.vidx[self.head[v]], self.vidx[v] + 1), &l);
            v = self.par[self.head[v]];
        }
        M::operate(
            &M::operate(&f(self.vidx[u] + is_edge as usize, self.vidx[v] + 1), &l),
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
    ) -> M::T {
        let (mut l, mut r) = (M::unit(), M::unit());
        while self.head[u] != self.head[v] {
            if self.vidx[u] > self.vidx[v] {
                l = M::operate(&l, &f2(self.vidx[self.head[u]], self.vidx[u] + 1));
                u = self.par[self.head[u]];
            } else {
                r = M::operate(&f1(self.vidx[self.head[v]], self.vidx[v] + 1), &r);
                v = self.par[self.head[v]];
            }
        }
        M::operate(
            &M::operate(
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
