use super::{
    Adjacencies, AdjacenciesWithEindex, AdjacencyIndex, AdjacencyIndexWithEindex, AdjacencyView,
    AdjacencyViewIterFromEindex, EIndexedGraph, EdgeMap, EdgeSize, EdgeView, GraphBase, IterScan,
    MarkedIterScan, VertexMap, VertexSize, VertexView, Vertices,
};
use std::{iter::Cloned, marker::PhantomData, ops, slice};

type Marker<T> = PhantomData<fn() -> T>;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DirectedEdge {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum UndirectedEdge {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BidirectionalEdge {}

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

impl<D> SparseGraph<D>
where
    D: SparseGraphConstruction,
{
    /// Construct graph from edges.
    pub fn from_edges(vsize: usize, edges: Vec<(usize, usize)>) -> Self {
        D::construct_graph(vsize, edges)
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
        let mut elist = Vec::<Adjacency>::with_capacity(edges.len());
        let ptr = elist.as_mut_ptr();
        for (id, (from, to)) in edges.iter().cloned().enumerate() {
            start[from] -= 1;
            unsafe { ptr.add(start[from]).write(Adjacency::new(id, to)) };
        }
        unsafe { elist.set_len(edges.len()) };
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
        let mut start: Vec<_> = vec![0usize; vsize + 1];
        for (from, to) in edges.iter().cloned() {
            start[to] += 1;
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        let mut elist = Vec::<Adjacency>::with_capacity(edges.len() * 2);
        let ptr = elist.as_mut_ptr();
        for (id, (from, to)) in edges.iter().cloned().enumerate() {
            start[from] -= 1;
            unsafe { ptr.add(start[from]).write(Adjacency::new(id, to)) };
            start[to] -= 1;
            unsafe { ptr.add(start[to]).write(Adjacency::new(id, from)) };
        }
        unsafe { elist.set_len(edges.len() * 2) };
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
        let mut start: Vec<_> = vec![0usize; vsize + 1];
        for (from, to) in edges.iter().cloned() {
            start[to] += 1;
            start[from] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        let mut elist = Vec::<Adjacency>::with_capacity(edges.len() * 2);
        let ptr = elist.as_mut_ptr();
        for (id, (from, to)) in edges.iter().cloned().enumerate() {
            start[from] -= 1;
            unsafe { ptr.add(start[from]).write(Adjacency::new(id * 2, to)) };
            start[to] -= 1;
            unsafe { ptr.add(start[to]).write(Adjacency::new(id * 2 + 1, from)) };
        }
        unsafe { elist.set_len(edges.len() * 2) };
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

impl<D> GraphBase<'_> for SparseGraph<D> {
    type VIndex = usize;
}
impl<D> EIndexedGraph<'_> for SparseGraph<D> {
    type EIndex = usize;
}

impl<D> VertexSize<'_> for SparseGraph<D> {
    fn vsize(&self) -> usize {
        self.vsize
    }
}
impl<D> EdgeSize<'_> for SparseGraph<D> {
    fn esize(&self) -> usize {
        self.edges.len()
    }
}

impl<D> Vertices<'_> for SparseGraph<D> {
    type VIter = ops::Range<usize>;
    fn vertices(&self) -> Self::VIter {
        0..self.vsize
    }
}
impl<'g, D> Adjacencies<'g> for SparseGraph<D> {
    type AIndex = Adjacency;
    type AIter = Cloned<slice::Iter<'g, Adjacency>>;
    fn adjacencies(&'g self, vid: Self::VIndex) -> Self::AIter {
        self.elist[self.start[vid]..self.start[vid + 1]]
            .iter()
            .cloned()
    }
}
impl<'g, D> AdjacenciesWithEindex<'g> for SparseGraph<D> {
    type AIndex = Adjacency;
    type AIter = Cloned<slice::Iter<'g, Adjacency>>;
    fn adjacencies_with_eindex(&'g self, vid: Self::VIndex) -> Self::AIter {
        self.elist[self.start[vid]..self.start[vid + 1]]
            .iter()
            .cloned()
    }
}

impl AdjacencyIndex for Adjacency {
    type VIndex = usize;
    fn vindex(&self) -> Self::VIndex {
        self.to
    }
}
impl AdjacencyIndexWithEindex for Adjacency {
    type EIndex = usize;
    fn eindex(&self) -> Self::EIndex {
        self.id
    }
}

impl<D, T> VertexMap<'_, T> for SparseGraph<D> {
    type Vmap = Vec<T>;
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        let mut v = Vec::with_capacity(self.vsize);
        v.resize_with(self.vsize, f);
        v
    }
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vid: Self::VIndex) -> &'a T {
        assert!(vid < self.vsize, "expected 0..{}, but {}", self.vsize, vid);
        unsafe { map.get_unchecked(vid) }
    }
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vid: Self::VIndex) -> &'a mut T {
        assert!(vid < self.vsize, "expected 0..{}, but {}", self.vsize, vid);
        unsafe { map.get_unchecked_mut(vid) }
    }
}
impl<D, T> VertexView<'_, Vec<T>, T> for SparseGraph<D>
where
    T: Clone,
{
    fn vview(&self, map: &Vec<T>, vid: Self::VIndex) -> T {
        self.vmap_get(map, vid).clone()
    }
}
impl<D, T> VertexView<'_, [T], T> for SparseGraph<D>
where
    T: Clone,
{
    fn vview(&self, map: &[T], vid: Self::VIndex) -> T {
        assert!(vid < self.vsize, "expected 0..{}, but {}", self.vsize, vid);
        unsafe { map.get_unchecked(vid) }.clone()
    }
}

impl<D, T> EdgeMap<'_, T> for SparseGraph<D> {
    type Emap = Vec<T>;
    fn construct_emap<F>(&self, f: F) -> Self::Emap
    where
        F: FnMut() -> T,
    {
        let mut v = Vec::with_capacity(self.vsize);
        v.resize_with(self.vsize, f);
        v
    }
    fn emap_get<'a>(&self, map: &'a Self::Emap, eid: Self::EIndex) -> &'a T {
        let esize = self.edges.len();
        assert!(eid < esize, "expected 0..{}, but {}", esize, eid);
        unsafe { map.get_unchecked(eid) }
    }
    fn emap_get_mut<'a>(&self, map: &'a mut Self::Emap, eid: Self::EIndex) -> &'a mut T {
        let esize = self.edges.len();
        assert!(eid < esize, "expected 0..{}, but {}", esize, eid);
        unsafe { map.get_unchecked_mut(eid) }
    }
}
impl<D, T> EdgeView<'_, Vec<T>, T> for SparseGraph<D>
where
    T: Clone,
{
    fn eview(&self, map: &Vec<T>, eid: Self::EIndex) -> T {
        self.emap_get(map, eid).clone()
    }
}

impl<D, T> EdgeView<'_, [T], T> for SparseGraph<D>
where
    T: Clone,
{
    fn eview(&self, map: &[T], eid: Self::EIndex) -> T {
        let esize = self.edges.len();
        assert!(eid < esize, "expected 0..{}, but {}", esize, eid);
        unsafe { map.get_unchecked(eid) }.clone()
    }
}

impl<'g, 'a, D: 'g, M, T> AdjacencyView<'g, 'a, M, T> for SparseGraph<D>
where
    Self: AdjacenciesWithEindex<'g> + EdgeView<'g, M, T>,
    T: Clone,
    M: 'a,
{
    type AViewIter = AdjacencyViewIterFromEindex<'g, 'a, Self, M, T>;
    fn aviews(&'g self, map: &'a M, vid: Self::VIndex) -> Self::AViewIter {
        AdjacencyViewIterFromEindex::new(self.adjacencies_with_eindex(vid), self, map)
    }
}
