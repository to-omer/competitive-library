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

pub trait ShortestPathExt<'g>: GraphBase<'g> {
    fn bfs_distance_ss<'a, S, M>(
        &'g self,
        source: Self::VIndex,
        weight: &'a M,
    ) -> <Self as VertexMap<'g, S::T>>::Vmap
    where
        Self: VertexMap<'g, S::T> + AdjacencyView<'g, 'a, M, S::T>,
        S: ShortestPathSemiRing,
    {
        self.bfs_distance_ms::<S, M, _>(once(source), weight)
    }
    fn bfs_distance_ms<'a, S, M, I>(
        &'g self,
        sources: I,
        weight: &'a M,
    ) -> <Self as VertexMap<'g, S::T>>::Vmap
    where
        Self: VertexMap<'g, S::T> + AdjacencyView<'g, 'a, M, S::T>,
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
        &'g self,
        source: Self::VIndex,
        weight: &'a M,
    ) -> <Self as VertexMap<'g, S::T>>::Vmap
    where
        Self: VertexMap<'g, S::T> + AdjacencyView<'g, 'a, M, S::T>,
        S: ShortestPathSemiRing,
    {
        self.dijkstra_ms::<S, M, _>(once(source), weight)
    }
    fn dijkstra_ms<'a, S, M, I>(
        &'g self,
        sources: I,
        weight: &'a M,
    ) -> <Self as VertexMap<'g, S::T>>::Vmap
    where
        Self: VertexMap<'g, S::T> + AdjacencyView<'g, 'a, M, S::T>,
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
        &'g self,
        source: Self::VIndex,
        weight: &'a M,
        check: bool,
    ) -> Option<<Self as VertexMap<'g, S::T>>::Vmap>
    where
        Self: Vertices<'g> + VertexMap<'g, S::T> + AdjacencyView<'g, 'a, M, S::T> + VertexSize<'g>,
        S: ShortestPathSemiRing,
    {
        self.bellman_ford_ms::<S, M, _>(once(source), weight, check)
    }
    fn bellman_ford_ms<'a, S, M, I>(
        &'g self,
        sources: I,
        weight: &'a M,
        check: bool,
    ) -> Option<<Self as VertexMap<'g, S::T>>::Vmap>
    where
        Self: Vertices<'g> + VertexMap<'g, S::T> + AdjacencyView<'g, 'a, M, S::T> + VertexSize<'g>,
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
        &'g self,
        weight: &'a M,
    ) -> <Self as VertexMap<'g, <Self as VertexMap<'g, S::T>>::Vmap>>::Vmap
    where
        Self: Vertices<'g>
            + VertexMap<'g, S::T>
            + VertexMap<'g, <Self as VertexMap<'g, S::T>>::Vmap>
            + AdjacencyView<'g, 'a, M, S::T>,
        <Self as VertexMap<'g, S::T>>::Vmap: Clone,
        S: ShortestPathSemiRing,
    {
        let mut cost = self.construct_vmap(|| self.construct_vmap(S::inf));
        for u in self.vertices() {
            *self.vmap_get_mut(self.vmap_get_mut(&mut cost, u), u) = S::source();
        }
        for u in self.vertices() {
            for a in self.aviews(weight, u) {
                *self.vmap_get_mut(self.vmap_get_mut(&mut cost, u), a.vindex()) = a.avalue();
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
impl<'g, G> ShortestPathExt<'g> for G where G: GraphBase<'g> {}
