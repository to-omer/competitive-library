use super::{DirectedGraph, EdgeMap, Graph, IterScan, MarkedIterScan, Neighbor, VertexMap};
use std::{iter::Copied, marker::PhantomData, ops, slice};

type Marker<T> = PhantomData<fn() -> T>;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DirectedEdge {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum UndirectedEdge {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BidirectionalEdge {}

/// Static Sparse Graph represented as Compressed Sparse Row.
#[derive(Debug, Clone)]
pub struct SparseGraph<D> {
    vsize: usize,
    start: Vec<usize>,
    neighbors: Vec<Neighbor<usize, usize>>,
    pub edges: Vec<(usize, usize)>,
    _marker: Marker<D>,
}

impl<D> SparseGraph<D> {
    /// Return the number of vertices.
    #[inline]
    pub fn vertices_size(&self) -> usize {
        self.vsize
    }
    /// Return the number of edges.
    #[inline]
    pub fn edges_size(&self) -> usize {
        self.edges.len()
    }
    /// Return an iterator over graph vertices.
    #[inline]
    pub fn vertices(&self) -> ops::Range<usize> {
        0..self.vertices_size()
    }
    pub fn builder<T>(vsize: usize) -> SparseGraphBuilder<T, D> {
        SparseGraphBuilder::new(vsize)
    }
    pub fn builder_with_esize<T>(vsize: usize, esize: usize) -> SparseGraphBuilder<T, D> {
        SparseGraphBuilder::new_with_esize(vsize, esize)
    }
}

pub trait SparseGraphConstruction: Sized {
    fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self>;
}

impl<D> SparseGraph<D>
where
    D: SparseGraphConstruction,
{
    /// Construct graph from edges.
    pub fn from_edges(vsize: usize, edges: Vec<(usize, usize)>) -> Self {
        D::construct_graph(vsize, edges)
    }
    pub fn reverse_graph(&self) -> SparseGraph<D> {
        let edges = self.edges.iter().map(|&(from, to)| (to, from)).collect();
        D::construct_graph(self.vsize, edges)
    }
}

impl SparseGraphConstruction for DirectedEdge {
    fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self> {
        let mut start: Vec<_> = vec![0usize; vsize + 1];
        for (from, _) in edges.iter().cloned() {
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        let mut neighbors = Vec::<Neighbor<usize, usize>>::with_capacity(edges.len());
        let ptr = neighbors.as_mut_ptr();
        for (id, (from, to)) in edges.iter().cloned().enumerate() {
            start[from] -= 1;
            unsafe { ptr.add(start[from]).write(Neighbor::new(to, id)) };
        }
        unsafe { neighbors.set_len(edges.len()) };
        SparseGraph {
            vsize,
            start,
            neighbors,
            edges,
            _marker: PhantomData,
        }
    }
}

impl SparseGraphConstruction for UndirectedEdge {
    fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self> {
        let mut start: Vec<_> = vec![0usize; vsize + 1];
        for (from, to) in edges.iter().cloned() {
            start[to] += 1;
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        let mut neighbors = Vec::<Neighbor<usize, usize>>::with_capacity(edges.len() * 2);
        let ptr = neighbors.as_mut_ptr();
        for (id, (from, to)) in edges.iter().cloned().enumerate() {
            start[from] -= 1;
            unsafe { ptr.add(start[from]).write(Neighbor::new(to, id)) };
            start[to] -= 1;
            unsafe { ptr.add(start[to]).write(Neighbor::new(from, id)) };
        }
        unsafe { neighbors.set_len(edges.len() * 2) };
        SparseGraph {
            vsize,
            start,
            neighbors,
            edges,
            _marker: PhantomData,
        }
    }
}

impl SparseGraphConstruction for BidirectionalEdge {
    fn construct_graph(vsize: usize, edges: Vec<(usize, usize)>) -> SparseGraph<Self> {
        let mut start: Vec<_> = vec![0usize; vsize + 1];
        for (from, to) in edges.iter().cloned() {
            start[to] += 1;
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        let mut neighbors = Vec::<Neighbor<usize, usize>>::with_capacity(edges.len() * 2);
        let ptr = neighbors.as_mut_ptr();
        for (id, (from, to)) in edges.iter().cloned().enumerate() {
            start[from] -= 1;
            unsafe { ptr.add(start[from]).write(Neighbor::new(to, id * 2)) };
            start[to] -= 1;
            unsafe { ptr.add(start[to]).write(Neighbor::new(from, id * 2 + 1)) };
        }
        unsafe { neighbors.set_len(edges.len() * 2) };
        SparseGraph {
            vsize,
            start,
            neighbors,
            edges,
            _marker: PhantomData,
        }
    }
}

pub type DirectedSparseGraph = SparseGraph<DirectedEdge>;
pub type UndirectedSparseGraph = SparseGraph<UndirectedEdge>;
pub type BidirectionalSparseGraph = SparseGraph<BidirectionalEdge>;

pub struct SparseGraphBuilder<T, D> {
    vsize: usize,
    edges: Vec<(usize, usize)>,
    rest: Vec<T>,
    _marker: Marker<D>,
}
impl<T, D> SparseGraphBuilder<T, D> {
    pub fn new(vsize: usize) -> Self {
        Self {
            vsize,
            edges: Default::default(),
            rest: Default::default(),
            _marker: PhantomData,
        }
    }
    pub fn new_with_esize(vsize: usize, esize: usize) -> Self {
        Self {
            vsize,
            edges: Vec::with_capacity(esize),
            rest: Vec::with_capacity(esize),
            _marker: PhantomData,
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize, w: T) {
        self.edges.push((u, v));
        self.rest.push(w);
    }
}
impl<T, D> SparseGraphBuilder<T, D>
where
    D: SparseGraphConstruction,
{
    pub fn build(self) -> (SparseGraph<D>, Vec<T>) {
        let graph = SparseGraph::from_edges(self.vsize, self.edges);
        (graph, self.rest)
    }
}

pub struct SparseGraphScanner<U, T, D>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    vsize: usize,
    esize: usize,
    _marker: Marker<(U, T, D)>,
}

impl<U, T, D> SparseGraphScanner<U, T, D>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    pub fn new(vsize: usize, esize: usize) -> Self {
        Self {
            vsize,
            esize,
            _marker: PhantomData,
        }
    }
}

impl<U, T, D> MarkedIterScan for SparseGraphScanner<U, T, D>
where
    U: IterScan<Output = usize>,
    T: IterScan,
    D: SparseGraphConstruction,
{
    type Output = (SparseGraph<D>, Vec<<T as IterScan>::Output>);
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut builder = SparseGraphBuilder::new_with_esize(self.vsize, self.esize);
        for _ in 0..self.esize {
            builder.add_edge(U::scan(iter)?, U::scan(iter)?, T::scan(iter)?);
        }
        Some(builder.build())
    }
}

pub type DirectedGraphScanner<U, T = ()> = SparseGraphScanner<U, T, DirectedEdge>;
pub type UndirectedGraphScanner<U, T = ()> = SparseGraphScanner<U, T, UndirectedEdge>;
pub type BidirectionalGraphScanner<U, T = ()> = SparseGraphScanner<U, T, BidirectionalEdge>;

pub struct TreeGraphScanner<U, T = ()>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    vsize: usize,
    _marker: Marker<(U, T)>,
}
impl<U, T> TreeGraphScanner<U, T>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    pub fn new(vsize: usize) -> Self {
        Self {
            vsize,
            _marker: PhantomData,
        }
    }
}
impl<U, T> MarkedIterScan for TreeGraphScanner<U, T>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    type Output = (UndirectedSparseGraph, Vec<<T as IterScan>::Output>);
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        UndirectedGraphScanner::<U, T>::new(self.vsize, self.vsize - 1).mscan(iter)
    }
}

impl<D> Graph for SparseGraph<D>
where
    D: SparseGraphConstruction,
{
    type Vertex = usize;
    type Label = usize;
    type Vertices<'g>
        = ops::Range<usize>
    where
        D: 'g;
    type Neighbors<'g>
        = Copied<slice::Iter<'g, Neighbor<usize, usize>>>
    where
        D: 'g;

    #[inline]
    fn vsize(&self) -> usize {
        self.vsize
    }

    #[inline]
    fn vertices(&self) -> Self::Vertices<'_> {
        0..self.vsize
    }

    #[inline]
    fn neighbors(&self, vertex: Self::Vertex) -> Self::Neighbors<'_> {
        self.neighbors[self.start[vertex]..self.start[vertex + 1]]
            .iter()
            .copied()
    }
}

impl DirectedGraph for SparseGraph<DirectedEdge> {}

impl<T> EdgeMap<T> for SparseGraph<DirectedEdge> {
    type Emap = Vec<T>;

    #[inline]
    fn construct_emap<F>(&self, f: F) -> Self::Emap
    where
        F: FnMut() -> T,
    {
        let mut map = Vec::with_capacity(self.edges.len());
        map.resize_with(self.edges.len(), f);
        map
    }

    #[inline]
    fn emap_get<'a>(&self, map: &'a Self::Emap, eid: Self::Label) -> &'a T {
        &map[eid]
    }

    #[inline]
    fn emap_get_mut<'a>(&self, map: &'a mut Self::Emap, eid: Self::Label) -> &'a mut T {
        &mut map[eid]
    }
}

impl<T> EdgeMap<T> for SparseGraph<UndirectedEdge> {
    type Emap = Vec<T>;

    #[inline]
    fn construct_emap<F>(&self, f: F) -> Self::Emap
    where
        F: FnMut() -> T,
    {
        let mut map = Vec::with_capacity(self.edges.len());
        map.resize_with(self.edges.len(), f);
        map
    }

    #[inline]
    fn emap_get<'a>(&self, map: &'a Self::Emap, eid: Self::Label) -> &'a T {
        &map[eid]
    }

    #[inline]
    fn emap_get_mut<'a>(&self, map: &'a mut Self::Emap, eid: Self::Label) -> &'a mut T {
        &mut map[eid]
    }
}

impl<D, T> VertexMap<T> for SparseGraph<D>
where
    D: SparseGraphConstruction,
{
    type Vmap = Vec<T>;
    #[inline]
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        let mut v = Vec::with_capacity(self.vsize);
        v.resize_with(self.vsize, f);
        v
    }
    #[inline]
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vid: Self::Vertex) -> &'a T {
        &map[vid]
    }
    #[inline]
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vid: Self::Vertex) -> &'a mut T {
        &mut map[vid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    #[test]
    fn test_sparse_graph() {
        const Q: usize = 1_000;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);

            let directed = DirectedSparseGraph::from_edges(n, edges.clone());
            let mut occurrences = vec![0; m];
            for u in directed.vertices() {
                for neighbor in directed.neighbors(u) {
                    assert_eq!(edges[neighbor.label], (u, neighbor.to));
                    occurrences[neighbor.label] += 1;
                }
            }
            assert!(occurrences.into_iter().all(|count| count == 1));

            let undirected = UndirectedSparseGraph::from_edges(n, edges.clone());
            let mut occurrences = vec![0; m];
            for u in undirected.vertices() {
                for neighbor in undirected.neighbors(u) {
                    let (from, to) = edges[neighbor.label];
                    assert!((u == from && neighbor.to == to) || (u == to && neighbor.to == from));
                    occurrences[neighbor.label] += 1;
                }
            }
            assert!(occurrences.into_iter().all(|count| count == 2));

            let bidirectional = BidirectionalSparseGraph::from_edges(n, edges.clone());
            let mut occurrences = vec![0; m * 2];
            for u in bidirectional.vertices() {
                for neighbor in bidirectional.neighbors(u) {
                    let (from, to) = edges[neighbor.label / 2];
                    let expected = if neighbor.label % 2 == 0 {
                        (from, to)
                    } else {
                        (to, from)
                    };
                    assert_eq!((u, neighbor.to), expected);
                    occurrences[neighbor.label] += 1;
                }
            }
            assert!(occurrences.into_iter().all(|count| count == 1));
        }
    }
}
