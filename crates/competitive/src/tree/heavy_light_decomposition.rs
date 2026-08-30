use super::{Graph, Monoid, UndirectedSparseGraph};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct HeavyLightDecomposition {
    parent: Vec<usize>,
    size: Vec<usize>,
    head: Vec<usize>,
    index: Vec<usize>,
    order: Vec<usize>,
}

impl UndirectedSparseGraph {
    pub fn hld(&self, root: usize) -> HeavyLightDecomposition {
        HeavyLightDecomposition::new(root, self)
    }
}

impl HeavyLightDecomposition {
    pub fn new(root: usize, graph: &UndirectedSparseGraph) -> Self {
        let n = graph.vertices_size();
        let mut self_ = Self {
            parent: vec![n; n],
            size: vec![0; n],
            head: vec![n; n],
            index: vec![0; n],
            order: Vec::with_capacity(n),
        };
        self_.dfs_size(root, n, graph);
        self_.dfs_hld(root, n, root, graph);
        self_
    }

    fn dfs_size(&mut self, u: usize, p: usize, graph: &UndirectedSparseGraph) {
        self.parent[u] = p;
        self.size[u] = 1;
        for a in graph.neighbors(u) {
            if a.to != p {
                self.dfs_size(a.to, u, graph);
                self.size[u] += self.size[a.to];
                if self.head[u] == graph.vertices_size()
                    || self.size[self.head[u]] < self.size[a.to]
                {
                    self.head[u] = a.to;
                }
            }
        }
    }

    fn dfs_hld(&mut self, u: usize, p: usize, head: usize, graph: &UndirectedSparseGraph) {
        let heavy = self.head[u];
        self.head[u] = head;
        self.index[u] = self.order.len();
        self.order.push(u);
        if heavy != graph.vertices_size() {
            self.dfs_hld(heavy, u, head, graph);
        }
        for a in graph.neighbors(u) {
            if a.to != p && a.to != heavy {
                self.dfs_hld(a.to, u, a.to, graph);
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.order.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    #[inline]
    pub fn root(&self) -> usize {
        self.order[0]
    }

    #[inline]
    pub fn parent(&self, v: usize) -> Option<usize> {
        (self.parent[v] < self.len()).then_some(self.parent[v])
    }

    #[inline]
    pub fn index(&self, v: usize) -> usize {
        self.index[v]
    }

    #[inline]
    pub fn vertex(&self, index: usize) -> usize {
        self.order[index]
    }

    #[inline]
    pub fn subtree_size(&self, v: usize) -> usize {
        self.size[v]
    }

    #[inline]
    pub fn subtree_range(&self, v: usize) -> Range<usize> {
        self.index[v]..self.index[v] + self.size[v]
    }

    #[inline]
    pub fn is_ancestor(&self, ancestor: usize, v: usize) -> bool {
        self.subtree_range(ancestor).contains(&self.index[v])
    }

    #[inline]
    pub fn kth_ancestor(&self, mut v: usize, mut k: usize) -> Option<usize> {
        loop {
            let head = self.head[v];
            let chain_len = self.index[v] - self.index[head];
            if k <= chain_len {
                return Some(self.order[self.index[v] - k]);
            }
            k -= chain_len + 1;
            v = self.parent(head)?;
        }
    }

    #[inline]
    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        while self.head[u] != self.head[v] {
            if self.index[self.head[u]] > self.index[self.head[v]] {
                u = self.parent[self.head[u]];
            } else {
                v = self.parent[self.head[v]];
            }
        }
        if self.index[u] < self.index[v] { u } else { v }
    }

    #[inline]
    pub fn distance(&self, u: usize, v: usize) -> usize {
        let (up, down) = self.path_lengths(u, v);
        up + down
    }

    #[inline]
    pub fn jump(&self, u: usize, v: usize, k: usize) -> Option<usize> {
        let (up, down) = self.path_lengths(u, v);
        if k <= up {
            self.kth_ancestor(u, k)
        } else if k <= up + down {
            self.kth_ancestor(v, up + down - k)
        } else {
            None
        }
    }

    #[inline]
    fn path_lengths(&self, mut u: usize, mut v: usize) -> (usize, usize) {
        let (mut up, mut down) = (0, 0);
        while self.head[u] != self.head[v] {
            if self.index[u] > self.index[v] {
                up += self.index[u] - self.index[self.head[u]] + 1;
                u = self.parent[self.head[u]];
            } else {
                down += self.index[v] - self.index[self.head[v]] + 1;
                v = self.parent[self.head[v]];
            }
        }
        if self.index[u] > self.index[v] {
            up += self.index[u] - self.index[v];
        } else {
            down += self.index[v] - self.index[u];
        }
        (up, down)
    }

    /// Calls `f` once for each nonempty DFS-index range on the vertex path.
    /// The callback order is unspecified.
    #[inline]
    pub fn path_vertices<F: FnMut(usize, usize)>(&self, u: usize, v: usize, f: F) {
        self.path(u, v, false, f);
    }

    /// Calls `f` once for each nonempty DFS-index range on the edge path.
    /// Each index represents the deeper endpoint of an edge. The callback order is unspecified.
    #[inline]
    pub fn path_edges<F: FnMut(usize, usize)>(&self, u: usize, v: usize, f: F) {
        self.path(u, v, true, f);
    }

    #[inline]
    fn path<F: FnMut(usize, usize)>(&self, mut u: usize, mut v: usize, is_edge: bool, mut f: F) {
        loop {
            if self.index[u] > self.index[v] {
                std::mem::swap(&mut u, &mut v);
            }
            if self.head[u] == self.head[v] {
                break;
            }
            f(self.index[self.head[v]], self.index[v] + 1);
            v = self.parent[self.head[v]];
        }
        let l = self.index[u] + usize::from(is_edge);
        let r = self.index[v] + 1;
        if l < r {
            f(l, r);
        }
    }

    /// Folds a vertex path in `u`-to-`v` order.
    /// `forward` folds a DFS-index range from left to right, and `reverse` folds it from right to
    /// left.
    #[inline]
    pub fn fold_vertices<
        M: Monoid,
        F1: FnMut(usize, usize) -> M::T,
        F2: FnMut(usize, usize) -> M::T,
    >(
        &self,
        u: usize,
        v: usize,
        forward: F1,
        reverse: F2,
    ) -> M::T {
        self.fold::<M, _, _>(u, v, false, forward, reverse)
    }

    /// Folds an edge path in `u`-to-`v` order.
    /// Each index represents the deeper endpoint of an edge. `forward` folds a DFS-index range
    /// from left to right, and `reverse` folds it from right to left.
    #[inline]
    pub fn fold_edges<
        M: Monoid,
        F1: FnMut(usize, usize) -> M::T,
        F2: FnMut(usize, usize) -> M::T,
    >(
        &self,
        u: usize,
        v: usize,
        forward: F1,
        reverse: F2,
    ) -> M::T {
        self.fold::<M, _, _>(u, v, true, forward, reverse)
    }

    #[inline]
    fn fold<M: Monoid, F1: FnMut(usize, usize) -> M::T, F2: FnMut(usize, usize) -> M::T>(
        &self,
        mut u: usize,
        mut v: usize,
        is_edge: bool,
        mut forward: F1,
        mut reverse: F2,
    ) -> M::T {
        let (mut left, mut right) = (M::unit(), M::unit());
        while self.head[u] != self.head[v] {
            if self.index[u] > self.index[v] {
                left = M::operate(&left, &reverse(self.index[self.head[u]], self.index[u] + 1));
                u = self.parent[self.head[u]];
            } else {
                right = M::operate(
                    &forward(self.index[self.head[v]], self.index[v] + 1),
                    &right,
                );
                v = self.parent[self.head[v]];
            }
        }
        let middle = if self.index[u] > self.index[v] {
            reverse(self.index[v] + usize::from(is_edge), self.index[u] + 1)
        } else {
            forward(self.index[u] + usize::from(is_edge), self.index[v] + 1)
        };
        M::operate(&M::operate(&left, &middle), &right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::ConcatenateOperation,
        tools::Xorshift,
        tree::{MixedTree, PathTree, StarTree},
    };

    fn parent_and_depth(graph: &UndirectedSparseGraph, root: usize) -> (Vec<usize>, Vec<usize>) {
        let n = graph.vertices_size();
        let mut parent = vec![n; n];
        let mut depth = vec![0; n];
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            for a in graph.neighbors(u) {
                if a.to != parent[u] {
                    parent[a.to] = u;
                    depth[a.to] = depth[u] + 1;
                    stack.push(a.to);
                }
            }
        }
        (parent, depth)
    }

    fn path(mut u: usize, mut v: usize, parent: &[usize], depth: &[usize]) -> Vec<usize> {
        let mut left = vec![];
        let mut right = vec![];
        while depth[u] > depth[v] {
            left.push(u);
            u = parent[u];
        }
        while depth[v] > depth[u] {
            right.push(v);
            v = parent[v];
        }
        while u != v {
            left.push(u);
            right.push(v);
            u = parent[u];
            v = parent[v];
        }
        left.push(u);
        left.extend(right.into_iter().rev());
        left
    }

    fn verify(graph: UndirectedSparseGraph, root: usize) {
        let n = graph.vertices_size();
        let (parent, depth) = parent_and_depth(&graph, root);
        let hld = graph.hld(root);
        assert_eq!(hld.len(), n);
        assert_eq!(hld.root(), root);

        let mut vertex = vec![0; n];
        for v in 0..n {
            vertex[hld.index(v)] = v;
        }

        for v in 0..n {
            assert_eq!(hld.vertex(hld.index(v)), v);
            assert_eq!(hld.parent(v), (parent[v] < n).then_some(parent[v]));
            for k in 0..=depth[v] + 1 {
                let mut ancestor = Some(v);
                for _ in 0..k {
                    ancestor = ancestor.and_then(|u| (parent[u] < n).then_some(parent[u]));
                }
                assert_eq!(hld.kth_ancestor(v, k), ancestor);
            }

            let expected: Vec<_> = (0..n)
                .filter(|&u| {
                    let mut u = u;
                    while depth[u] > depth[v] {
                        u = parent[u];
                    }
                    u == v
                })
                .collect();
            let range = hld.subtree_range(v);
            let mut actual: Vec<_> = range.clone().map(|i| vertex[i]).collect();
            actual.sort_unstable();
            assert_eq!(actual, expected);
            assert_eq!(hld.subtree_size(v), range.len());
            for u in 0..n {
                assert_eq!(hld.is_ancestor(v, u), expected.contains(&u));
            }
        }

        for u in 0..n {
            for v in 0..n {
                let expected = path(u, v, &parent, &depth);
                let lca = *expected.iter().min_by_key(|&&v| depth[v]).unwrap();
                assert_eq!(hld.lca(u, v), lca);
                assert_eq!(hld.distance(u, v), expected.len() - 1);
                for k in 0..=expected.len() {
                    assert_eq!(hld.jump(u, v, k), expected.get(k).copied());
                }

                let actual = hld.fold_vertices::<ConcatenateOperation<_>, _, _>(
                    u,
                    v,
                    |l, r| (l..r).map(|i| vertex[i]).collect(),
                    |l, r| (l..r).rev().map(|i| vertex[i]).collect(),
                );
                assert_eq!(actual, expected);

                let mut actual = vec![];
                hld.path_vertices(u, v, |l, r| {
                    actual.extend((l..r).map(|i| vertex[i]));
                });
                actual.sort_unstable();
                let mut expected_unordered = expected.clone();
                expected_unordered.sort_unstable();
                assert_eq!(actual, expected_unordered);

                let expected_edges: Vec<_> =
                    expected.iter().copied().filter(|&v| v != lca).collect();
                let actual = hld.fold_edges::<ConcatenateOperation<_>, _, _>(
                    u,
                    v,
                    |l, r| (l..r).map(|i| vertex[i]).collect(),
                    |l, r| (l..r).rev().map(|i| vertex[i]).collect(),
                );
                assert_eq!(actual, expected_edges);
            }
        }
    }

    #[test]
    fn heavy_light_decomposition_against_naive() {
        let mut rng = Xorshift::default();
        for n in 1..=20 {
            verify(rng.random(PathTree(n)), rng.random(0..n));
            verify(rng.random(StarTree(n)), rng.random(0..n));
        }
        for _ in 0..100 {
            let n = rng.random(1..=40);
            verify(rng.random(MixedTree(n)), rng.random(0..n));
        }
    }
}
