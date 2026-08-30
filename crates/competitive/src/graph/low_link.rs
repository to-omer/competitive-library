use super::{Graph, UndirectedSparseGraph};

pub struct LowLink<'a> {
    graph: &'a UndirectedSparseGraph,
    pub low: Vec<usize>,
    pub ord: Vec<usize>,
    pub articulation: Vec<usize>,
    pub bridge: Vec<(usize, usize)>,
}
impl<'a> LowLink<'a> {
    pub fn new(graph: &'a UndirectedSparseGraph) -> Self {
        let mut self_ = Self {
            graph,
            low: vec![0; graph.vertices_size()],
            ord: vec![usize::MAX; graph.vertices_size()],
            articulation: vec![],
            bridge: vec![],
        };
        for u in graph.vertices() {
            if self_.ord[u] == usize::MAX {
                self_.dfs(u, !0, &mut 0);
            }
        }
        self_
    }
    fn dfs(&mut self, u: usize, parent_eid: usize, now_ord: &mut usize) {
        self.low[u] = *now_ord;
        self.ord[u] = *now_ord;
        *now_ord += 1;
        let mut is_articulation = false;
        let mut cnt = 0;
        for a in self.graph.neighbors(u) {
            if a.label == parent_eid {
                continue;
            }
            if self.ord[a.to] == usize::MAX {
                cnt += 1;
                self.dfs(a.to, a.label, now_ord);
                self.low[u] = self.low[u].min(self.low[a.to]);
                is_articulation |= parent_eid != !0 && self.ord[u] <= self.low[a.to];
                if self.ord[u] < self.low[a.to] {
                    self.bridge.push((u.min(a.to), u.max(a.to)));
                }
            } else {
                self.low[u] = self.low[u].min(self.ord[a.to]);
            }
        }
        is_articulation |= parent_eid == !0 && cnt > 1;
        if is_articulation {
            self.articulation.push(u);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    fn components(
        n: usize,
        edges: &[(usize, usize)],
        removed_vertex: usize,
        removed_eid: usize,
    ) -> usize {
        let mut visited = vec![false; n];
        if removed_vertex < n {
            visited[removed_vertex] = true;
        }
        let mut count = 0;
        for root in 0..n {
            if visited[root] {
                continue;
            }
            count += 1;
            visited[root] = true;
            let mut stack = vec![root];
            while let Some(u) = stack.pop() {
                for (eid, &(from, to)) in edges.iter().enumerate() {
                    if eid == removed_eid {
                        continue;
                    }
                    let v = if from == u {
                        to
                    } else if to == u {
                        from
                    } else {
                        continue;
                    };
                    if !visited[v] {
                        visited[v] = true;
                        stack.push(v);
                    }
                }
            }
        }
        count
    }

    #[test]
    fn test_low_link() {
        const Q: usize = 1_000;
        const N: usize = 8;
        const M: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N, m: 0..=M, edges: [(0..n, 0..n); m]);
            let component_count = components(n, &edges, !0, !0);
            let expected_articulation: Vec<_> = (0..n)
                .filter(|&vertex| components(n, &edges, vertex, !0) > component_count)
                .collect();
            let mut expected_bridges: Vec<_> = edges
                .iter()
                .enumerate()
                .filter(|&(eid, _)| components(n, &edges, !0, eid) > component_count)
                .map(|(_, &(u, v))| (u.min(v), u.max(v)))
                .collect();
            expected_bridges.sort_unstable();

            let graph = UndirectedSparseGraph::from_edges(n, edges);
            let mut low_link = LowLink::new(&graph);
            low_link.articulation.sort_unstable();
            low_link.bridge.sort_unstable();

            assert_eq!(low_link.articulation, expected_articulation);
            assert_eq!(low_link.bridge, expected_bridges);
        }
    }
}
