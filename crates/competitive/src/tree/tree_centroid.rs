use super::UndirectedSparseGraph;

impl UndirectedSparseGraph {
    pub fn tree_centroid(&self) -> usize {
        fn dfs(g: &UndirectedSparseGraph, u: usize, p: usize) -> (usize, usize) {
            let n = g.vertices_size();
            let mut size = 1;
            let mut ok = true;
            for a in g.adjacencies(u) {
                if a.to != p {
                    let (s, c) = dfs(g, a.to, u);
                    if c != !0 {
                        return (0, c);
                    }
                    ok &= s * 2 <= n;
                    size += s;
                }
            }
            ok &= (n - size) * 2 <= n;
            (size, if ok { u } else { !0 })
        }
        dfs(self, 0, !0).1
    }
}
