use super::{EdgeMap, Graph, IterScan, MarkedIterScan, Neighbor, VertexMap};
use std::{iter::Copied, marker::PhantomData, ops::Range, slice};

#[derive(Clone, Debug, Default)]
pub struct AdjacencyListGraph {
    pub vsize: usize,
    pub esize: usize,
    pub graph: Vec<Vec<Neighbor<usize, usize>>>,
}
impl AdjacencyListGraph {
    pub fn new(vsize: usize) -> AdjacencyListGraph {
        AdjacencyListGraph {
            vsize,
            esize: 0,
            graph: vec![vec![]; vsize],
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.graph[from].push(Neighbor::new(to, self.esize));
        self.esize += 1;
    }
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        self.graph[u].push(Neighbor::new(v, self.esize));
        self.graph[v].push(Neighbor::new(u, self.esize));
        self.esize += 1;
    }
    pub fn vertices(&self) -> Range<usize> {
        0..self.vsize
    }
}

impl Graph for AdjacencyListGraph {
    type Vertex = usize;
    type Label = usize;
    type Vertices<'g> = Range<usize>;
    type Neighbors<'g> = Copied<slice::Iter<'g, Neighbor<usize, usize>>>;

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
        self.graph[vertex].iter().copied()
    }
}

impl<T> EdgeMap<T> for AdjacencyListGraph {
    type Emap = Vec<T>;

    #[inline]
    fn construct_emap<F>(&self, f: F) -> Self::Emap
    where
        F: FnMut() -> T,
    {
        let mut map = Vec::with_capacity(self.esize);
        map.resize_with(self.esize, f);
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

impl<T> VertexMap<T> for AdjacencyListGraph {
    type Vmap = Vec<T>;

    #[inline]
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        let mut map = Vec::with_capacity(self.vsize);
        map.resize_with(self.vsize, f);
        map
    }

    #[inline]
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vertex: Self::Vertex) -> &'a T {
        &map[vertex]
    }

    #[inline]
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vertex: Self::Vertex) -> &'a mut T {
        &mut map[vertex]
    }
}

pub struct AdjacencyListGraphScanner<U: IterScan<Output = usize>, T: IterScan> {
    vsize: usize,
    esize: usize,
    directed: bool,
    _marker: PhantomData<fn() -> (U, T)>,
}

impl<U: IterScan<Output = usize>, T: IterScan> AdjacencyListGraphScanner<U, T> {
    pub fn new(vsize: usize, esize: usize, directed: bool) -> Self {
        Self {
            vsize,
            esize,
            directed,
            _marker: PhantomData,
        }
    }
}

impl<U: IterScan<Output = usize>, T: IterScan> MarkedIterScan for AdjacencyListGraphScanner<U, T> {
    type Output = (AdjacencyListGraph, Vec<<T as IterScan>::Output>);
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut graph = AdjacencyListGraph::new(self.vsize);
        let mut rest = Vec::with_capacity(self.esize);
        for _ in 0..self.esize {
            let (from, to) = (U::scan(iter)?, U::scan(iter)?);
            if self.directed {
                graph.add_edge(from, to);
            } else {
                graph.add_undirected_edge(from, to);
            }
            rest.push(T::scan(iter)?);
        }
        Some((graph, rest))
    }
}
