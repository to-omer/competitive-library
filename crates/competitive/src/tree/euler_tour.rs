use super::{RangeMinimumQuery, UndirectedSparseGraph};
use std::{marker::PhantomData, mem::swap, ops::Range};

pub trait EulerTourKind {
    const USE_LAST: bool = false;
    const USE_VISIT: bool = false;

    fn size(n: usize) -> usize {
        if Self::USE_VISIT {
            2 * n - 1
        } else if Self::USE_LAST {
            2 * n
        } else {
            n
        }
    }
}

mod marker {
    use super::EulerTourKind;

    #[derive(Debug, Clone)]
    pub enum First {}
    #[derive(Debug, Clone)]
    pub enum FirstLast {}
    #[derive(Debug, Clone)]
    pub enum Visit {}

    impl EulerTourKind for First {}
    impl EulerTourKind for FirstLast {
        const USE_LAST: bool = true;
    }
    impl EulerTourKind for Visit {
        const USE_VISIT: bool = true;
    }
}

#[derive(Debug)]
pub struct EulerTourBuilder<'a, K>
where
    K: EulerTourKind,
{
    tree: &'a UndirectedSparseGraph,
    root: usize,
    vidx: Vec<[usize; 2]>,
    eidx: Vec<[usize; 2]>,
    pos: usize,
    _marker: PhantomData<fn() -> K>,
}

#[derive(Debug, Clone)]
pub struct EulerTour<K>
where
    K: EulerTourKind,
{
    pub root: usize,
    pub vidx: Vec<[usize; 2]>,
    pub eidx: Vec<[usize; 2]>,
    pub size: usize,
    _marker: PhantomData<fn() -> K>,
}

impl<'a, K> EulerTourBuilder<'a, K>
where
    K: EulerTourKind,
{
    pub fn new(tree: &'a UndirectedSparseGraph, root: usize) -> Self {
        let n = tree.vertices_size();
        Self {
            tree,
            root,
            vidx: vec![[0usize; 2]; n],
            eidx: vec![[0usize; 2]; n - 1],
            pos: 0,
            _marker: PhantomData,
        }
    }

    pub fn build_with_trace(mut self, mut trace: impl FnMut(usize)) -> EulerTour<K> {
        self.dfs(self.root, !0, &mut trace);
        EulerTour {
            root: self.root,
            vidx: self.vidx,
            eidx: self.eidx,
            size: self.pos,
            _marker: PhantomData,
        }
    }

    pub fn build(self) -> EulerTour<K> {
        self.build_with_trace(|_u| {})
    }

    fn dfs(&mut self, u: usize, parent: usize, trace: &mut impl FnMut(usize)) {
        self.vidx[u][0] = self.pos;
        trace(u);
        self.pos += 1;
        for a in self.tree.adjacencies(u) {
            if a.to != parent {
                self.eidx[a.id][0] = self.pos;
                self.dfs(a.to, u, trace);
                self.eidx[a.id][1] = self.pos;
                if K::USE_VISIT {
                    trace(u);
                    self.pos += 1;
                }
            }
        }
        self.vidx[u][1] = self.pos;
        if K::USE_LAST {
            trace(u);
            self.pos += 1;
        }
    }
}

impl EulerTourBuilder<'_, marker::First> {
    pub fn build_with_rearrange<T>(self, s: &[T]) -> (EulerTour<marker::First>, Vec<T>)
    where
        T: Clone,
    {
        assert_eq!(s.len(), self.tree.vertices_size());
        let mut trace = Vec::with_capacity(marker::First::size(s.len()));
        let tour = self.build_with_trace(|u| {
            trace.push(s[u].clone());
        });
        (tour, trace)
    }
}

impl EulerTourBuilder<'_, marker::FirstLast> {
    pub fn build_with_rearrange<T>(
        self,
        s: &[T],
        mut inverse: impl FnMut(T) -> T,
    ) -> (EulerTour<marker::FirstLast>, Vec<T>)
    where
        T: Clone,
    {
        assert_eq!(s.len(), self.tree.vertices_size());
        let mut visited = vec![false; s.len()];
        let mut trace = Vec::with_capacity(marker::FirstLast::size(s.len()));
        let tour = self.build_with_trace(|u| {
            if !visited[u] {
                trace.push(s[u].clone());
                visited[u] = true;
            } else {
                trace.push(inverse(s[u].clone()));
            }
        });
        (tour, trace)
    }
}

impl EulerTourBuilder<'_, marker::Visit> {
    pub fn build_with_rearrange<T>(self, s: &[T]) -> (EulerTour<marker::Visit>, Vec<T>)
    where
        T: Clone,
    {
        assert_eq!(s.len(), self.tree.vertices_size());
        let mut trace = Vec::with_capacity(marker::Visit::size(s.len()));
        let tour = self.build_with_trace(|u| {
            trace.push(s[u].clone());
        });
        (tour, trace)
    }
}

impl UndirectedSparseGraph {
    pub fn subtree_euler_tour_builder<'a>(
        &'a self,
        root: usize,
    ) -> EulerTourBuilder<'a, marker::First> {
        EulerTourBuilder::new(self, root)
    }

    pub fn path_euler_tour_builder<'a>(
        &'a self,
        root: usize,
    ) -> EulerTourBuilder<'a, marker::FirstLast> {
        EulerTourBuilder::new(self, root)
    }

    pub fn full_euler_tour_builder<'a>(
        &'a self,
        root: usize,
    ) -> EulerTourBuilder<'a, marker::Visit> {
        EulerTourBuilder::new(self, root)
    }

    pub fn lca(&self, root: usize) -> LowestCommonAncestor {
        let depth = self.tree_depth(root);
        let mut trace = Vec::with_capacity(2 * self.vertices_size() - 1);
        let mut depth_trace = Vec::with_capacity(2 * self.vertices_size() - 1);
        let euler_tour = self.full_euler_tour_builder(root).build_with_trace(|u| {
            trace.push(u);
            depth_trace.push(depth[u]);
        });
        let rmq = RangeMinimumQuery::new(depth_trace);
        LowestCommonAncestor {
            euler_tour,
            trace,
            rmq,
        }
    }
}

impl EulerTour<marker::First> {
    pub fn get<T>(&self, u: usize, mut f: impl FnMut(usize) -> T) -> T {
        let [l, _] = self.vidx[u];
        f(l)
    }

    pub fn update<T>(&self, u: usize, x: T, mut f: impl FnMut(usize, T)) {
        let [l, _] = self.vidx[u];
        f(l, x);
    }

    pub fn fold<T>(&self, u: usize, mut f: impl FnMut(Range<usize>) -> T) -> T {
        let [l, r] = self.vidx[u];
        f(l..r)
    }

    pub fn range_update<T>(&self, u: usize, x: T, mut f: impl FnMut(Range<usize>, T)) {
        let [l, r] = self.vidx[u];
        f(l..r, x);
    }
}

impl EulerTour<marker::FirstLast> {
    pub fn get<T>(&self, u: usize, mut f: impl FnMut(usize) -> T) -> T {
        let [l, _] = self.vidx[u];
        f(l)
    }

    pub fn update<T>(&self, u: usize, x: T, invx: T, mut f: impl FnMut(usize, T)) {
        let [l, r] = self.vidx[u];
        f(l, x);
        f(r, invx);
    }

    // f: accumulate
    pub fn fold<T>(&self, u: usize, mut f: impl FnMut(usize) -> T) -> T {
        f(self.vidx[u][0])
    }
}

#[derive(Debug)]
pub struct LowestCommonAncestor {
    euler_tour: EulerTour<marker::Visit>,
    trace: Vec<usize>,
    rmq: RangeMinimumQuery<u64>,
}

impl LowestCommonAncestor {
    pub fn lca(&self, u: usize, v: usize) -> usize {
        let mut l = self.euler_tour.vidx[u][0];
        let mut r = self.euler_tour.vidx[v][0];
        if l > r {
            swap(&mut l, &mut r);
        }
        let idx = self.rmq.argmin(l, r + 1);
        self.trace[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, RangeSumRangeAdd},
        crecurse,
        data_structure::{LazySegmentTree, SegmentTree},
        tools::Xorshift,
        tree::MixedTree,
    };

    #[test]
    fn test_builder() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1..=200);
            let tree = rng.random(MixedTree(n));
            let root = rng.random(0..n);
            let et1 = tree.subtree_euler_tour_builder(root).build();
            let et2 = tree.path_euler_tour_builder(root).build();
            let et3 = tree.full_euler_tour_builder(root).build();
            assert_eq!(et1.size, marker::First::size(n));
            assert_eq!(et2.size, marker::FirstLast::size(n));
            assert_eq!(et3.size, marker::Visit::size(n));
            for u in 0..n {
                assert!(et1.vidx[u][0] < et1.vidx[u][1]);
                assert!(et1.vidx[u][1] <= marker::First::size(n));
                assert!(et2.vidx[u][0] < et2.vidx[u][1]);
                assert!(et2.vidx[u][1] < marker::FirstLast::size(n));
                assert!(et3.vidx[u][0] < et3.vidx[u][1]);
                assert!(et3.vidx[u][1] <= marker::Visit::size(n));
            }
        }
    }

    #[test]
    fn test_subtree_euler_tour() {
        const A: i64 = 1_000_000;
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1..=200);
            let tree = rng.random(MixedTree(n));
            let root = rng.random(0..n);
            let mut a: Vec<_> = rng.random_iter(0..A).take(n).collect();
            let (et, arr) = tree
                .subtree_euler_tour_builder(root)
                .build_with_rearrange(&a);
            let mut seg = LazySegmentTree::<RangeSumRangeAdd<i64>>::from_keys(arr.into_iter());
            for _ in 0..200 {
                match rng.random(0..4) {
                    0 => {
                        let u = rng.random(0..n);
                        let result = et.get(u, |idx| seg.get(idx)).0;
                        let expected = a[u];
                        assert_eq!(result, expected);
                    }
                    1 => {
                        let u = rng.random(0..n);
                        let x = rng.random(0..A);
                        et.update(u, x, |i, x| seg.update(i..=i, x));
                        a[u] += x;
                    }
                    2 => {
                        let u = rng.random(0..n);
                        let result = et.fold(u, |r| seg.fold(r)).0;
                        let mut expected = 0;
                        crecurse!(
                            unsafe fn dfs(v: usize, p: usize, b: bool) {
                                let b = b || v == u;
                                if b {
                                    expected += a[v];
                                }
                                for a in tree.adjacencies(v) {
                                    if a.to != p {
                                        dfs!(a.to, v, b);
                                    }
                                }
                            }
                        )(root, !0, false);
                        assert_eq!(result, expected);
                    }
                    _ => {
                        let u = rng.random(0..n);
                        let x = rng.random(0..A);
                        et.range_update(u, x, |r, x| seg.update(r, x));
                        crecurse!(
                            unsafe fn dfs(v: usize, p: usize, b: bool) {
                                let b = b || v == u;
                                if b {
                                    a[v] += x;
                                }
                                for a in tree.adjacencies(v) {
                                    if a.to != p {
                                        dfs!(a.to, v, b);
                                    }
                                }
                            }
                        )(root, !0, false);
                    }
                }
            }
        }
    }

    #[test]
    fn test_path_euler_tour() {
        const A: i64 = 1_000_000;
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1..=200);
            let tree = rng.random(MixedTree(n));
            let root = rng.random(0..n);
            let mut a: Vec<_> = rng.random_iter(0..A).take(n).collect();
            let (et, arr) = tree
                .path_euler_tour_builder(root)
                .build_with_rearrange(&a, |x| -x);
            let mut seg = SegmentTree::<AdditiveOperation<i64>>::from_vec(arr);
            for _ in 0..200 {
                match rng.random(0..3) {
                    0 => {
                        let u = rng.random(0..n);
                        let result = et.get(u, |idx| seg.get(idx));
                        let expected = a[u];
                        assert_eq!(result, expected);
                    }
                    1 => {
                        let u = rng.random(0..n);
                        let x = rng.random(0..A);
                        let invx = -x;
                        et.update(u, x, invx, |i, x| seg.update(i, x));
                        a[u] += x;
                    }
                    _ => {
                        let u = rng.random(0..n);
                        let result = et.fold(u, |k| seg.fold(0..=k));
                        let mut expected = 0;
                        crecurse!(
                            unsafe fn dfs(v: usize, p: usize) -> bool {
                                if v == u {
                                    expected += a[v];
                                    return true;
                                }
                                for adj in tree.adjacencies(v) {
                                    if adj.to != p && dfs!(adj.to, v) {
                                        expected += a[v];
                                        return true;
                                    }
                                }
                                false
                            }
                        )(root, !0);
                        assert_eq!(result, expected);
                    }
                }
            }
        }
    }

    #[test]
    fn test_lca() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n = rng.random(1..=200);
            let tree = rng.random(MixedTree(n));
            let root = rng.random(0..n);
            let lca = tree.lca(root);
            for _ in 0..200 {
                let u = rng.random(0..n);
                let v = rng.random(0..n);
                let result = lca.lca(u, v);
                let expected = crecurse!(
                    unsafe fn dfs(w: usize, p: usize) -> Result<usize, [bool; 2]> {
                        let mut found = [false; 2];
                        if w == u {
                            found[0] = true;
                        }
                        if w == v {
                            found[1] = true;
                        }
                        for adj in tree.adjacencies(w) {
                            if adj.to != p {
                                match dfs!(adj.to, w) {
                                    Ok(lca) => return Ok(lca),
                                    Err(res) => {
                                        for i in 0..2 {
                                            if res[i] {
                                                found[i] = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if found[0] && found[1] {
                            Ok(w)
                        } else {
                            Err(found)
                        }
                    }
                )(root, !0)
                .unwrap();
                assert_eq!(result, expected);
            }
        }
    }
}
