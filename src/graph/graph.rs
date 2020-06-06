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
    pub fn new(n: usize) -> GraphRec {
        GraphRec {
            n: n,
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
