use crate::algebra::{Associative, Magma};
use crate::data_structure::DisjointSparseTable;
use crate::graph::UndirectedSparseGraph;

#[cfg_attr(nightly, codesnip::entry("EulerTourForEdge", include("SparseGraph")))]
#[derive(Clone, Debug)]
pub struct EulerTourForEdge<'a> {
    graph: &'a UndirectedSparseGraph,
    pub eidx: Vec<(usize, usize)>,
    pub par: Vec<usize>,
    epos: usize,
}
#[cfg_attr(nightly, codesnip::entry("EulerTourForEdge"))]
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

#[cfg_attr(nightly, codesnip::entry("EulerTourForVertex", include("SparseGraph")))]
#[derive(Clone, Debug)]
pub struct EulerTourForVertex<'a> {
    graph: &'a UndirectedSparseGraph,
    pub vidx: Vec<(usize, usize)>,
    vpos: usize,
}
#[cfg_attr(nightly, codesnip::entry("EulerTourForVertex"))]
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

#[cfg_attr(
    nightly,
    codesnip::entry("EulerTourForRichVertex", include("SparseGraph"))
)]
#[derive(Clone, Debug)]
pub struct EulerTourForRichVertex<'a> {
    graph: &'a UndirectedSparseGraph,
    root: usize,
    vidx: Vec<(usize, usize)>,
    vtrace: Vec<usize>,
}
#[cfg_attr(nightly, codesnip::entry("EulerTourForRichVertex"))]
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

#[cfg_attr(nightly, codesnip::entry("LowestCommonAncestor"))]
impl<'a> EulerTourForRichVertex<'a> {
    pub fn gen_lca<D: LcaMonoidDispatch>(&'a self) -> LowestCommonAncestor<'a, D> {
        D::set_depth(self.graph.tree_depth(self.root));
        let dst = DisjointSparseTable::<LcaMonoid<D>>::new(self.vtrace.clone());
        LowestCommonAncestor { euler: self, dst }
    }
}
#[cfg_attr(
    nightly,
    codesnip::entry(
        "LowestCommonAncestor",
        include(
            "algebra",
            "DisjointSparseTable",
            "EulerTourForRichVertex",
            "SparseGraph",
            "tree_depth"
        )
    )
)]
#[derive(Clone, Debug)]
pub struct LowestCommonAncestor<'a, D: LcaMonoidDispatch> {
    euler: &'a EulerTourForRichVertex<'a>,
    dst: DisjointSparseTable<LcaMonoid<D>>,
}
#[cfg_attr(nightly, codesnip::entry("LowestCommonAncestor"))]
impl<'a, D: LcaMonoidDispatch> LowestCommonAncestor<'a, D> {
    pub fn lca(&self, u: usize, v: usize) -> usize {
        self.euler.query(u, v, |l, r| self.dst.fold(l, r))
    }
}
#[cfg_attr(nightly, codesnip::entry("LowestCommonAncestor"))]
pub trait LcaMonoidDispatch {
    fn vsize() -> usize;
    fn depth(u: usize) -> u64;
    fn set_depth(depth: Vec<u64>);
}
#[cfg_attr(nightly, codesnip::entry("LowestCommonAncestor"))]
pub struct LcaMonoidDefaultId;
#[cfg_attr(nightly, codesnip::entry("LowestCommonAncestor"))]
#[derive(Clone, Debug)]
pub struct LcaMonoid<D: LcaMonoidDispatch = LcaMonoidDefaultId> {
    _marker: std::marker::PhantomData<fn() -> D>,
}
#[cfg_attr(nightly, codesnip::entry("LowestCommonAncestor"))]
pub mod impl_lcam {
    use super::*;
    thread_local! {
        static DEPTH: std::cell::UnsafeCell<Vec<u64>> = std::cell::UnsafeCell::new(Vec::new());
    }
    impl LcaMonoidDispatch for LcaMonoidDefaultId {
        fn vsize() -> usize {
            DEPTH.with(|c| unsafe { (&*c.get()).len() })
        }
        fn depth(u: usize) -> u64 {
            DEPTH.with(|c| unsafe { (&*c.get())[u] })
        }
        fn set_depth(depth: Vec<u64>) {
            DEPTH.with(|c| unsafe { *c.get() = depth })
        }
    }
    impl<D: LcaMonoidDispatch> Magma for LcaMonoid<D> {
        type T = usize;
        fn operate(&x: &Self::T, &y: &Self::T) -> Self::T {
            if x >= D::vsize() {
                y
            } else if y >= D::vsize() || D::depth(x) < D::depth(y) {
                x
            } else {
                y
            }
        }
    }
    impl<D: LcaMonoidDispatch> Associative for LcaMonoid<D> {}
}
