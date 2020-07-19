use super::Graph;

#[cargo_snippet::snippet("LowLink")]
pub struct LowLink {
    pub ord: Vec<usize>,
    pub low: Vec<usize>,
    pub articulation: Vec<usize>,
    pub bridge: Vec<(usize, usize)>,
}
#[cargo_snippet::snippet("LowLink")]
impl LowLink {
    pub fn new(graph: &Graph) -> Self {
        let mut self_ = Self {
            ord: vec![0; graph.vsize],
            low: vec![0; graph.vsize],
            articulation: vec![],
            bridge: vec![],
        };
        let mut visited = vec![false; graph.vsize];
        for u in graph.vertices() {
            if !visited[u] {
                self_.dfs(u, graph.vsize, &mut 0, &mut visited, &graph);
            }
        }
        self_
    }
    fn dfs(&mut self, u: usize, p: usize, i: &mut usize, visited: &mut Vec<bool>, graph: &Graph) {
        visited[u] = true;
        self.ord[u] = *i;
        self.low[u] = *i;
        *i += 1;
        let mut is_articulation = false;
        let mut cnt = 0;
        for a in graph.adjacency(u) {
            if !visited[a.to] {
                cnt += 1;
                self.dfs(a.to, u, i, visited, graph);
                self.low[u] = self.low[u].min(self.low[a.to]);
                is_articulation |= p < graph.vsize && self.ord[u] <= self.low[a.to];
                if self.ord[u] < self.low[a.to] {
                    self.bridge.push((u.min(a.to), u.max(a.to)));
                }
            } else if a.to != p {
                self.low[u] = self.low[u].min(self.ord[a.to]);
            }
        }
        is_articulation |= p == graph.vsize && cnt > 1;
        if is_articulation {
            self.articulation.push(u);
        }
    }
}
