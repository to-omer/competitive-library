#[cargo_snippet::snippet("Graph")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Adjacent {
    pub to: usize,
    pub id: usize,
}
#[cargo_snippet::snippet("Graph")]
impl Adjacent {
    pub fn new(to: usize, id: usize) -> Adjacent {
        Adjacent { to: to, id: id }
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
            vsize: vsize,
            esize: 0,
            graph: vec![vec![]; vsize],
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.graph[from].push(Adjacent::new(to, self.esize));
        self.esize += 1;
    }
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        self.graph[u].push(Adjacent::new(v, self.esize));
        self.graph[v].push(Adjacent::new(u, self.esize));
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
