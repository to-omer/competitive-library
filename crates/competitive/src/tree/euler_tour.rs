use crate::algebra::{Associative, Magma};
use crate::data_structure::DisjointSparseTable;
use crate::graph::UndirectedSparseGraph;

#[codesnip::entry("EulerTourForEdge")]
#[derive(Clone, Debug)]
pub struct EulerTourForEdge<'a> {
    graph: &'a UndirectedSparseGraph,
    pub eidx: Vec<(usize, usize)>,
    pub par: Vec<usize>,
    epos: usize,
}
#[codesnip::entry("EulerTourForEdge")]
impl<'a> EulerTourForEdge<'a> {
    pub fn new(root: usize, graph: &'a UndirectedSparseGraph) -> Self {
        let mut self_ = Self {
            graph,
            eidx: vec![(0, 0); graph.vertices_size() - 1],
            par: vec![std::usize::MAX; graph.vertices_size()],
            epos: 0,
        };
        self_.edge_tour(root, std::usize::MAX);
        self_
    }
    pub fn length(&self) -> usize {
        self.epos
    }
    fn edge_tour(&mut self, u: usize, p: usize) {
        for a in self.graph.adjacencies(u).filter(|a| a.to != p) {
            self.par[a.to] = a.id;
            self.eidx[a.id].0 = self.epos;
            self.epos += 1;
            self.edge_tour(a.to, u);
            self.eidx[a.id].1 = self.epos;
            self.epos += 1;
        }
    }
}

#[codesnip::entry("EulerTourForVertex")]
#[derive(Clone, Debug)]
pub struct EulerTourForVertex<'a> {
    graph: &'a UndirectedSparseGraph,
    pub vidx: Vec<(usize, usize)>,
    vpos: usize,
}
#[codesnip::entry("EulerTourForVertex")]
impl<'a> EulerTourForVertex<'a> {
    pub fn new(graph: &'a UndirectedSparseGraph) -> Self {
        Self {
            graph,
            vidx: vec![(0, 0); graph.vertices_size()],
            vpos: 0,
        }
    }
    pub fn length(&self) -> usize {
        self.vpos
    }
    pub fn subtree_vertex_tour(&mut self, u: usize, p: usize) {
        self.vidx[u].0 = self.vpos;
        self.vpos += 1;
        for a in self.graph.adjacencies(u).filter(|a| a.to != p) {
            self.subtree_vertex_tour(a.to, u);
        }
        self.vidx[u].1 = self.vpos;
    }
    pub fn path_vertex_tour(&mut self, u: usize, p: usize) {
        self.vidx[u].0 = self.vpos;
        self.vpos += 1;
        for a in self.graph.adjacencies(u).filter(|a| a.to != p) {
            self.path_vertex_tour(a.to, u);
        }
        self.vidx[u].1 = self.vpos;
        self.vpos += 1;
    }
    pub fn subtree_query<T, F: FnMut(usize, usize) -> T>(&self, u: usize, mut f: F) -> T {
        let (l, r) = self.vidx[u];
        f(l, r)
    }
    pub fn subtree_update<T, F: FnMut(usize, T)>(&self, u: usize, x: T, mut f: F) {
        let (l, _r) = self.vidx[u];
        f(l, x);
    }
    pub fn path_query<T, F: FnMut(usize, usize) -> T>(&self, u: usize, v: usize, mut f: F) -> T {
        let (mut l, mut r) = (self.vidx[u].0, self.vidx[v].0);
        if l > r {
            std::mem::swap(&mut l, &mut r);
        }
        f(l, r + 1)
    }
    pub fn path_update<T, F: FnMut(usize, T)>(&self, u: usize, x: T, invx: T, mut f: F) {
        let (l, r) = self.vidx[u];
        f(l, x);
        f(r, invx);
    }
}

#[codesnip::entry("EulerTourForRichVertex")]
#[derive(Clone, Debug)]
pub struct EulerTourForRichVertex<'a> {
    graph: &'a UndirectedSparseGraph,
    root: usize,
    vidx: Vec<(usize, usize)>,
    vtrace: Vec<usize>,
}
#[codesnip::entry("EulerTourForRichVertex")]
impl<'a> EulerTourForRichVertex<'a> {
    pub fn new(root: usize, graph: &'a UndirectedSparseGraph) -> Self {
        let mut self_ = Self {
            graph,
            root,
            vidx: vec![(0, 0); graph.vertices_size()],
            vtrace: vec![],
        };
        self_.vertex_tour(root, std::usize::MAX);
        self_
    }
    pub fn length(&self) -> usize {
        self.vtrace.len()
    }
    fn vertex_tour(&mut self, u: usize, p: usize) {
        self.vidx[u].0 = self.vtrace.len();
        self.vtrace.push(u);
        for a in self.graph.adjacencies(u).filter(|a| a.to != p) {
            self.vertex_tour(a.to, u);
            self.vtrace.push(u);
        }
        self.vidx[u].1 = self.vtrace.len() - 1;
    }
    pub fn query<T, F: FnMut(usize, usize) -> T>(&self, u: usize, v: usize, mut f: F) -> T {
        let (mut l, mut r) = (self.vidx[u].0, self.vidx[v].0);
        if l > r {
            std::mem::swap(&mut l, &mut r);
        }
        f(l, r + 1)
    }
}

#[codesnip::entry("LowestCommonAncestor")]
impl<'a> EulerTourForRichVertex<'a> {
    pub fn gen_lca(&'a self) -> LowestCommonAncestor<'a> {
        let monoid = LCAMonoid::new(self.root, self.graph);
        let dst = DisjointSparseTable::new(self.vtrace.clone(), monoid);
        LowestCommonAncestor { euler: self, dst }
    }
}
#[codesnip::entry("LowestCommonAncestor")]
#[derive(Clone, Debug)]
pub struct LowestCommonAncestor<'a> {
    euler: &'a EulerTourForRichVertex<'a>,
    dst: DisjointSparseTable<LCAMonoid>,
}
#[codesnip::entry("LowestCommonAncestor")]
impl<'a> LowestCommonAncestor<'a> {
    pub fn lca(&self, u: usize, v: usize) -> usize {
        self.euler.query(u, v, |l, r| self.dst.fold(l, r))
    }
}
#[codesnip::entry("LowestCommonAncestor")]
#[derive(Clone, Debug)]
pub struct LCAMonoid {
    depth: Vec<u64>,
}
#[codesnip::entry("LowestCommonAncestor")]
pub mod impl_lcam {
    use super::*;
    impl LCAMonoid {
        pub fn new(root: usize, graph: &UndirectedSparseGraph) -> Self {
            LCAMonoid {
                depth: graph.tree_depth(root),
            }
        }
        pub fn ancestor(&self, u: usize, v: usize) -> usize {
            if u >= self.depth.len() {
                v
            } else if v >= self.depth.len() || self.depth[u] < self.depth[v] {
                u
            } else {
                v
            }
        }
    }
    impl Magma for LCAMonoid {
        type T = usize;
        fn operate(&self, x: &Self::T, y: &Self::T) -> Self::T {
            self.ancestor(*x, *y)
        }
    }
    impl Associative for LCAMonoid {}
}
