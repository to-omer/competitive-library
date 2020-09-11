use crate::tools::{IterScan, MarkedIterScan};

#[cargo_snippet::snippet("SparseGraph")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DirectedEdge;
#[cargo_snippet::snippet("SparseGraph")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct UndirectedEdge;

#[cargo_snippet::snippet("SparseGraph")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Adjacency {
    pub id: usize,
    pub to: usize,
}
#[cargo_snippet::snippet("SparseGraph")]
impl Adjacency {
    pub fn new(id: usize, to: usize) -> Adjacency {
        Adjacency { id, to }
    }
}

#[cargo_snippet::snippet("SparseGraph")]
/// Adjacency Graph Abstraction.
pub trait AdjacencyGraphAbstraction<'a> {
    type AdjIter: Iterator<Item = &'a Adjacency> + ExactSizeIterator;
    /// Return the number of vertices.
    fn vertices_size(&self) -> usize;
    /// Return the number of edges.
    fn edges_size(&self) -> usize;
    /// Return an iterator over graph vertices.
    fn vertices(&self) -> std::ops::Range<usize> {
        0..self.vertices_size()
    }
    /// Return a slice of adjacency vertices.
    fn adjacencies(&'a self, v: usize) -> Self::AdjIter;
    /// Construct graph from edges.
    fn from_edges(
        vsize: usize,
        edges: impl Iterator<Item = (usize, usize)> + ExactSizeIterator + Clone,
    ) -> Self;
}

#[cargo_snippet::snippet("SparseGraph")]
/// Static Sparse Graph represented as Compressed Sparse Row.
#[derive(Debug, Clone)]
pub struct SparseGraph<D> {
    vsize: usize,
    start: Vec<usize>,
    elist: Vec<Adjacency>,
    _marker: std::marker::PhantomData<fn() -> D>,
}
#[cargo_snippet::snippet("SparseGraph")]
impl<'a> AdjacencyGraphAbstraction<'a> for SparseGraph<DirectedEdge> {
    type AdjIter = std::slice::Iter<'a, Adjacency>;
    fn vertices_size(&self) -> usize {
        self.vsize
    }
    fn edges_size(&self) -> usize {
        self.elist.len()
    }
    fn adjacencies(&'a self, v: usize) -> Self::AdjIter {
        self.elist[self.start[v]..self.start[v + 1]].iter()
    }
    fn from_edges(
        vsize: usize,
        edges: impl Iterator<Item = (usize, usize)> + ExactSizeIterator + Clone,
    ) -> Self {
        let mut start = vec![0; vsize + 1];
        let mut elist = Vec::with_capacity(edges.len());
        unsafe { elist.set_len(edges.len()) }
        for (from, _) in edges.clone() {
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        for (id, (from, to)) in edges.enumerate() {
            start[from] -= 1;
            elist[start[from]] = Adjacency::new(id, to);
        }
        Self {
            vsize,
            start,
            elist,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cargo_snippet::snippet("SparseGraph")]
impl<'a> AdjacencyGraphAbstraction<'a> for SparseGraph<UndirectedEdge> {
    type AdjIter = std::slice::Iter<'a, Adjacency>;
    fn vertices_size(&self) -> usize {
        self.vsize
    }
    fn edges_size(&self) -> usize {
        self.elist.len() / 2
    }
    fn adjacencies(&'a self, v: usize) -> Self::AdjIter {
        self.elist[self.start[v]..self.start[v + 1]].iter()
    }
    fn from_edges(
        vsize: usize,
        edges: impl Iterator<Item = (usize, usize)> + ExactSizeIterator + Clone,
    ) -> Self {
        let mut start = vec![0; vsize + 1];
        let mut elist = Vec::with_capacity(edges.len() * 2);
        unsafe { elist.set_len(edges.len() * 2) }
        for (from, to) in edges.clone() {
            start[to] += 1;
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        for (id, (from, to)) in edges.enumerate() {
            start[from] -= 1;
            elist[start[from]] = Adjacency::new(id, to);
            start[to] -= 1;
            elist[start[to]] = Adjacency::new(id, from);
        }
        Self {
            vsize,
            start,
            elist,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cargo_snippet::snippet("SparseGraph")]
pub type DirectedSparseGraph = SparseGraph<DirectedEdge>;
#[cargo_snippet::snippet("SparseGraph")]
pub type UndirectedSparseGraph = SparseGraph<UndirectedEdge>;

#[cargo_snippet::snippet("SparseGraph")]
pub struct AdjacencyGraphScanner<U: IterScan<Output = usize>, T: IterScan, D> {
    vsize: usize,
    esize: usize,
    _marker: std::marker::PhantomData<fn() -> (U, T, D)>,
}

#[cargo_snippet::snippet("SparseGraph")]
impl<U: IterScan<Output = usize>, T: IterScan, D> AdjacencyGraphScanner<U, T, D> {
    pub fn new(vsize: usize, esize: usize) -> Self {
        Self {
            vsize,
            esize,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cargo_snippet::snippet("SparseGraph")]
impl<U: IterScan<Output = usize>, T: IterScan, D> MarkedIterScan for AdjacencyGraphScanner<U, T, D>
where
    SparseGraph<D>: for<'a> AdjacencyGraphAbstraction<'a>,
{
    type Output = (
        SparseGraph<D>,
        Vec<(usize, usize)>,
        Vec<<T as IterScan>::Output>,
    );
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut edges = Vec::with_capacity(self.esize);
        let mut rest = Vec::with_capacity(self.esize);
        for _ in 0..self.esize {
            edges.push((U::scan(iter)?, U::scan(iter)?));
            rest.push(T::scan(iter)?);
        }
        let graph = SparseGraph::from_edges(self.vsize, edges.iter().map(|t| (t.0, t.1)));
        Some((graph, edges, rest))
    }
}

#[cargo_snippet::snippet("SparseGraph")]
pub type DirectedGraphScanner<U, T> = AdjacencyGraphScanner<U, T, DirectedEdge>;
#[cargo_snippet::snippet("SparseGraph")]
pub type UndirectedGraphScanner<U, T> = AdjacencyGraphScanner<U, T, UndirectedEdge>;
