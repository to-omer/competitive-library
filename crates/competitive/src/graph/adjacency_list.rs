use crate::tools::{IterScan, MarkedIterScan};

#[codesnip::entry("AdjacencyListGraph", include("scanner"))]
pub use adjacency_list_graph::{AdjacencyListGraph, AdjacencyListGraphScanner};
#[codesnip::entry("AdjacencyListGraph")]
pub mod adjacency_list_graph {
    use super::*;
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
    #[derive(Clone, Debug, Default)]
    pub struct AdjacencyListGraph {
        pub vsize: usize,
        pub esize: usize,
        pub graph: Vec<Vec<Adjacency>>,
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
            self.graph[from].push(Adjacency::new(self.esize, to));
            self.esize += 1;
        }
        pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
            self.graph[u].push(Adjacency::new(self.esize, v));
            self.graph[v].push(Adjacency::new(self.esize, u));
            self.esize += 1;
        }
        pub fn vertices(&self) -> std::ops::Range<usize> {
            0..self.vsize
        }
        pub fn adjacency(&self, from: usize) -> &Vec<Adjacency> {
            &self.graph[from]
        }
    }

    pub struct AdjacencyListGraphScanner<U: IterScan<Output = usize>, T: IterScan> {
        vsize: usize,
        esize: usize,
        directed: bool,
        _marker: std::marker::PhantomData<fn() -> (U, T)>,
    }

    impl<U: IterScan<Output = usize>, T: IterScan> AdjacencyListGraphScanner<U, T> {
        pub fn new(vsize: usize, esize: usize, directed: bool) -> Self {
            Self {
                vsize,
                esize,
                directed,
                _marker: std::marker::PhantomData,
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
}
