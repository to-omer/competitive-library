use super::UndirectedSparseGraph;

pub struct LevelAncestor {
    parent: Vec<usize>,
    depth: Vec<usize>,
    start: Vec<usize>,
    index: Vec<usize>,
    ladder: Vec<usize>,
}

impl UndirectedSparseGraph {
    pub fn level_ancestor(&self, root: usize) -> LevelAncestor {
        let n = self.vertices_size();
        let (order, parent) = self.tree_order(root);
        let mut depth = vec![0; n];
        for &u in order.iter().skip(1) {
            depth[u] = depth[parent[u]] + 1;
        }
        let mut height = vec![1; n];
        let mut heavy = vec![n; n];
        for &u in order.iter().skip(1).rev() {
            let p = parent[u];
            if heavy[p] == n || height[heavy[p]] < height[u] {
                heavy[p] = u;
            }
            height[p] = height[p].max(height[u] + 1);
        }

        let mut start = vec![0; n];
        let mut index = vec![0; n];
        let mut ladder = Vec::with_capacity(2 * n);
        for &head in &order {
            if head != root && heavy[parent[head]] == head {
                continue;
            }
            let extension = height[head].min(depth[head]);
            let offset = ladder.len();
            ladder.resize(offset + extension + height[head], n);
            let mut u = head;
            for i in (0..extension).rev() {
                u = parent[u];
                ladder[offset + i] = u;
            }
            let mut u = head;
            for i in extension..extension + height[head] {
                ladder[offset + i] = u;
                start[u] = offset;
                index[u] = offset + i;
                u = heavy[u];
            }
        }

        LevelAncestor {
            parent,
            depth,
            start,
            index,
            ladder,
        }
    }

    pub fn level_ancestor_batch(
        &self,
        root: usize,
        queries: impl IntoIterator<Item = (usize, usize)>,
    ) -> Vec<Option<usize>> {
        let n = self.vertices_size();
        let mut start = vec![0; n + 1];
        let queries: Vec<(usize, usize)> = queries.into_iter().collect();
        for &(u, _) in &queries {
            start[u] += 1;
        }
        for d in 0..n {
            start[d + 1] += start[d];
        }
        let qsize = queries.len();
        let mut batch = vec![(0, 0); qsize];
        for (i, &(u, k)) in queries.iter().enumerate() {
            start[u] -= 1;
            batch[start[u]] = (k, i);
        }
        let (order, parent) = self.tree_order(root);
        let mut path = Vec::with_capacity(n);
        let mut results = vec![None; qsize];
        for u in order {
            while path.last().is_some_and(|&v| v != parent[u]) {
                path.pop();
            }
            path.push(u);
            for &(k, qi) in &batch[start[u]..start[u + 1]] {
                let depth = path.len() - 1;
                if k <= depth {
                    results[qi] = Some(path[depth - k]);
                }
            }
        }
        results
    }
}

impl LevelAncestor {
    #[inline]
    pub fn la(&self, mut u: usize, mut k: usize) -> Option<usize> {
        if self.depth[u] < k {
            return None;
        }
        loop {
            let start = self.start[u];
            let index = self.index[u];
            if k <= index - start {
                return Some(self.ladder[index - k]);
            }
            k -= index - start + 1;
            u = self.parent[self.ladder[start]];
        }
    }

    #[inline]
    pub fn depth(&self, u: usize) -> usize {
        self.depth[u]
    }
}

#[cfg(test)]
mod tests {
    use crate::{graph::Graph, tools::Xorshift, tree::MixedTree};

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
                for a in tree.neighbors(u) {
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
