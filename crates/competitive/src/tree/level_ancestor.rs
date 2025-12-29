use super::UndirectedSparseGraph;

pub struct LevelAncestor {
    vidx: Vec<usize>,
    inv_vidx: Vec<usize>,
    depth: Vec<usize>,
    start: Vec<usize>,
    bucket: Vec<usize>,
}

struct LevelAncestorBatch<'a> {
    tree: &'a UndirectedSparseGraph,
    path: Vec<usize>,
    queries: Vec<Vec<(usize, usize)>>,
    results: Vec<Option<usize>>,
}

impl UndirectedSparseGraph {
    pub fn level_ancestor(&self, root: usize) -> LevelAncestor {
        let n = self.vertices_size();
        let mut vidx = vec![0; n];
        let mut inv_vidx = vec![0; n];
        let mut depth = vec![0; n];
        let mut start = vec![0; n + 1];
        let mut bucket = vec![0; n];
        let mut stack = Vec::with_capacity(n);
        stack.push((root, !0));
        let mut idx = 0usize;
        while let Some((u, p)) = stack.pop() {
            vidx[u] = idx;
            inv_vidx[idx] = u;
            idx += 1;
            start[depth[u]] += 1;
            for a in self.adjacencies(u) {
                if a.to != p {
                    depth[a.to] = depth[u] + 1;
                    stack.push((a.to, u));
                }
            }
        }
        for d in 0..n {
            start[d + 1] += start[d];
        }
        for &u in &inv_vidx {
            start[depth[u]] -= 1;
            bucket[start[depth[u]]] = vidx[u];
        }

        LevelAncestor {
            vidx,
            inv_vidx,
            depth,
            start,
            bucket,
        }
    }

    pub fn level_ancestor_batch(
        &self,
        root: usize,
        queries: impl IntoIterator<Item = (usize, usize)>,
    ) -> Vec<Option<usize>> {
        let n = self.vertices_size();
        let mut batch = vec![vec![]; n];
        for (i, (u, k)) in queries.into_iter().enumerate() {
            batch[u].push((k, i));
        }
        let qsize: usize = batch.iter().map(|v| v.len()).sum();
        let mut la = LevelAncestorBatch {
            tree: self,
            path: Vec::with_capacity(n),
            queries: batch,
            results: vec![None; qsize],
        };
        la.dfs(root, !0);
        la.results
    }
}

impl LevelAncestor {
    pub fn la(&self, u: usize, k: usize) -> Option<usize> {
        if self.depth[u] < k {
            return None;
        }
        let d = self.depth[u] - k;
        let slice = &self.bucket[self.start[d]..self.start[d + 1]];
        let idx = slice.partition_point(|&v| v > self.vidx[u]);
        Some(self.inv_vidx[slice[idx]])
    }
}

impl<'a> LevelAncestorBatch<'a> {
    fn dfs(&mut self, u: usize, p: usize) {
        self.path.push(u);
        for &(k, qi) in &self.queries[u] {
            let depth = self.path.len() - 1;
            if k <= depth {
                self.results[qi] = Some(self.path[depth - k]);
            }
        }
        for a in self.tree.adjacencies(u) {
            if a.to != p {
                self.dfs(a.to, u);
            }
        }
        self.path.pop();
    }
}

#[cfg(test)]
mod tests {
    use crate::{tools::Xorshift, tree::MixedTree};

    #[test]
    fn test_level_ancestor() {
        let mut rng = Xorshift::default();
        for _ in 0..500 {
            let n = rng.random(1..=200);
            let tree = rng.random(MixedTree(n));
            let root = rng.random(0..n);
            let la = tree.level_ancestor(root);
            let mut parent = vec![None; n];
            let mut stack = vec![(root, None)];
            while let Some((u, p)) = stack.pop() {
                parent[u] = p;
                for a in tree.adjacencies(u) {
                    if Some(a.to) != p {
                        stack.push((a.to, Some(u)));
                    }
                }
            }
            let mut queries = vec![];
            let mut results = vec![];
            for u in 0..n {
                let mut v = Some(u);
                for d in 0..=n {
                    assert_eq!(la.la(u, d), v);
                    queries.push((u, d));
                    results.push(v);
                    v = v.and_then(|x| parent[x]);
                }
            }
            assert_eq!(tree.level_ancestor_batch(root, queries), results);
        }
    }
}
