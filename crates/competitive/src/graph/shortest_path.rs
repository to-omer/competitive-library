use super::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, VecDeque},
    iter::once,
    marker::PhantomData,
    ops::{Add, Mul},
};

pub trait ShortestPathSemiRing {
    type T: Clone + Ord;
    fn source() -> Self::T;
    fn inf() -> Self::T;
    fn mul(x: &Self::T, y: &Self::T) -> Self::T;
    fn add_assign(x: &mut Self::T, y: &Self::T) -> bool;
}

pub struct StandardSp<M>(PhantomData<fn() -> M>);
impl<M> ShortestPathSemiRing for StandardSp<M>
where
    M: Monoid,
    M::T: Bounded + Ord,
{
    type T = M::T;
    fn source() -> Self::T {
        M::unit()
    }
    fn inf() -> Self::T {
        M::T::maximum()
    }
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        M::operate(x, y)
    }
    fn add_assign(x: &mut Self::T, y: &Self::T) -> bool {
        if &*x > y {
            *x = y.clone();
            true
        } else {
            false
        }
    }
}

pub struct OptionSp<M>(PhantomData<fn() -> M>);
impl<M> ShortestPathSemiRing for OptionSp<M>
where
    M: Monoid,
    M::T: Ord,
{
    type T = Option<M::T>;
    fn source() -> Self::T {
        Some(M::unit())
    }
    fn inf() -> Self::T {
        None
    }
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        match (x, y) {
            (Some(x), Some(y)) => Some(M::operate(x, y)),
            _ => None,
        }
    }
    fn add_assign(x: &mut Self::T, y: &Self::T) -> bool {
        if let Some(y) = y {
            if let Some(x) = x {
                if &*x > y {
                    *x = y.clone();
                    true
                } else {
                    false
                }
            } else {
                *x = Some(y.clone());
                true
            }
        } else {
            false
        }
    }
}

pub struct PathFoldingSp<M, S>(PhantomData<fn() -> (M, S)>);
impl<M, S> ShortestPathSemiRing for PathFoldingSp<M, S>
where
    M: Monoid,
    M::T: Bounded + Ord,
    S: SemiRing,
{
    type T = PartialIgnoredOrd<M::T, S::T>;
    fn source() -> Self::T {
        PartialIgnoredOrd(M::unit(), S::one())
    }
    fn inf() -> Self::T {
        PartialIgnoredOrd(M::T::maximum(), S::zero())
    }
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        PartialIgnoredOrd(M::operate(&x.0, &y.0), S::mul(&x.1, &y.1))
    }
    fn add_assign(x: &mut Self::T, y: &Self::T) -> bool {
        match x.0.cmp(&y.0) {
            Ordering::Equal => {
                x.1 = S::add(&x.1, &y.1);
                false
            }
            Ordering::Greater => {
                *x = y.clone();
                true
            }
            _ => false,
        }
    }
}

pub trait ShortestPathExt: GraphBase {
    fn standard_sp<'a, M>(&'a self) -> ShortestPathBuilder<'a, Self, StandardSp<M>>
    where
        Self: Sized + GraphBase,
        M: Monoid,
        M::T: Bounded + Ord,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn standard_sp_additive<'a, T>(
        &'a self,
    ) -> ShortestPathBuilder<'a, Self, StandardSp<AdditiveOperation<T>>>
    where
        Self: Sized + GraphBase,
        T: Clone + Zero + Add<Output = T> + Bounded + Ord,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn option_sp<'a, M>(&'a self) -> ShortestPathBuilder<'a, Self, OptionSp<M>>
    where
        Self: Sized + GraphBase,
        M: Monoid,
        M::T: Ord,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn option_sp_additive<'a, T>(
        &'a self,
    ) -> ShortestPathBuilder<'a, Self, OptionSp<AdditiveOperation<T>>>
    where
        Self: Sized + GraphBase,
        T: Clone + Zero + Add<Output = T> + Ord,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn path_folding_sp<'a, M, S>(&'a self) -> ShortestPathBuilder<'a, Self, PathFoldingSp<M, S>>
    where
        Self: Sized + GraphBase,
        M: Monoid,
        M::T: Bounded + Ord,
        S: SemiRing,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn path_folding_sp_additive_addmul<'a, T, U>(
        &'a self,
    ) -> ShortestPathBuilder<'a, Self, PathFoldingSp<AdditiveOperation<T>, AddMulOperation<U>>>
    where
        Self: Sized + GraphBase,
        T: Clone + Zero + Add<Output = T> + Bounded + Ord,
        U: Clone + Zero + One + Add<Output = U> + Mul<Output = U>,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }
}
impl<G> ShortestPathExt for G where G: GraphBase {}

pub struct ShortestPathBuilder<'a, G, S>
where
    G: GraphBase,
    S: ShortestPathSemiRing,
{
    graph: &'a G,
    _marker: PhantomData<fn() -> S>,
}

impl<'a, G, S> ShortestPathBuilder<'a, G, S>
where
    G: GraphBase,
    S: ShortestPathSemiRing,
{
    pub fn bfs_distance_ss<M>(
        &self,
        source: G::VIndex,
        weight: &'a M,
    ) -> <G as VertexMap<S::T>>::Vmap
    where
        G: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
    {
        self.bfs_distance_ms::<M, _>(once(source), weight)
    }

    pub fn bfs_distance_ms<M, I>(&self, sources: I, weight: &'a M) -> <G as VertexMap<S::T>>::Vmap
    where
        G: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
        I: IntoIterator<Item = G::VIndex>,
    {
        let graph = self.graph;
        let mut cost = graph.construct_vmap(S::inf);
        let mut deq = VecDeque::new();
        for source in sources.into_iter() {
            *graph.vmap_get_mut(&mut cost, source) = S::source();
            deq.push_back(source);
        }
        let zero = S::source();
        while let Some(u) = deq.pop_front() {
            for a in graph.aviews(weight, u) {
                let v = a.vindex();
                let w = a.avalue();
                let nd = S::mul(graph.vmap_get(&cost, u), &w);
                if S::add_assign(graph.vmap_get_mut(&mut cost, v), &nd) {
                    if w == zero {
                        deq.push_front(v);
                    } else {
                        deq.push_back(v);
                    }
                }
            }
        }
        cost
    }

    pub fn dijkstra_ss<M>(&self, source: G::VIndex, weight: &'a M) -> <G as VertexMap<S::T>>::Vmap
    where
        G: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
    {
        self.dijkstra_ms::<M, _>(once(source), weight)
    }

    pub fn dijkstra_ms<M, I>(&self, sources: I, weight: &'a M) -> <G as VertexMap<S::T>>::Vmap
    where
        G: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
        I: IntoIterator<Item = G::VIndex>,
    {
        let graph = self.graph;
        let mut cost = graph.construct_vmap(S::inf);
        let mut heap = BinaryHeap::new();
        for source in sources.into_iter() {
            *graph.vmap_get_mut(&mut cost, source) = S::source();
            heap.push(PartialIgnoredOrd(Reverse(S::source()), source));
        }
        while let Some(PartialIgnoredOrd(Reverse(d), u)) = heap.pop() {
            if graph.vmap_get(&cost, u) != &d {
                continue;
            }
            let d = graph.vmap_get(&cost, u).clone();
            for a in graph.aviews(weight, u) {
                let v = a.vindex();
                let nd = S::mul(&d, &a.avalue());
                if S::add_assign(graph.vmap_get_mut(&mut cost, v), &nd) {
                    heap.push(PartialIgnoredOrd(Reverse(nd), v));
                }
            }
        }
        cost
    }

    pub fn bellman_ford_ss<M>(
        &self,
        source: G::VIndex,
        weight: &'a M,
        check: bool,
    ) -> Option<<G as VertexMap<S::T>>::Vmap>
    where
        G: Vertices + VertexMap<S::T> + AdjacencyView<'a, M, S::T> + VertexSize,
        S: ShortestPathSemiRing,
    {
        self.bellman_ford_ms::<M, _>(once(source), weight, check)
    }

    pub fn bellman_ford_ms<M, I>(
        &self,
        sources: I,
        weight: &'a M,
        check: bool,
    ) -> Option<<G as VertexMap<S::T>>::Vmap>
    where
        G: Vertices + VertexMap<S::T> + AdjacencyView<'a, M, S::T> + VertexSize,
        S: ShortestPathSemiRing,
        I: IntoIterator<Item = G::VIndex>,
    {
        let graph = self.graph;
        let mut cost = graph.construct_vmap(S::inf);
        for source in sources.into_iter() {
            *graph.vmap_get_mut(&mut cost, source) = S::source();
        }
        let vsize = graph.vsize();
        for _ in 1..vsize {
            let mut updated = false;
            for u in graph.vertices() {
                for a in graph.aviews(weight, u) {
                    let v = a.vindex();
                    let nd = S::mul(graph.vmap_get(&cost, u), &a.avalue());
                    updated |= S::add_assign(graph.vmap_get_mut(&mut cost, v), &nd);
                }
            }
            if !updated {
                break;
            }
        }
        if check {
            for u in graph.vertices() {
                for a in graph.aviews(weight, u) {
                    let v = a.vindex();
                    let nd = S::mul(graph.vmap_get(&cost, u), &a.avalue());
                    if S::add_assign(graph.vmap_get_mut(&mut cost, v), &nd) {
                        return None;
                    }
                }
            }
        }
        Some(cost)
    }

    pub fn warshall_floyd_ap<M>(
        &self,
        weight: &'a M,
    ) -> <G as VertexMap<<G as VertexMap<S::T>>::Vmap>>::Vmap
    where
        G: Vertices
            + VertexMap<S::T>
            + VertexMap<<G as VertexMap<S::T>>::Vmap>
            + AdjacencyView<'a, M, S::T>,
        <G as VertexMap<S::T>>::Vmap: Clone,
        S: ShortestPathSemiRing,
    {
        let graph = self.graph;
        let mut cost = graph.construct_vmap(|| graph.construct_vmap(S::inf));
        for u in graph.vertices() {
            *graph.vmap_get_mut(graph.vmap_get_mut(&mut cost, u), u) = S::source();
        }
        for u in graph.vertices() {
            for a in graph.aviews(weight, u) {
                S::add_assign(
                    graph.vmap_get_mut(graph.vmap_get_mut(&mut cost, u), a.vindex()),
                    &a.avalue(),
                );
            }
        }
        for k in graph.vertices() {
            for i in graph.vertices() {
                for j in graph.vertices() {
                    let d1 = graph.vmap_get(graph.vmap_get(&cost, i), k);
                    let d2 = graph.vmap_get(graph.vmap_get(&cost, k), j);
                    let nd = S::mul(d1, d2);
                    S::add_assign(graph.vmap_get_mut(graph.vmap_get_mut(&mut cost, i), j), &nd);
                }
            }
        }
        cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::{Saturating, mint_basic::MInt998244353},
        rand,
        tools::Xorshift,
    };

    #[test]
    fn test_shortest_path() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..100, m: 1..200, edges: [(0..n, 0..n); m], w: [0..100_000i64; m]);
            let g = DirectedSparseGraph::from_edges(n, edges);
            let dijkstra: Vec<_> = (0..n)
                .map(|src| g.option_sp_additive().dijkstra_ss(src, &|eid| Some(w[eid])))
                .collect();
            let bellman_ford: Vec<_> = (0..n)
                .map(|src| {
                    g.option_sp_additive()
                        .bellman_ford_ss(src, &|eid| Some(w[eid]), false)
                        .unwrap()
                })
                .collect();
            let warshall_floyd = g
                .option_sp_additive()
                .warshall_floyd_ap(&|eid| Some(w[eid]));
            assert_eq!(dijkstra, bellman_ford);
            assert_eq!(dijkstra, warshall_floyd);
        }
    }

    #[test]
    fn test_spfa() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..100, m: 1..200, edges: [(0..n, 0..n); m], ub: 0..=1i64, w: [0..=ub * 100_000i64; m]);
            let g = DirectedSparseGraph::from_edges(n, edges);
            let bfs: Vec<_> = (0..n)
                .map(|src| {
                    g.option_sp_additive()
                        .bfs_distance_ss(src, &|eid| Some(w[eid]))
                })
                .collect();
            let warshall_floyd = g
                .option_sp_additive()
                .warshall_floyd_ap(&|eid| Some(w[eid]));
            assert_eq!(bfs, warshall_floyd);
        }
    }

    #[test]
    fn test_path_folding() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..100, m: 1..200, edges: [(0..n, 0..n); m], w: [0..100_000usize; m]);
            let g = DirectedSparseGraph::from_edges(n, edges);
            let dijkstra: Vec<_> = (0..n)
                .map(|src| {
                    g.path_folding_sp_additive_addmul()
                        .dijkstra_ss(src, &|eid| {
                            PartialIgnoredOrd(Saturating(w[eid]), MInt998244353::one())
                        })
                })
                .collect();
            let bellman_ford: Vec<_> = (0..n)
                .map(|src| {
                    g.path_folding_sp_additive_addmul()
                        .bellman_ford_ss(
                            src,
                            &|eid| PartialIgnoredOrd(Saturating(w[eid]), MInt998244353::one()),
                            false,
                        )
                        .unwrap()
                })
                .collect();
            let warshall_floyd = g
                .path_folding_sp_additive_addmul()
                .warshall_floyd_ap(&|eid| {
                    PartialIgnoredOrd(Saturating(w[eid]), MInt998244353::one())
                });
            assert_eq!(dijkstra, bellman_ford);
            assert_eq!(dijkstra, warshall_floyd);
        }
    }
}
