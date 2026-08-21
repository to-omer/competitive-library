use super::{Group, Monoid};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{MaybeUninit, needs_drop, swap},
};

pub struct UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<M::Data, P>>,
{
    cells: Vec<UfCell<M::Data, P>>,
    merger: M,
    history: H::History,
    _marker: PhantomData<fn() -> (U, F)>,
}

impl<U, F, M, P, H> Clone for UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec<Data: Clone> + Clone,
    P: Monoid,
    H: UndoStrategy<UfCell<M::Data, P>, History: Clone>,
{
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            merger: self.merger.clone(),
            history: self.history.clone(),
            _marker: self._marker,
        }
    }
}

impl<U, F, M, P, H> Debug for UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec<Data: Debug>,
    P: Monoid<T: Debug>,
    H: UndoStrategy<UfCell<M::Data, P>, History: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionFindBase")
            .field("cells", &self.cells)
            .field("history", &self.history)
            .finish()
    }
}

pub struct UfCell<D, P>
where
    P: Monoid,
{
    // Roots store `!info` and initialized data; children store a parent and no data.
    parent_or_info: i32,
    data: MaybeUninit<D>,
    potential: P::T,
}

impl<D, P> Clone for UfCell<D, P>
where
    D: Clone,
    P: Monoid,
{
    fn clone(&self) -> Self {
        let is_root = self.is_root();
        let potential = if is_root {
            P::unit()
        } else {
            self.potential.clone()
        };
        Self {
            parent_or_info: self.parent_or_info,
            data: if is_root {
                MaybeUninit::new(self.data().clone())
            } else {
                MaybeUninit::uninit()
            },
            potential,
        }
    }
}

impl<D, P> Debug for UfCell<D, P>
where
    D: Debug,
    P: Monoid<T: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(info) = self.root_info() {
            f.debug_tuple("Root").field(&(info, self.data())).finish()
        } else {
            f.debug_tuple("Child")
                .field(&(self.parent().unwrap(), &self.potential))
                .finish()
        }
    }
}

impl<D, P> UfCell<D, P>
where
    P: Monoid,
{
    fn root(info: u32, data: D) -> Self {
        Self {
            parent_or_info: !(info as i32),
            data: MaybeUninit::new(data),
            potential: P::unit(),
        }
    }

    fn root_info(&self) -> Option<u32> {
        (self.parent_or_info < 0).then_some((!self.parent_or_info) as u32)
    }

    fn set_root_info(&mut self, info: u32) {
        self.parent_or_info = !(info as i32);
    }

    fn parent(&self) -> Option<usize> {
        (self.parent_or_info >= 0).then_some(self.parent_or_info as usize)
    }

    fn set_child(&mut self, parent: usize, potential: P::T) {
        if needs_drop::<D>() && self.is_root() {
            unsafe { self.data.assume_init_drop() };
        }
        self.parent_or_info = parent as i32;
        self.potential = potential;
    }

    fn is_root(&self) -> bool {
        self.parent_or_info < 0
    }

    fn data(&self) -> &D {
        unsafe { self.data.assume_init_ref() }
    }

    fn data_mut(&mut self) -> &mut D {
        unsafe { self.data.assume_init_mut() }
    }
}

impl<D, P> Drop for UfCell<D, P>
where
    P: Monoid,
{
    fn drop(&mut self) {
        if needs_drop::<D>() && self.is_root() {
            unsafe { self.data.assume_init_drop() };
        }
    }
}

pub trait FindStrategy {
    const CHENGE_ROOT: bool;
}

pub enum PathCompression {}

impl FindStrategy for PathCompression {
    const CHENGE_ROOT: bool = true;
}

impl FindStrategy for () {
    const CHENGE_ROOT: bool = false;
}

pub trait UnionStrategy {
    fn single_info() -> u32;
    fn check_directoin(parent: &u32, child: &u32) -> bool;
    fn unite(parent: &u32, child: &u32) -> u32;
}

pub enum UnionBySize {}

impl UnionStrategy for UnionBySize {
    fn single_info() -> u32 {
        1
    }

    fn check_directoin(parent: &u32, child: &u32) -> bool {
        parent >= child
    }

    fn unite(parent: &u32, child: &u32) -> u32 {
        parent + child
    }
}

pub enum UnionByRank {}

impl UnionStrategy for UnionByRank {
    fn single_info() -> u32 {
        0
    }

    fn check_directoin(parent: &u32, child: &u32) -> bool {
        parent >= child
    }

    fn unite(parent: &u32, child: &u32) -> u32 {
        parent + (parent == child) as u32
    }
}

impl UnionStrategy for () {
    fn single_info() -> u32 {
        0
    }

    fn check_directoin(_parent: &u32, _child: &u32) -> bool {
        false
    }

    fn unite(_parent: &u32, _child: &u32) -> u32 {
        0
    }
}

pub trait UfMergeSpec {
    type Data;
    fn merge(&mut self, to: &mut Self::Data, from: &mut Self::Data);
}

#[derive(Debug, Clone)]
pub struct FnMerger<T, F> {
    f: F,
    _marker: PhantomData<fn() -> T>,
}

impl<T, F> UfMergeSpec for FnMerger<T, F>
where
    F: FnMut(&mut T, &mut T),
{
    type Data = T;

    fn merge(&mut self, to: &mut Self::Data, from: &mut Self::Data) {
        (self.f)(to, from)
    }
}

impl UfMergeSpec for () {
    type Data = ();

    fn merge(&mut self, _to: &mut Self::Data, _from: &mut Self::Data) {}
}

pub trait UndoStrategy<T> {
    const UNDOABLE: bool;

    type History: Default;

    fn unite(history: &mut Self::History, x: usize, y: usize, cells: &[T]);

    fn undo_unite(history: &mut Self::History, cells: &mut [T]);
}

pub enum Undoable {}

impl<T> UndoStrategy<T> for Undoable
where
    T: Clone,
{
    const UNDOABLE: bool = true;

    type History = Vec<[(usize, T); 2]>;

    fn unite(history: &mut Self::History, x: usize, y: usize, cells: &[T]) {
        let cx = cells[x].clone();
        let cy = cells[y].clone();
        history.push([(x, cx), (y, cy)]);
    }

    fn undo_unite(history: &mut Self::History, cells: &mut [T]) {
        if let Some([(x, cx), (y, cy)]) = history.pop() {
            cells[x] = cx;
            cells[y] = cy;
        }
    }
}

impl<T> UndoStrategy<T> for () {
    const UNDOABLE: bool = false;

    type History = ();

    fn unite(_history: &mut Self::History, _x: usize, _y: usize, _cells: &[T]) {}

    fn undo_unite(_history: &mut Self::History, _cells: &mut [T]) {}
}

impl<U, F, P, H> UnionFindBase<U, F, (), P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    P: Monoid,
    H: UndoStrategy<UfCell<(), P>>,
{
    pub fn new(n: usize) -> Self {
        let cells: Vec<_> = (0..n).map(|_| UfCell::root(U::single_info(), ())).collect();
        Self {
            cells,
            merger: (),
            history: Default::default(),
            _marker: PhantomData,
        }
    }
    pub fn push(&mut self) {
        self.cells.push(UfCell::root(U::single_info(), ()));
    }
}

impl<U, F, T, Merge, P, H> UnionFindBase<U, F, FnMerger<T, Merge>, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    Merge: FnMut(&mut T, &mut T),
    P: Monoid,
    H: UndoStrategy<UfCell<T, P>>,
{
    pub fn new_with_merger(n: usize, mut init: impl FnMut(usize) -> T, merge: Merge) -> Self {
        let cells: Vec<_> = (0..n)
            .map(|i| UfCell::root(U::single_info(), init(i)))
            .collect();
        Self {
            cells,
            merger: FnMerger {
                f: merge,
                _marker: PhantomData,
            },
            history: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<F, M, P, H> UnionFindBase<UnionBySize, F, M, P, H>
where
    F: FindStrategy,
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<M::Data, P>>,
{
    pub fn size(&mut self, x: usize) -> usize {
        let root = self.find_root(x);
        self.root_info(root).unwrap() as usize
    }
}

impl<U, F, M, P, H> UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<M::Data, P>>,
{
    fn root_info(&self, x: usize) -> Option<u32> {
        self.cells[x].root_info()
    }

    fn set_root_info(&mut self, x: usize, info: u32) {
        self.cells[x].set_root_info(info);
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find_root(x) == self.find_root(y)
    }

    pub fn merge_data(&mut self, x: usize) -> &M::Data {
        let root = self.find_root(x);
        self.cells[root].data()
    }

    pub fn merge_data_mut(&mut self, x: usize) -> &mut M::Data {
        let root = self.find_root(x);
        self.cells[root].data_mut()
    }

    pub fn roots(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.cells.len()).filter(|&x| self.cells[x].is_root())
    }

    pub fn all_group_members(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut groups_map = HashMap::new();
        for x in 0..self.cells.len() {
            let r = self.find_root(x);
            groups_map.entry(r).or_insert_with(Vec::new).push(x);
        }
        groups_map
    }

    pub fn find(&mut self, x: usize) -> (usize, P::T) {
        let mut current = x;
        let mut potential = P::unit();
        while let Some(parent) = self.cells[current].parent() {
            let current_potential = self.cells[current].potential.clone();
            potential = P::operate(&current_potential, &potential);
            if F::CHENGE_ROOT
                && let Some(parent_parent) = self.cells[parent].parent()
            {
                let potential = P::operate(&self.cells[parent].potential, &current_potential);
                self.cells[current].set_child(parent_parent, potential);
            }
            current = parent;
        }
        (current, potential)
    }

    pub fn find_root(&mut self, x: usize) -> usize {
        let mut current = x;
        while let Some(parent) = self.cells[current].parent() {
            if F::CHENGE_ROOT
                && let Some(parent_parent) = self.cells[parent].parent()
            {
                let potential = P::operate(
                    &self.cells[parent].potential,
                    &self.cells[current].potential,
                );
                self.cells[current].set_child(parent_parent, potential);
            }
            current = parent;
        }
        current
    }

    pub fn unite_noninv(&mut self, x: usize, y: usize, potential: P::T) -> bool {
        let (rx, potx) = self.find(x);
        let ry = self.find_root(y);
        if rx == ry || y != ry {
            return false;
        }
        H::unite(&mut self.history, rx, ry, &self.cells);
        {
            let ptr = self.cells.as_mut_ptr();
            let (cx, cy) = unsafe { (&mut *ptr.add(rx), &mut *ptr.add(ry)) };
            self.merger.merge(cx.data_mut(), cy.data_mut());
        }
        let info = U::unite(&self.root_info(rx).unwrap(), &self.root_info(ry).unwrap());
        self.set_root_info(rx, info);
        self.cells[ry].set_child(rx, P::operate(&potx, &potential));
        true
    }
}

impl<U, F, M, P, H> UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec,
    P: Group,
    H: UndoStrategy<UfCell<M::Data, P>>,
{
    pub fn difference(&mut self, x: usize, y: usize) -> Option<P::T> {
        let (rx, potx) = self.find(x);
        let (ry, poty) = self.find(y);
        if rx == ry {
            Some(P::operate(&P::inverse(&potx), &poty))
        } else {
            None
        }
    }

    pub fn unite_with(&mut self, x: usize, y: usize, potential: P::T) -> bool {
        let (mut rx, potx) = self.find(x);
        let (mut ry, poty) = self.find(y);
        if rx == ry {
            return false;
        }
        let mut xinfo = self.root_info(rx).unwrap();
        let mut yinfo = self.root_info(ry).unwrap();
        let inverse = !U::check_directoin(&xinfo, &yinfo);
        let potential = if inverse {
            P::rinv_operate(&poty, &P::operate(&potx, &potential))
        } else {
            P::operate(&potx, &P::rinv_operate(&potential, &poty))
        };
        if inverse {
            swap(&mut rx, &mut ry);
            swap(&mut xinfo, &mut yinfo);
        }
        H::unite(&mut self.history, rx, ry, &self.cells);
        {
            let ptr = self.cells.as_mut_ptr();
            let (cx, cy) = unsafe { (&mut *ptr.add(rx), &mut *ptr.add(ry)) };
            self.merger.merge(cx.data_mut(), cy.data_mut());
        }
        self.set_root_info(rx, U::unite(&xinfo, &yinfo));
        self.cells[ry].set_child(rx, potential);
        true
    }

    pub fn unite(&mut self, x: usize, y: usize) -> bool {
        self.unite_with(x, y, P::unit())
    }
}

impl<U, M, P, H> UnionFindBase<U, (), M, P, H>
where
    U: UnionStrategy,
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<M::Data, P>>,
{
    pub fn undo(&mut self) {
        H::undo_unite(&mut self.history, &mut self.cells);
    }
}

pub type UnionFind = UnionFindBase<UnionBySize, PathCompression, (), (), ()>;
pub type MergingUnionFind<T, M> =
    UnionFindBase<UnionBySize, PathCompression, FnMerger<T, M>, (), ()>;
pub type PotentializedUnionFind<P> = UnionFindBase<UnionBySize, PathCompression, (), P, ()>;
pub type UndoableUnionFind = UnionFindBase<UnionBySize, (), (), (), Undoable>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{Invertible, LinearOperation, Magma, Unital},
        graph::UndirectedSparseGraph,
        num::mint_basic::MInt998244353 as M,
        rand,
        tools::Xorshift,
        tree::MixedTree,
    };
    use std::collections::HashSet;

    fn distinct_edges(rng: &mut Xorshift, n: usize, m: usize) -> Vec<(usize, usize)> {
        let mut edges = vec![];
        for x in 0..n {
            for y in 0..n {
                edges.push((x, y));
            }
        }
        rng.shuffle(&mut edges);
        edges.truncate(m);
        edges
    }

    fn dfs(
        g: &UndirectedSparseGraph,
        u: usize,
        vis: &mut [bool],
        f: &mut impl FnMut(usize),
        f2: &mut impl FnMut(usize, usize, usize),
    ) {
        vis[u] = true;
        f(u);
        for a in g.adjacencies(u) {
            if !vis[a.to] {
                f2(u, a.to, a.id);
                dfs(g, a.to, vis, f, f2);
            }
        }
    }

    #[test]
    fn test_union_find() {
        const N: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            rand!(rng, n: 1..=N, m: 1..=n * n);
            let edges = distinct_edges(&mut rng, n, m);

            macro_rules! test_uf {
                ($union:ty, $find:ty) => {{
                    let mut uf = UnionFindBase::<$union, $find, FnMerger<Vec<usize>, _>, (), ()>::new_with_merger(n, |i| vec![i], |x, y| x.append(y));
                    for &(x, y) in &edges {
                        uf.unite(x, y);
                    }
                    let g = UndirectedSparseGraph::from_edges(n, edges.to_vec());
                    let mut id = vec![!0; n];
                    {
                        let mut vis = vec![false; n];
                        for x in 0..n {
                            if vis[x] {
                                continue;
                            }
                            let mut set = HashSet::new();
                            dfs(
                                &g,
                                x,
                                &mut vis,
                                &mut |x| {
                                    set.insert(x);
                                },
                                &mut |_, _, _| {},
                            );
                            for s in set {
                                id[s] = x;
                            }
                        }
                    }
                    for x in 0..n {
                        for y in 0..n {
                            assert_eq!(id[x] == id[y], uf.same(x, y));
                        }
                        assert_eq!(
                            (0..n).filter(|&y| id[x] == id[y]).collect::<HashSet<_>>(),
                            uf.merge_data(x).iter().cloned().collect()
                        );
                    }
                }};
            }
            test_uf!(UnionBySize, PathCompression);
            test_uf!(UnionByRank, PathCompression);
            test_uf!((), PathCompression);
            test_uf!(UnionBySize, ());
            test_uf!(UnionByRank, ());
            test_uf!((), ());
        }
    }

    #[test]
    fn test_potential_union_find() {
        const N: usize = 20;
        let mut rng = Xorshift::default();
        type G = LinearOperation<M>;
        for _ in 0..1000 {
            rand!(rng, n: 1..=N, g: MixedTree(n), p: [(.., ..); n - 1], k: 0..n);

            macro_rules! test_uf {
                ($union:ty, $find:ty) => {{
                    let mut uf = UnionFindBase::<$union, $find, (), G, ()>::new(n);
                    for (i, &(u, v)) in g.edges.iter().enumerate().take(k) {
                        uf.unite_with(u, v, p[i]);
                    }
                    for x in 0..n {
                        let mut vis = vec![false; n];
                        let mut dp = vec![None; n];
                        dp[x] = Some(G::unit());
                        dfs(&g, x, &mut vis, &mut |_| {}, &mut |u, to, id| {
                            let p = if g.edges[id] == (u, to) {
                                p[id]
                            } else {
                                G::inverse(&p[id])
                            };
                            if id < k {
                                if let Some(d) = dp[u] {
                                    dp[to] = Some(G::operate(&d, &p));
                                }
                            }
                        });
                        for (y, d) in dp.into_iter().enumerate() {
                            assert_eq!(d, uf.difference(x, y));
                        }
                    }
                }};
            }
            test_uf!(UnionBySize, PathCompression);
            test_uf!(UnionByRank, PathCompression);
            test_uf!((), PathCompression);
            test_uf!(UnionBySize, ());
            test_uf!(UnionByRank, ());
            test_uf!((), ());
        }
    }

    #[test]
    fn test_undoable_union_find() {
        const N: usize = 10;
        const M: usize = 200;
        let mut rng = Xorshift::default();
        for _ in 0..10 {
            rand!(rng, n: 1..=N, m: 1..=M, g: MixedTree(m), p: [(0..n, 0..n); m]);

            macro_rules! test_uf {
                ($union:ty, $find:ty) => {{
                    let uf = UnionFind::new(n);
                    let mut uf2 = UnionFindBase::<$union, $find, (), (), Undoable>::new(n);
                    fn dfs(
                        n: usize,
                        g: &UndirectedSparseGraph,
                        u: usize,
                        vis: &mut [bool],
                        mut uf: UnionFindBase<UnionBySize, PathCompression, (), (), ()>,
                        uf2: &mut UnionFindBase<$union, $find, (), (), Undoable>,
                        p: &[(usize, usize)],
                    ) {
                        vis[u] = true;
                        for x in 0..n {
                            for y in 0..n {
                                assert_eq!(uf.same(x, y), uf2.same(x, y));
                            }
                        }
                        for a in g.adjacencies(u) {
                            if !vis[a.to] {
                                let (x, y) = p[a.id];
                                let mut uf = uf.clone();
                                uf.unite(x, y);
                                let merged = uf2.unite(x, y);
                                dfs(n, g, a.to, vis, uf, uf2, p);
                                if merged {
                                    uf2.undo();
                                }
                            }
                        }
                    }
                    for u in 0..m {
                        dfs(n, &g, u, &mut vec![false; m], uf.clone(), &mut uf2, &p);
                    }
                }};
            }
            test_uf!(UnionBySize, ());
            test_uf!(UnionByRank, ());
            test_uf!((), ());
        }
    }
}
