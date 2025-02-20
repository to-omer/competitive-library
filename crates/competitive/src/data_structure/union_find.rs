use super::{Group, Monoid};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::swap,
};

pub struct UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<U, M, P>>,
{
    cells: Vec<UfCell<U, M, P>>,
    merger: M,
    history: H::History,
    _marker: PhantomData<fn() -> F>,
}

impl<U, F, M, P, H> Clone for UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec + Clone,
    P: Monoid,
    H: UndoStrategy<UfCell<U, M, P>>,
    U::Info: Clone,
    M::Data: Clone,
    H::History: Clone,
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
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<U, M, P>>,
    U::Info: Debug,
    M::Data: Debug,
    P::T: Debug,
    H::History: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionFindBase")
            .field("cells", &self.cells)
            .field("history", &self.history)
            .finish()
    }
}

pub enum UfCell<U, M, P>
where
    U: UnionStrategy,
    M: UfMergeSpec,
    P: Monoid,
{
    Root((U::Info, M::Data)),
    Child((usize, P::T)),
}

impl<U, M, P> Clone for UfCell<U, M, P>
where
    U: UnionStrategy,
    M: UfMergeSpec,
    P: Monoid,
    U::Info: Clone,
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Root(data) => Self::Root(data.clone()),
            Self::Child(data) => Self::Child(data.clone()),
        }
    }
}

impl<U, M, P> Debug for UfCell<U, M, P>
where
    U: UnionStrategy,
    M: UfMergeSpec,
    P: Monoid,
    U::Info: Debug,
    M::Data: Debug,
    P::T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root(data) => f.debug_tuple("Root").field(data).finish(),
            Self::Child(data) => f.debug_tuple("Child").field(data).finish(),
        }
    }
}

impl<U, M, P> UfCell<U, M, P>
where
    U: UnionStrategy,
    M: UfMergeSpec,
    P: Monoid,
{
    fn root_mut(&mut self) -> Option<&mut (U::Info, M::Data)> {
        match self {
            UfCell::Root(root) => Some(root),
            UfCell::Child(_) => None,
        }
    }
    fn child_mut(&mut self) -> Option<&mut (usize, P::T)> {
        match self {
            UfCell::Child(child) => Some(child),
            UfCell::Root(_) => None,
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
    type Info: Clone;
    fn single_info() -> Self::Info;
    fn check_directoin(parent: &Self::Info, child: &Self::Info) -> bool;
    fn unite(parent: &Self::Info, child: &Self::Info) -> Self::Info;
}

pub enum UnionBySize {}

impl UnionStrategy for UnionBySize {
    type Info = usize;

    fn single_info() -> Self::Info {
        1
    }

    fn check_directoin(parent: &Self::Info, child: &Self::Info) -> bool {
        parent >= child
    }

    fn unite(parent: &Self::Info, child: &Self::Info) -> Self::Info {
        parent + child
    }
}

pub enum UnionByRank {}

impl UnionStrategy for UnionByRank {
    type Info = u32;

    fn single_info() -> Self::Info {
        0
    }

    fn check_directoin(parent: &Self::Info, child: &Self::Info) -> bool {
        parent >= child
    }

    fn unite(parent: &Self::Info, child: &Self::Info) -> Self::Info {
        parent + (parent == child) as u32
    }
}

impl UnionStrategy for () {
    type Info = ();

    fn single_info() -> Self::Info {}

    fn check_directoin(_parent: &Self::Info, _child: &Self::Info) -> bool {
        false
    }

    fn unite(_parent: &Self::Info, _child: &Self::Info) -> Self::Info {}
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
    H: UndoStrategy<UfCell<U, (), P>>,
{
    pub fn new(n: usize) -> Self {
        let cells: Vec<_> = (0..n)
            .map(|_| UfCell::Root((U::single_info(), ())))
            .collect();
        Self {
            cells,
            merger: (),
            history: Default::default(),
            _marker: PhantomData,
        }
    }
    pub fn push(&mut self) {
        self.cells.push(UfCell::Root((U::single_info(), ())));
    }
}

impl<U, F, T, Merge, P, H> UnionFindBase<U, F, FnMerger<T, Merge>, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    Merge: FnMut(&mut T, &mut T),
    P: Monoid,
    H: UndoStrategy<UfCell<U, FnMerger<T, Merge>, P>>,
{
    pub fn new_with_merger(n: usize, mut init: impl FnMut(usize) -> T, merge: Merge) -> Self {
        let cells: Vec<_> = (0..n)
            .map(|i| UfCell::Root((U::single_info(), init(i))))
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
    H: UndoStrategy<UfCell<UnionBySize, M, P>>,
{
    pub fn size(&mut self, x: usize) -> <UnionBySize as UnionStrategy>::Info {
        let root = self.find_root(x);
        self.root_info(root).unwrap()
    }
}

impl<U, F, M, P, H> UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec,
    P: Monoid,
    H: UndoStrategy<UfCell<U, M, P>>,
{
    fn root_info(&mut self, x: usize) -> Option<U::Info> {
        match &self.cells[x] {
            UfCell::Root((info, _)) => Some(info.clone()),
            UfCell::Child(_) => None,
        }
    }

    fn root_info_mut(&mut self, x: usize) -> Option<&mut U::Info> {
        match &mut self.cells[x] {
            UfCell::Root((info, _)) => Some(info),
            UfCell::Child(_) => None,
        }
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find_root(x) == self.find_root(y)
    }

    pub fn merge_data(&mut self, x: usize) -> &M::Data {
        let root = self.find_root(x);
        match &self.cells[root] {
            UfCell::Root((_, data)) => data,
            UfCell::Child(_) => unreachable!(),
        }
    }

    pub fn merge_data_mut(&mut self, x: usize) -> &mut M::Data {
        let root = self.find_root(x);
        match &mut self.cells[root] {
            UfCell::Root((_, data)) => data,
            UfCell::Child(_) => unreachable!(),
        }
    }

    pub fn roots(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.cells.len()).filter(|&x| matches!(self.cells[x], UfCell::Root(_)))
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
        let (parent_parent, parent_potential) = match &self.cells[x] {
            UfCell::Child((parent, _)) => self.find(*parent),
            UfCell::Root(_) => return (x, P::unit()),
        };
        let (parent, potential) = self.cells[x].child_mut().unwrap();
        let potential = if F::CHENGE_ROOT {
            *parent = parent_parent;
            *potential = P::operate(&parent_potential, potential);
            potential.clone()
        } else {
            P::operate(&parent_potential, potential)
        };
        (parent_parent, potential)
    }

    pub fn find_root(&mut self, x: usize) -> usize {
        let (parent, parent_parent) = match &self.cells[x] {
            UfCell::Child((parent, _)) => (*parent, self.find_root(*parent)),
            UfCell::Root(_) => return x,
        };
        if F::CHENGE_ROOT {
            let (cx, cp) = {
                let ptr = self.cells.as_mut_ptr();
                unsafe { (&mut *ptr.add(x), &*ptr.add(parent)) }
            };
            let (parent, potential) = cx.child_mut().unwrap();
            *parent = parent_parent;
            if let UfCell::Child((_, ppot)) = &cp {
                *potential = P::operate(ppot, potential);
            }
        }
        parent_parent
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
            self.merger
                .merge(&mut cx.root_mut().unwrap().1, &mut cy.root_mut().unwrap().1);
        }
        *self.root_info_mut(rx).unwrap() =
            U::unite(&self.root_info(rx).unwrap(), &self.root_info(ry).unwrap());
        self.cells[ry] = UfCell::Child((rx, P::operate(&potx, &potential)));
        true
    }
}

impl<U, F, M, P, H> UnionFindBase<U, F, M, P, H>
where
    U: UnionStrategy,
    F: FindStrategy,
    M: UfMergeSpec,
    P: Group,
    H: UndoStrategy<UfCell<U, M, P>>,
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
            self.merger
                .merge(&mut cx.root_mut().unwrap().1, &mut cy.root_mut().unwrap().1);
        }
        *self.root_info_mut(rx).unwrap() = U::unite(&xinfo, &yinfo);
        self.cells[ry] = UfCell::Child((rx, potential));
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
    H: UndoStrategy<UfCell<U, M, P>>,
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
        tools::{RandomSpec, Xorshift},
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

    struct Mspec;
    impl RandomSpec<M> for Mspec {
        fn rand(&self, rng: &mut Xorshift) -> M {
            M::new_unchecked(rng.random(0..M::get_mod()))
        }
    }

    #[test]
    fn test_union_find() {
        const N: usize = 20;
        let mut rng = Xorshift::default();
        for _ in 0..1000 {
            rand!(rng, n: (1..=N), m: (1..=n * n));
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
            rand!(rng, n: (1..=N), g: (MixedTree(n)), p: [(Mspec, Mspec); n - 1], k: (0..n));

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
            rand!(rng, n: (1..=N), m: (1..=M), g: (MixedTree(m)), p: [(0..n, 0..n); m]);

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
