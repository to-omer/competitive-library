use crate::tools::{IterScan, MarkedIterScan};

#[cargo_snippet::snippet("EdgeListGraph")]
#[derive(Clone, Debug)]
/// Graph represented by a list of edges.
pub struct EdgeListGraph {
    vsize: usize,
    edges: Vec<(usize, usize)>,
}
#[cargo_snippet::snippet("EdgeListGraph")]
impl EdgeListGraph {
    /// Construct empty graph.
    pub fn new(vsize: usize) -> Self {
        Self {
            vsize,
            edges: Vec::new(),
        }
    }
    /// Return the number of vertices.
    pub fn vertices_size(&self) -> usize {
        self.vsize
    }
    /// Return the number of edges.
    pub fn edges_size(&self) -> usize {
        self.edges.len()
    }
    /// Return an iterator over graph vertices.
    pub fn vertices(&self) -> std::ops::Range<usize> {
        0..self.vertices_size()
    }
    pub fn edges<'a>(&'a self) -> std::slice::Iter<'a, (usize, usize)> {
        self.edges.iter()
    }
    /// Construct graph from edges.
    pub fn from_edges(vsize: usize, edges: Vec<(usize, usize)>) -> Self {
        Self { vsize, edges }
    }
}
#[cargo_snippet::snippet("EdgeListGraph")]
impl std::ops::Index<usize> for EdgeListGraph {
    type Output = (usize, usize);
    fn index(&self, index: usize) -> &Self::Output {
        &self.edges[index]
    }
}

#[cargo_snippet::snippet("EdgeListGraph")]
pub struct EdgeListGraphScanner<U: IterScan<Output = usize>, T: IterScan> {
    vsize: usize,
    esize: usize,
    _marker: std::marker::PhantomData<fn() -> (U, T)>,
}

#[cargo_snippet::snippet("EdgeListGraph")]
impl<U: IterScan<Output = usize>, T: IterScan> EdgeListGraphScanner<U, T> {
    pub fn new(vsize: usize, esize: usize) -> Self {
        Self {
            vsize,
            esize,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cargo_snippet::snippet("EdgeListGraph")]
impl<U: IterScan<Output = usize>, T: IterScan> MarkedIterScan for EdgeListGraphScanner<U, T> {
    type Output = (EdgeListGraph, Vec<<T as IterScan>::Output>);
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut edges = Vec::with_capacity(self.esize);
        let mut rest = Vec::with_capacity(self.esize);
        for _ in 0..self.esize {
            edges.push((U::scan(iter)?, U::scan(iter)?));
            rest.push(T::scan(iter)?);
        }
        let graph = EdgeListGraph::from_edges(self.vsize, edges);
        Some((graph, rest))
    }
}
