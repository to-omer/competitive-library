use super::Graph;

#[cargo_snippet::snippet("LowLink")]
pub struct LowLink<'a> {
    graph: &'a Graph,
    pub low: Vec<usize>,
    pub ord: Vec<usize>,
    pub articulation: Vec<usize>,
    pub bridge: Vec<(usize, usize)>,
}
#[cargo_snippet::snippet("LowLink")]
impl<'a> LowLink<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        let mut self_ = Self {
            graph,
            low: vec![0; graph.vsize],
            ord: vec![std::usize::MAX; graph.vsize],
            articulation: vec![],
            bridge: vec![],
        };
        for u in graph.vertices() {
            if self_.ord[u] == std::usize::MAX {
                self_.dfs(u, graph.vsize, &mut 0);
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
        for a in self.graph.adjacency(u) {
            if self.ord[a.to] == std::usize::MAX {
                cnt += 1;
                self.dfs(a.to, u, now_ord);
                self.low[u] = self.low[u].min(self.low[a.to]);
                is_articulation |= p < self.graph.vsize && self.ord[u] <= self.low[a.to];
                if self.ord[u] < self.low[a.to] {
                    self.bridge.push((u.min(a.to), u.max(a.to)));
                }
            } else if a.to != p {
                self.low[u] = self.low[u].min(self.ord[a.to]);
            }
        }
        is_articulation |= p == self.graph.vsize && cnt > 1;
        if is_articulation {
            self.articulation.push(u);
        }
    }
}
