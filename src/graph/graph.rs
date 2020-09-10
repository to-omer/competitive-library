use crate::tools::{IterScan, MarkedIterScan};

#[cargo_snippet::snippet("Graph")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Adjacent {
    pub id: usize,
    pub to: usize,
}
#[cargo_snippet::snippet("Graph")]
impl Adjacent {
    pub fn new(id: usize, to: usize) -> Adjacent {
        Adjacent { id, to }
    }
}
#[cargo_snippet::snippet("Graph")]
#[derive(Clone, Debug, Default)]
pub struct Graph {
    pub vsize: usize,
    pub esize: usize,
    pub graph: Vec<Vec<Adjacent>>,
}
#[cargo_snippet::snippet("Graph")]
impl Graph {
    pub fn new(vsize: usize) -> Graph {
        Graph {
            vsize,
            esize: 0,
            graph: vec![vec![]; vsize],
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.graph[from].push(Adjacent::new(self.esize, to));
        self.esize += 1;
    }
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        self.graph[u].push(Adjacent::new(self.esize, v));
        self.graph[v].push(Adjacent::new(self.esize, u));
        self.esize += 1;
    }
    pub fn vertices(&self) -> std::ops::Range<usize> {
        0..self.vsize
    }
    pub fn adjacency(&self, from: usize) -> &Vec<Adjacent> {
        &self.graph[from]
    }
    pub fn eid_cache(&self) -> GraphEidCache<'_> {
        let mut cache = vec![(0, 0); self.esize];
        for u in self.vertices() {
            for a in self.adjacency(u) {
                cache[a.id] = (u, a.to);
            }
        }
        GraphEidCache { graph: self, cache }
    }
}

#[cargo_snippet::snippet("Graph")]
pub struct GraphScanner<U: IterScan<Output = usize>, T: IterScan> {
    vsize: usize,
    esize: usize,
    directed: bool,
    _marker: std::marker::PhantomData<fn() -> (U, T)>,
}

#[cargo_snippet::snippet("Graph")]
impl<U: IterScan<Output = usize>, T: IterScan> GraphScanner<U, T> {
    pub fn new(vsize: usize, esize: usize, directed: bool) -> Self {
        Self {
            vsize,
            esize,
            directed,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cargo_snippet::snippet("Graph")]
impl<U: IterScan<Output = usize>, T: IterScan> MarkedIterScan for GraphScanner<U, T> {
    type Output = (Graph, Vec<<T as IterScan>::Output>);
    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut graph = Graph::new(self.vsize);
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

#[cargo_snippet::snippet("GraphRec")]
#[derive(Debug)]
pub struct GraphRec {
    pub n: usize,
    pub visited: Vec<bool>,
    pub cost: Vec<usize>,
}
#[cargo_snippet::snippet("GraphRec")]
impl GraphRec {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            visited: vec![false; n],
            cost: vec![0; n],
        }
    }
    pub fn dfs(&mut self, u: usize, graph: &Graph) {
        let d = self.cost[u];
        for a in graph.adjacency(u) {
            if !self.visited[a.to] {
                self.visited[a.to] = true;
                self.cost[a.to] = d + 1;
                self.dfs(a.to, graph);
            }
        }
    }
}

#[cargo_snippet::snippet("Graph")]
#[derive(Debug, Clone)]
pub struct GraphEidCache<'a> {
    graph: &'a Graph,
    cache: Vec<(usize, usize)>,
}
#[cargo_snippet::snippet("Graph")]
impl<'a> std::ops::Index<usize> for GraphEidCache<'a> {
    type Output = (usize, usize);
    fn index(&self, index: usize) -> &Self::Output {
        &self.cache[index]
    }
}

#[cargo_snippet::snippet("GridGraph")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GridGraph {
    height: usize,
    width: usize,
}
#[cargo_snippet::snippet("GridGraph")]
impl GridGraph {
    pub fn new(height: usize, width: usize) -> Self {
        Self { height, width }
    }
    pub fn adjacency4(&self, x: usize, y: usize) -> Adjacent4<'_> {
        Adjacent4 {
            grid: self,
            x,
            y,
            state: 0,
        }
    }
    pub fn adjacency8(&self, x: usize, y: usize) -> Adjacent8<'_> {
        Adjacent8 {
            grid: self,
            x,
            y,
            state: 0,
        }
    }
}

#[cargo_snippet::snippet("GridGraph")]
#[derive(Debug)]
pub struct Adjacent4<'a> {
    grid: &'a GridGraph,
    x: usize,
    y: usize,
    state: usize,
}
#[cargo_snippet::snippet("GridGraph")]
impl<'a> Iterator for Adjacent4<'a> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        const D: [(usize, usize); 4] = [(1, 0), (0, 1), (!0, 0), (0, !0)];
        for &(dx, dy) in D[self.state..].into_iter() {
            self.state += 1;
            let nx = self.x.wrapping_add(dx);
            let ny = self.y.wrapping_add(dy);
            if nx < self.grid.height && ny < self.grid.width {
                return Some((nx, ny));
            }
        }
        None
    }
}
#[cargo_snippet::snippet("GridGraph")]
#[derive(Debug)]
pub struct Adjacent8<'a> {
    grid: &'a GridGraph,
    x: usize,
    y: usize,
    state: usize,
}
#[cargo_snippet::snippet("GridGraph")]
impl<'a> Iterator for Adjacent8<'a> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        const D: [(usize, usize); 8] = [
            (1, 0),
            (1, 1),
            (0, 1),
            (!0, 1),
            (!0, 0),
            (!0, !0),
            (0, !0),
            (1, !0),
        ];
        for &(dx, dy) in D[self.state..].into_iter() {
            self.state += 1;
            let nx = self.x.wrapping_add(dx);
            let ny = self.y.wrapping_add(dy);
            if nx < self.grid.height && ny < self.grid.width {
                return Some((nx, ny));
            }
        }
        None
    }
}

/// Sparse Graph represented as Compressed Sparse Row.
#[derive(Debug, Clone)]
pub struct SparseGraph {
    pub vsize: usize,
    pub start: Vec<usize>,
    pub elist: Vec<usize>,
}
impl SparseGraph {
    /// Construct graph from directed edges.
    ///
    /// # Example
    /// ```
    /// # use competitive::graph::SparseGraph;
    /// let vsize = 4;
    /// let edges = vec![(0, 1), (3, 2), (1, 2), (1, 0)];
    /// let graph = SparseGraph::from_edges(vsize, edges.iter().cloned());
    ///
    /// assert_eq!(graph.adjacency(0), &[1]);
    /// assert_eq!(graph.adjacency(1), &[0, 2]);
    /// assert_eq!(graph.adjacency(2), &[]);
    /// assert_eq!(graph.adjacency(3), &[2]);
    /// ```
    pub fn from_edges(
        vsize: usize,
        edges: impl Iterator<Item = (usize, usize)> + ExactSizeIterator + Clone,
    ) -> SparseGraph {
        let mut start = vec![0; vsize + 1];
        let mut elist = vec![0; edges.len()];
        for e in edges.clone() {
            start[e.0] += 1;
        }
        for i in 1..=vsize {
            start[i] += start[i - 1];
        }
        for (from, to) in edges {
            start[from] -= 1;
            elist[start[from]] = to;
        }
        SparseGraph {
            vsize,
            start,
            elist,
        }
    }
    /// Return the number of edges.
    pub fn esize(&self) -> usize {
        self.elist.len()
    }
    /// Return an iterator over graph vertices.
    pub fn vertices(&self) -> std::ops::Range<usize> {
        0..self.vsize
    }
    /// Return an iterator over adjacency vertices.
    pub fn adjacency(&self, from: usize) -> &[usize] {
        &self.elist[self.start[from]..self.start[from + 1]]
    }
}
