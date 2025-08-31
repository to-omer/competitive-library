use super::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, VecDeque},
    iter::once,
    marker::PhantomData,
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
    fn bfs_distance_ss<'a, S, M>(
        &self,
        source: Self::VIndex,
        weight: &'a M,
    ) -> <Self as VertexMap<S::T>>::Vmap
    where
        Self: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
    {
        self.bfs_distance_ms::<S, M, _>(once(source), weight)
    }
    fn bfs_distance_ms<'a, S, M, I>(
        &self,
        sources: I,
        weight: &'a M,
    ) -> <Self as VertexMap<S::T>>::Vmap
    where
        Self: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
        I: IntoIterator<Item = Self::VIndex>,
    {
        let mut cost = self.construct_vmap(S::inf);
        let mut deq = VecDeque::new();
        for source in sources.into_iter() {
            *self.vmap_get_mut(&mut cost, source) = S::source();
            deq.push_back(source);
        }
        let zero = S::source();
        while let Some(u) = deq.pop_front() {
            for a in self.aviews(weight, u) {
                let v = a.vindex();
                let w = a.avalue();
                let nd = S::mul(self.vmap_get(&cost, u), &w);
                if S::add_assign(self.vmap_get_mut(&mut cost, v), &nd) {
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
    fn dijkstra_ss<'a, S, M>(
        &self,
        source: Self::VIndex,
        weight: &'a M,
    ) -> <Self as VertexMap<S::T>>::Vmap
    where
        Self: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
    {
        self.dijkstra_ms::<S, M, _>(once(source), weight)
    }
    fn dijkstra_ms<'a, S, M, I>(&self, sources: I, weight: &'a M) -> <Self as VertexMap<S::T>>::Vmap
    where
        Self: VertexMap<S::T> + AdjacencyView<'a, M, S::T>,
        S: ShortestPathSemiRing,
        I: IntoIterator<Item = Self::VIndex>,
    {
        let mut cost = self.construct_vmap(S::inf);
        let mut heap = BinaryHeap::new();
        for source in sources.into_iter() {
            *self.vmap_get_mut(&mut cost, source) = S::source();
            heap.push(PartialIgnoredOrd(Reverse(S::source()), source));
        }
        while let Some(PartialIgnoredOrd(Reverse(d), u)) = heap.pop() {
            if self.vmap_get(&cost, u) != &d {
                continue;
            }
            let d = self.vmap_get(&cost, u).clone();
            for a in self.aviews(weight, u) {
                let v = a.vindex();
                let nd = S::mul(&d, &a.avalue());
                if S::add_assign(self.vmap_get_mut(&mut cost, v), &nd) {
                    heap.push(PartialIgnoredOrd(Reverse(nd), v));
                }
            }
        }
        cost
    }
    fn bellman_ford_ss<'a, S, M>(
        &self,
        source: Self::VIndex,
        weight: &'a M,
        check: bool,
    ) -> Option<<Self as VertexMap<S::T>>::Vmap>
    where
        Self: Vertices + VertexMap<S::T> + AdjacencyView<'a, M, S::T> + VertexSize,
        S: ShortestPathSemiRing,
    {
        self.bellman_ford_ms::<S, M, _>(once(source), weight, check)
    }
    fn bellman_ford_ms<'a, S, M, I>(
        &self,
        sources: I,
        weight: &'a M,
        check: bool,
    ) -> Option<<Self as VertexMap<S::T>>::Vmap>
    where
        Self: Vertices + VertexMap<S::T> + AdjacencyView<'a, M, S::T> + VertexSize,
        S: ShortestPathSemiRing,
        I: IntoIterator<Item = Self::VIndex>,
    {
        let mut cost = self.construct_vmap(S::inf);
        for source in sources.into_iter() {
            *self.vmap_get_mut(&mut cost, source) = S::source();
        }
        let vsize = self.vsize();
        for _ in 1..vsize {
            let mut updated = false;
            for u in self.vertices() {
                for a in self.aviews(weight, u) {
                    let v = a.vindex();
                    let nd = S::mul(self.vmap_get(&cost, u), &a.avalue());
                    updated |= S::add_assign(self.vmap_get_mut(&mut cost, v), &nd);
                }
            }
            if !updated {
                break;
            }
        }
        if check {
            for u in self.vertices() {
                for a in self.aviews(weight, u) {
                    let v = a.vindex();
                    let nd = S::mul(self.vmap_get(&cost, u), &a.avalue());
                    if S::add_assign(self.vmap_get_mut(&mut cost, v), &nd) {
                        return None;
                    }
                }
            }
        }
        Some(cost)
    }
    fn warshall_floyd_ap<'a, S, M>(
        &self,
        weight: &'a M,
    ) -> <Self as VertexMap<<Self as VertexMap<S::T>>::Vmap>>::Vmap
    where
        Self: Vertices
            + VertexMap<S::T>
            + VertexMap<<Self as VertexMap<S::T>>::Vmap>
            + AdjacencyView<'a, M, S::T>,
        <Self as VertexMap<S::T>>::Vmap: Clone,
        S: ShortestPathSemiRing,
    {
        let mut cost = self.construct_vmap(|| self.construct_vmap(S::inf));
        for u in self.vertices() {
            *self.vmap_get_mut(self.vmap_get_mut(&mut cost, u), u) = S::source();
        }
        for u in self.vertices() {
            for a in self.aviews(weight, u) {
                S::add_assign(
                    self.vmap_get_mut(self.vmap_get_mut(&mut cost, u), a.vindex()),
                    &a.avalue(),
                );
            }
        }
        for k in self.vertices() {
            for i in self.vertices() {
                for j in self.vertices() {
                    let d1 = self.vmap_get(self.vmap_get(&cost, i), k);
                    let d2 = self.vmap_get(self.vmap_get(&cost, k), j);
                    let nd = S::mul(d1, d2);
                    S::add_assign(self.vmap_get_mut(self.vmap_get_mut(&mut cost, i), j), &nd);
                }
            }
        }
        cost
    }
}
impl<G> ShortestPathExt for G where G: GraphBase {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AddMulOperation, AdditiveOperation},
        num::{One as _, Saturating, mint_basic::MInt998244353},
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
                .map(|src| {
                    g.dijkstra_ss::<OptionSp<AdditiveOperation<_>>, _>(src, &|eid| Some(w[eid]))
                })
                .collect();
            let bellman_ford: Vec<_> = (0..n)
                .map(|src| {
                    g.bellman_ford_ss::<OptionSp<AdditiveOperation<_>>, _>(
                        src,
                        &|eid| Some(w[eid]),
                        false,
                    )
                    .unwrap()
                })
                .collect();
            let warshall_floyd =
                g.warshall_floyd_ap::<OptionSp<AdditiveOperation<_>>, _>(&|eid| Some(w[eid]));
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
                    g.bfs_distance_ss::<OptionSp<AdditiveOperation<_>>, _>(src, &|eid| Some(w[eid]))
                })
                .collect();
            let warshall_floyd =
                g.warshall_floyd_ap::<OptionSp<AdditiveOperation<_>>, _>(&|eid| Some(w[eid]));
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
                    g.dijkstra_ss::<PathFoldingSp<AdditiveOperation<_>, AddMulOperation<MInt998244353>>, _>(src, &|eid| {
                        PartialIgnoredOrd(Saturating(w[eid]), MInt998244353::one())
                    })
                })
                .collect();
            let bellman_ford: Vec<_> = (0..n)
                .map(|src| {
                    g.bellman_ford_ss::<PathFoldingSp<AdditiveOperation<_>, AddMulOperation<MInt998244353>>, _>(
                        src,
                        &|eid| PartialIgnoredOrd(Saturating(w[eid]), MInt998244353::one()),
                        false,
                    )
                    .unwrap()
                })
                .collect();
            let warshall_floyd = g.warshall_floyd_ap::<PathFoldingSp<
                AdditiveOperation<_>,
                AddMulOperation<MInt998244353>,
            >, _>(&|eid| {
                PartialIgnoredOrd(Saturating(w[eid]), MInt998244353::one())
            });
            assert_eq!(dijkstra, bellman_ford);
            assert_eq!(dijkstra, warshall_floyd);
        }
    }
}
