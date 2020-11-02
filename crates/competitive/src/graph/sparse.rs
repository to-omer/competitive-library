use crate::tools::{IterScan, MarkedIterScan};

#[codesnip::entry("SparseGraph", include("scanner"))]
pub use sparse_graph::{
    Adjacency, BidirectionalGraphScanner, BidirectionalSparseGraph, DirectedGraphScanner,
    DirectedSparseGraph, SparseGraph, TreeGraphScanner, UndirectedGraphScanner,
    UndirectedSparseGraph,
};
#[codesnip::entry("SparseGraph")]
pub mod sparse_graph {
    use super::*;
    use std::{iter, marker::PhantomData, ops, slice};

    type Marker<T> = PhantomData<fn() -> T>;
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct DirectedEdge;
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct UndirectedEdge;
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct BidirectionalEdge;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Adjacency {
        pub id: usize,
        pub to: usize,
    }
    impl Adjacency {
        pub fn new(id: usize, to: usize) -> Adjacency {
            Adjacency { id, to }
        }
    }

    /// Static Sparse Graph represented as Compressed Sparse Row.
    #[derive(Debug, Clone)]
    pub struct SparseGraph<D> {
        vsize: usize,
        pub start: Vec<usize>,
        pub elist: Vec<Adjacency>,
        pub edges: Vec<(usize, usize)>,
        _marker: Marker<D>,
    }

    impl<D> SparseGraph<D> {
        /// Return the number of vertices.
        pub fn vertices_size(&self) -> usize {
            self.vsize
        }
        /// Return the number of edges.
        pub fn edges_size(&self) -> usize {
            self.edges.len()
        }
        /// Return an iterator over graph vertices.
        pub fn vertices(&self) -> ops::Range<usize> {
            0..self.vertices_size()
        }
        /// Return a slice of adjacency vertices.
        pub fn adjacencies(&self, v: usize) -> slice::Iter<'_, Adjacency> {
            self.elist[self.start[v]..self.start[v + 1]].iter()
        }
    }

    pub trait SparseGraphConstruction: Sized {
        fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self>;
    }

    impl<D: SparseGraphConstruction> SparseGraph<D> {
        /// Construct graph from edges.
        pub fn from_edges(vsize: usize, edges: Vec<(usize, usize)>) -> Self {
            D::construct_graph(vsize, edges)
        }
    }

    impl SparseGraphConstruction for DirectedEdge {
        fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self> {
            let mut start: Vec<_> = iter::repeat(0).take(vsize + 1).collect();
            let mut elist = Vec::with_capacity(edges.len());
            unsafe { elist.set_len(edges.len()) }
            for (from, _) in edges.iter().cloned() {
                start[from] += 1;
            }
            for i in 1..=vsize {
                start[i] += start[i - 1];
            }
            for (id, (from, to)) in edges.iter().cloned().enumerate() {
                start[from] -= 1;
                elist[start[from]] = Adjacency::new(id, to);
            }
            SparseGraph {
                vsize,
                start,
                elist,
                edges,
                _marker: PhantomData,
            }
        }
    }

    impl SparseGraphConstruction for UndirectedEdge {
        fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self> {
            let mut start: Vec<_> = iter::repeat(0).take(vsize + 1).collect();
            let mut elist = Vec::with_capacity(edges.len() * 2);
            unsafe { elist.set_len(edges.len() * 2) }
            for (from, to) in edges.iter().cloned() {
                start[to] += 1;
                start[from] += 1;
            }
            for i in 1..=vsize {
                start[i] += start[i - 1];
            }
            for (id, (from, to)) in edges.iter().cloned().enumerate() {
                start[from] -= 1;
                elist[start[from]] = Adjacency::new(id, to);
                start[to] -= 1;
                elist[start[to]] = Adjacency::new(id, from);
            }
            SparseGraph {
                vsize,
                start,
                elist,
                edges,
                _marker: PhantomData,
            }
        }
    }

    impl SparseGraphConstruction for BidirectionalEdge {
        fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self> {
            let mut start: Vec<_> = iter::repeat(0).take(vsize + 1).collect();
            let mut elist = Vec::with_capacity(edges.len() * 2);
            unsafe { elist.set_len(edges.len() * 2) }
            for (from, to) in edges.iter().cloned() {
                start[to] += 1;
                start[from] += 1;
            }
            for i in 1..=vsize {
                start[i] += start[i - 1];
            }
            for (id, (from, to)) in edges.iter().cloned().enumerate() {
                start[from] -= 1;
                elist[start[from]] = Adjacency::new(id * 2, to);
                start[to] -= 1;
                elist[start[to]] = Adjacency::new(id * 2 + 1, from);
            }
            SparseGraph {
                vsize,
                start,
                elist,
                edges,
                _marker: PhantomData,
            }
        }
    }

    pub type DirectedSparseGraph = SparseGraph<DirectedEdge>;
    pub type UndirectedSparseGraph = SparseGraph<UndirectedEdge>;
    pub type BidirectionalSparseGraph = SparseGraph<BidirectionalEdge>;

    pub struct SparseGraphScanner<U: IterScan<Output = usize>, T: IterScan, D> {
        vsize: usize,
        esize: usize,
        _marker: Marker<(U, T, D)>,
    }

    impl<U: IterScan<Output = usize>, T: IterScan, D> SparseGraphScanner<U, T, D> {
        pub fn new(vsize: usize, esize: usize) -> Self {
            Self {
                vsize,
                esize,
                _marker: PhantomData,
            }
        }
    }

    impl<U: IterScan<Output = usize>, T: IterScan, D: SparseGraphConstruction> MarkedIterScan
        for SparseGraphScanner<U, T, D>
    {
        type Output = (SparseGraph<D>, Vec<<T as IterScan>::Output>);
        fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
            let mut edges = Vec::with_capacity(self.esize);
            let mut rest = Vec::with_capacity(self.esize);
            for _ in 0..self.esize {
                edges.push((U::scan(iter)?, U::scan(iter)?));
                rest.push(T::scan(iter)?);
            }
            let graph = SparseGraph::from_edges(self.vsize, edges);
            Some((graph, rest))
        }
    }

    pub type DirectedGraphScanner<U, T> = SparseGraphScanner<U, T, DirectedEdge>;
    pub type UndirectedGraphScanner<U, T> = SparseGraphScanner<U, T, UndirectedEdge>;
    pub type BidirectionalGraphScanner<U, T> = SparseGraphScanner<U, T, BidirectionalEdge>;

    pub struct TreeGraphScanner<U: IterScan<Output = usize>, T: IterScan> {
        vsize: usize,
        _marker: Marker<(U, T)>,
    }
    impl<U: IterScan<Output = usize>, T: IterScan> TreeGraphScanner<U, T> {
        pub fn new(vsize: usize) -> Self {
            Self {
                vsize,
                _marker: PhantomData,
            }
        }
    }
    impl<U: IterScan<Output = usize>, T: IterScan> MarkedIterScan for TreeGraphScanner<U, T> {
        type Output = (UndirectedSparseGraph, Vec<<T as IterScan>::Output>);
        fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
            UndirectedGraphScanner::<U, T>::new(self.vsize, self.vsize - 1).mscan(iter)
        }
    }
}
