use super::{CartesianTree, Graph, Monoid, UndirectedSparseGraph};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct HeavyLightDecomposition {
    nodes: Vec<HeavyLightNode>,
    order: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct HeavyLightNode {
    parent: u32,
    size: u32,
    head: u32,
    index: u32,
}

impl UndirectedSparseGraph {
    pub fn hld(&self, root: usize) -> HeavyLightDecomposition {
        HeavyLightDecomposition::new(root, self)
    }
}

impl HeavyLightDecomposition {
    pub fn new(root: usize, graph: &UndirectedSparseGraph) -> Self {
        let n = graph.vertices_size();
        assert!(n <= u32::MAX as usize);
        let mut self_ = Self {
            nodes: vec![
                HeavyLightNode {
                    parent: n as u32,
                    size: 1,
                    head: n as u32,
                    index: 0
                };
                n
            ],
            order: Vec::with_capacity(n),
        };
        self_.order.push(root);
        for i in 0..n {
            let u = self_.order[i];
            for a in graph.neighbors(u) {
                if a.to != self_.nodes[u].parent as usize {
                    self_.nodes[a.to].parent = u as u32;
                    self_.order.push(a.to);
                }
            }
        }
        for &u in self_.order.iter().skip(1).rev() {
            let p = self_.nodes[u].parent as usize;
            self_.nodes[p].size += self_.nodes[u].size;
            let heavy = self_.nodes[p].head as usize;
            if heavy == n || self_.nodes[heavy].size <= self_.nodes[u].size {
                self_.nodes[p].head = u as u32;
            }
        }
        self_.order.clear();
        let mut stack = vec![root];
        while let Some(head) = stack.pop() {
            let chain_start = self_.order.len() as u32;
            let chain_parent = self_.nodes[head].parent;
            let mut u = head;
            while u != n {
                let heavy = self_.nodes[u].head as usize;
                let parent = self_.nodes[u].parent as usize;
                self_.nodes[u].head = chain_start;
                self_.nodes[u].parent = chain_parent;
                self_.nodes[u].index = self_.order.len() as u32;
                self_.order.push(u);
                for a in graph.neighbors(u).rev() {
                    if a.to != parent && a.to != heavy {
                        stack.push(a.to);
                    }
                }
                u = heavy;
            }
        }
        self_
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
        let index = self.nodes[v].index as usize;
        if index == self.nodes[v].head as usize {
            ((self.nodes[v].parent as usize) < self.len()).then_some(self.nodes[v].parent as usize)
        } else {
            Some(self.order[index - 1])
        }
    }

    #[inline]
    pub fn index(&self, v: usize) -> usize {
        self.nodes[v].index as usize
    }

    #[inline]
    pub fn vertex(&self, index: usize) -> usize {
        self.order[index]
    }

    #[inline]
    pub fn subtree_size(&self, v: usize) -> usize {
        self.nodes[v].size as usize
    }

    #[inline]
    pub fn subtree_range(&self, v: usize) -> Range<usize> {
        self.nodes[v].index as usize..self.nodes[v].index as usize + self.nodes[v].size as usize
    }

    #[inline]
    pub fn is_ancestor(&self, ancestor: usize, v: usize) -> bool {
        self.subtree_range(ancestor)
            .contains(&(self.nodes[v].index as usize))
    }

    #[inline]
    pub fn kth_ancestor(&self, mut v: usize, mut k: usize) -> Option<usize> {
        loop {
            let head = self.nodes[v].head as usize;
            let chain_len = self.nodes[v].index as usize - head;
            if k <= chain_len {
                return Some(self.order[self.nodes[v].index as usize - k]);
            }
            k -= chain_len + 1;
            v = self.nodes[v].parent as usize;
            if v == self.len() {
                return None;
            }
        }
    }

    #[inline]
    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        while self.nodes[u].head != self.nodes[v].head {
            if self.nodes[u].index > self.nodes[v].index {
                u = self.nodes[u].parent as usize;
            } else {
                v = self.nodes[v].parent as usize;
            }
        }
        if self.nodes[u].index < self.nodes[v].index {
            u
        } else {
            v
        }
    }

    #[inline]
    pub fn distance(&self, u: usize, v: usize) -> usize {
        let (up, down) = self.path_lengths(u, v);
        up + down
    }

    #[inline]
    pub fn jump(&self, u: usize, v: usize, mut k: usize) -> Option<usize> {
        let target = v;
        let mut u = self.nodes[u];
        let mut v = self.nodes[v];
        let mut down = 0;
        while u.head != v.head {
            if u.index > v.index {
                let up = u.index as usize - u.head as usize + 1;
                if k < up {
                    return Some(self.order[u.index as usize - k]);
                }
                k -= up;
                u = self.nodes[u.parent as usize];
            } else {
                down += v.index as usize - v.head as usize + 1;
                v = self.nodes[v.parent as usize];
            }
        }
        if u.index >= v.index {
            let up = u.index as usize - v.index as usize;
            if k <= up {
                return Some(self.order[u.index as usize - k]);
            }
            k -= up;
        } else {
            down += v.index as usize - u.index as usize;
        }
        down.checked_sub(k)
            .and_then(|k| self.kth_ancestor(target, k))
    }

    #[inline]
    fn path_lengths(&self, mut u: usize, mut v: usize) -> (usize, usize) {
        let (mut up, mut down) = (0, 0);
        while self.nodes[u].head != self.nodes[v].head {
            if self.nodes[u].index > self.nodes[v].index {
                up += self.nodes[u].index as usize - self.nodes[u].head as usize + 1;
                u = self.nodes[u].parent as usize;
            } else {
                down += self.nodes[v].index as usize - self.nodes[v].head as usize + 1;
                v = self.nodes[v].parent as usize;
            }
        }
        if self.nodes[u].index > self.nodes[v].index {
            up += self.nodes[u].index as usize - self.nodes[v].index as usize;
        } else {
            down += self.nodes[v].index as usize - self.nodes[u].index as usize;
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
            if self.nodes[u].index > self.nodes[v].index {
                std::mem::swap(&mut u, &mut v);
            }
            if self.nodes[u].head == self.nodes[v].head {
                break;
            }
            f(
                self.nodes[v].head as usize,
                self.nodes[v].index as usize + 1,
            );
            v = self.nodes[v].parent as usize;
        }
        let l = self.nodes[u].index as usize + usize::from(is_edge);
        let r = self.nodes[v].index as usize + 1;
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
        while self.nodes[u].head != self.nodes[v].head {
            if self.nodes[u].index > self.nodes[v].index {
                left = M::operate(
                    &left,
                    &reverse(
                        self.nodes[u].head as usize,
                        self.nodes[u].index as usize + 1,
                    ),
                );
                u = self.nodes[u].parent as usize;
            } else {
                right = M::operate(
                    &forward(
                        self.nodes[v].head as usize,
                        self.nodes[v].index as usize + 1,
                    ),
                    &right,
                );
                v = self.nodes[v].parent as usize;
            }
        }
        let middle = if self.nodes[u].index > self.nodes[v].index {
            reverse(
                self.nodes[v].index as usize + usize::from(is_edge),
                self.nodes[u].index as usize + 1,
            )
        } else {
            forward(
                self.nodes[u].index as usize + usize::from(is_edge),
                self.nodes[v].index as usize + 1,
            )
        };
        M::operate(&M::operate(&left, &middle), &right)
    }
}

pub struct HeavyLightPathFold<'a, M: Monoid> {
    tree: &'a HeavyLightDecomposition,
    nodes: Vec<PathFoldNode<M::T>>,
}

struct PathFoldNode<T> {
    parent: u32,
    children: [u32; 2],
    priority: u32,
    value: T,
    aggregate: [T; 2],
    prefix: [T; 2],
}

impl HeavyLightDecomposition {
    /// `values` is indexed by vertex, not by DFS index.
    pub fn build_fold<M: Monoid>(&self, values: &[M::T]) -> HeavyLightPathFold<'_, M> {
        assert_eq!(values.len(), self.len());
        let mut fold = HeavyLightPathFold {
            tree: self,
            nodes: self
                .order
                .iter()
                .map(|&v| PathFoldNode {
                    parent: u32::MAX,
                    children: [u32::MAX; 2],
                    priority: 0,
                    value: values[v].clone(),
                    aggregate: [values[v].clone(), values[v].clone()],
                    prefix: [values[v].clone(), values[v].clone()],
                })
                .collect(),
        };
        let mut start = 0;
        let mut priorities = Vec::new();
        let mut stack = Vec::new();
        while start < self.len() {
            let mut end = start + 1;
            while end < self.len() && self.nodes[self.order[end]].head as usize == start {
                end += 1;
            }
            priorities.clear();
            let mut sum = 0usize;
            for i in start..end {
                let weight = self.subtree_size(self.order[i])
                    - if i + 1 < end {
                        self.subtree_size(self.order[i + 1])
                    } else {
                        0
                    };
                let priority = (sum ^ (sum + weight)).ilog2();
                sum += weight;
                fold.nodes[i].priority = priority;
                priorities.push(std::cmp::Reverse(priority));
            }
            let cartesian = CartesianTree::new(&priorities);
            for i in start..end {
                let parent = cartesian.parents[i - start];
                fold.nodes[i].parent = if parent == usize::MAX {
                    u32::MAX
                } else {
                    (parent + start) as u32
                };
                fold.nodes[i].children = cartesian.children[i - start].map(|v| {
                    if v == usize::MAX {
                        u32::MAX
                    } else {
                        (v + start) as u32
                    }
                });
            }
            stack.clear();
            stack.push(cartesian.root + start);
            let mut i = 0;
            while i < stack.len() {
                stack.extend(
                    fold.nodes[stack[i]]
                        .children
                        .into_iter()
                        .filter(|&v| v != u32::MAX)
                        .map(|v| v as usize),
                );
                i += 1;
            }
            for &i in stack.iter().rev() {
                fold.pull(i);
            }
            start = end;
        }
        fold
    }
}

impl<M: Monoid> HeavyLightPathFold<'_, M> {
    #[inline(always)]
    fn pull(&mut self, i: usize) {
        let [l, r] = self.nodes[i].children.map(|v| {
            if v == u32::MAX {
                usize::MAX
            } else {
                v as usize
            }
        });
        self.nodes[i].prefix = if l == usize::MAX {
            [self.nodes[i].value.clone(), self.nodes[i].value.clone()]
        } else {
            [
                M::operate(&self.nodes[l].aggregate[0], &self.nodes[i].value),
                M::operate(&self.nodes[i].value, &self.nodes[l].aggregate[1]),
            ]
        };
        self.nodes[i].aggregate = if r == usize::MAX {
            self.nodes[i].prefix.clone()
        } else {
            [
                M::operate(&self.nodes[i].prefix[0], &self.nodes[r].aggregate[0]),
                M::operate(&self.nodes[r].aggregate[1], &self.nodes[i].prefix[1]),
            ]
        };
    }

    pub fn set(&mut self, vertex: usize, value: M::T) {
        let mut i = self.tree.index(vertex);
        self.nodes[i].value = value;
        while i != usize::MAX {
            self.pull(i);
            i = if self.nodes[i].parent == u32::MAX {
                usize::MAX
            } else {
                self.nodes[i].parent as usize
            };
        }
    }

    fn fold_prefix<const REVERSE: bool>(&self, k: usize) -> M::T {
        let mut result = M::unit();
        let mut i = k;
        while i != usize::MAX {
            if i <= k {
                result = if REVERSE {
                    M::operate(&result, &self.nodes[i].prefix[1])
                } else {
                    M::operate(&self.nodes[i].prefix[0], &result)
                };
            }
            i = if self.nodes[i].parent == u32::MAX {
                usize::MAX
            } else {
                self.nodes[i].parent as usize
            };
        }
        result
    }

    fn fold_range<const REVERSE: bool>(&self, l: usize, r: usize) -> M::T {
        let (mut u, mut v) = (l, r);
        let (mut left, mut right) = (M::unit(), M::unit());
        while u != v {
            if self.nodes[u].priority < self.nodes[v].priority {
                if u >= l {
                    let child = self.nodes[u].children[1];
                    if REVERSE {
                        left = M::operate(&self.nodes[u].value, &left);
                        if child != u32::MAX {
                            left = M::operate(&self.nodes[child as usize].aggregate[1], &left);
                        }
                    } else {
                        left = M::operate(&left, &self.nodes[u].value);
                        if child != u32::MAX {
                            left = M::operate(&left, &self.nodes[child as usize].aggregate[0]);
                        }
                    }
                }
                u = self.nodes[u].parent as usize;
            } else {
                if v <= r {
                    right = if REVERSE {
                        M::operate(&right, &self.nodes[v].prefix[1])
                    } else {
                        M::operate(&self.nodes[v].prefix[0], &right)
                    };
                }
                v = self.nodes[v].parent as usize;
            }
        }
        if REVERSE {
            M::operate(&M::operate(&right, &self.nodes[u].value), &left)
        } else {
            M::operate(&M::operate(&left, &self.nodes[u].value), &right)
        }
    }

    /// Folds the vertex values in `u`-to-`v` order.
    #[inline(always)]
    pub fn fold_vertices(&self, mut u: usize, mut v: usize) -> M::T {
        let (mut left, mut right) = (M::unit(), M::unit());
        while self.tree.nodes[u].head != self.tree.nodes[v].head {
            if self.tree.index(u) > self.tree.index(v) {
                left = M::operate(&left, &self.fold_prefix::<true>(self.tree.index(u)));
                u = self.tree.nodes[u].parent as usize;
            } else {
                right = M::operate(&self.fold_prefix::<false>(self.tree.index(v)), &right);
                v = self.tree.nodes[v].parent as usize;
            }
        }
        let middle = if self.tree.index(u) > self.tree.index(v) {
            self.fold_range::<true>(self.tree.index(v), self.tree.index(u))
        } else {
            self.fold_range::<false>(self.tree.index(u), self.tree.index(v))
        };
        M::operate(&M::operate(&left, &middle), &right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::ConcatenateOperation,
        tools::{Xorshift, testutil::exhaustive_sequences},
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

        let mut fold =
            hld.build_fold::<ConcatenateOperation<_>>(&(0..n).map(|v| vec![v]).collect::<Vec<_>>());
        for u in 0..n {
            fold.set(u, vec![u + n]);
            for v in 0..n {
                let expected = path(u, v, &parent, &depth);
                assert_eq!(
                    fold.fold_vertices(u, v),
                    expected
                        .iter()
                        .map(|&w| if w <= u { w + n } else { w })
                        .collect::<Vec<_>>()
                );
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
        for n in 1..=5 {
            for parents in exhaustive_sequences(0..n, n - 1..=n - 1) {
                if parents.iter().enumerate().all(|(i, &p)| p <= i) {
                    let edges: Vec<_> = parents
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| (p, i + 1))
                        .collect();
                    for root in 0..n {
                        verify(UndirectedSparseGraph::from_edges(n, edges.clone()), root);
                    }
                }
            }
        }
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
