use super::{AdjacencyGraphAbstraction, UndirectedSparseGraph};

#[cargo_snippet::snippet("LowLink")]
pub struct LowLink<'a> {
    graph: &'a UndirectedSparseGraph,
    pub low: Vec<usize>,
    pub ord: Vec<usize>,
    pub articulation: Vec<usize>,
    pub bridge: Vec<(usize, usize)>,
}
#[cargo_snippet::snippet("LowLink")]
impl<'a> LowLink<'a> {
    pub fn new(graph: &'a UndirectedSparseGraph) -> Self {
        let mut self_ = Self {
            graph,
            low: vec![0; graph.vertices_size()],
            ord: vec![std::usize::MAX; graph.vertices_size()],
            articulation: vec![],
            bridge: vec![],
        };
        for u in graph.vertices() {
            if self_.ord[u] == std::usize::MAX {
                self_.dfs(u, graph.vertices_size(), &mut 0);
            }
        }
        self_
    }
    fn dfs(&mut self, u: usize, p: usize, now_ord: &mut usize) {
        self.low[u] = *now_ord;
        self.ord[u] = *now_ord;
        *now_ord += 1;
        let mut is_articulation = false;
        let mut cnt = 0;
        for a in self.graph.adjacencies(u) {
            if self.ord[a.to] == std::usize::MAX {
                cnt += 1;
                self.dfs(a.to, u, now_ord);
                self.low[u] = self.low[u].min(self.low[a.to]);
                is_articulation |= p < self.graph.vertices_size() && self.ord[u] <= self.low[a.to];
                if self.ord[u] < self.low[a.to] {
                    self.bridge.push((u.min(a.to), u.max(a.to)));
                }
            } else if a.to != p {
                self.low[u] = self.low[u].min(self.ord[a.to]);
            }
        }
        is_articulation |= p == self.graph.vertices_size() && cnt > 1;
        if is_articulation {
            self.articulation.push(u);
        }
    }
}
