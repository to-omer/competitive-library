use super::*;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, VecDeque},
    marker::PhantomData,
    ops::Add,
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
    M: Monoid<T: Bounded + Ord>,
{
    type T = M::T;
    #[inline]
    fn source() -> Self::T {
        M::unit()
    }
    #[inline]
    fn inf() -> Self::T {
        M::T::maximum()
    }
    #[inline]
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        M::operate(x, y)
    }
    #[inline]
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
    M: Monoid<T: Ord>,
{
    type T = Option<M::T>;
    #[inline]
    fn source() -> Self::T {
        Some(M::unit())
    }
    #[inline]
    fn inf() -> Self::T {
        None
    }
    #[inline]
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        match (x, y) {
            (Some(x), Some(y)) => Some(M::operate(x, y)),
            _ => None,
        }
    }
    #[inline]
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
    M: Monoid<T: Bounded + Ord>,
    S: SemiRing,
{
    type T = PartialIgnoredOrd<M::T, S::T>;
    #[inline]
    fn source() -> Self::T {
        PartialIgnoredOrd(M::unit(), S::one())
    }
    #[inline]
    fn inf() -> Self::T {
        PartialIgnoredOrd(M::T::maximum(), S::zero())
    }
    #[inline]
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        PartialIgnoredOrd(M::operate(&x.0, &y.0), S::mul(&x.1, &y.1))
    }
    #[inline]
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

pub trait ParentPolicy<G>
where
    G: Graph,
{
    type State;
    fn init(graph: &G) -> Self::State;
    fn save_parent(graph: &G, state: &mut Self::State, from: G::Vertex, to: G::Vertex);
}

pub enum NoParent {}
impl<G> ParentPolicy<G> for NoParent
where
    G: Graph,
{
    type State = ();
    fn init(_graph: &G) {}
    fn save_parent(_graph: &G, _state: &mut Self::State, _from: G::Vertex, _to: G::Vertex) {}
}

pub struct RecordParent;
impl<G> ParentPolicy<G> for RecordParent
where
    G: Graph + VertexMap<Option<<G as Graph>::Vertex>>,
{
    type State = <G as VertexMap<Option<<G as Graph>::Vertex>>>::Vmap;
    fn init(graph: &G) -> Self::State {
        graph.construct_vmap(|| None)
    }
    fn save_parent(graph: &G, state: &mut Self::State, from: G::Vertex, to: G::Vertex) {
        *graph.vmap_get_mut(state, to) = Some(from);
    }
}

pub struct ShortestPathWithParent<G, S, P = RecordParent>
where
    G: Graph + VertexMap<S::T>,
    S: ShortestPathSemiRing,
    P: ParentPolicy<G>,
{
    pub dist: <G as VertexMap<S::T>>::Vmap,
    pub parent: P::State,
}

impl<G, S> ShortestPathWithParent<G, S, RecordParent>
where
    G: Graph + VertexMap<S::T> + VertexMap<Option<<G as Graph>::Vertex>>,
    S: ShortestPathSemiRing,
{
    pub fn path_to(&self, graph: &G, target: G::Vertex) -> Option<Vec<G::Vertex>> {
        let dist: &S::T = graph.vmap_get(&self.dist, target);
        if dist == &S::inf() {
            return None;
        }
        let mut cur = target;
        let mut path = vec![cur];
        while let &Some(p) = graph.vmap_get(&self.parent, cur) {
            path.push(p);
            cur = p;
        }
        path.reverse();
        Some(path)
    }
}

pub trait ShortestPathExt: Graph {
    fn standard_sp<'a, M>(&'a self) -> ShortestPathBuilder<'a, Self, StandardSp<M>>
    where
        Self: Sized,
        M: Monoid<T: Bounded + Ord>,
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
        Self: Sized,
        T: Clone + Zero + Add<Output = T> + Bounded + Ord,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn option_sp<'a, M>(&'a self) -> ShortestPathBuilder<'a, Self, OptionSp<M>>
    where
        Self: Sized,
        M: Monoid<T: Ord>,
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
        Self: Sized,
        T: Clone + Zero + Add<Output = T> + Ord,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }

    fn path_folding_sp<'a, M, S>(&'a self) -> ShortestPathBuilder<'a, Self, PathFoldingSp<M, S>>
    where
        Self: Sized,
        M: Monoid<T: Bounded + Ord>,
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
        Self: Sized,
        T: Clone + Zero + Add<Output = T> + Bounded + Ord,
        U: DotProduct + One,
    {
        ShortestPathBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }
}
impl<G> ShortestPathExt for G where G: Graph + ?Sized {}

pub struct ShortestPathBuilder<'a, G, S, P = NoParent>
where
    G: Graph,
    S: ShortestPathSemiRing,
    P: ParentPolicy<G>,
{
    graph: &'a G,
    _marker: PhantomData<fn() -> (S, P)>,
}

impl<'a, G, S, P> ShortestPathBuilder<'a, G, S, P>
where
    G: Graph,
    S: ShortestPathSemiRing,
    P: ParentPolicy<G>,
{
    fn bfs_distance_core<M, I>(&self, sources: I, weight: &M) -> ShortestPathWithParent<G, S, P>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T + ?Sized,
        I: IntoIterator<Item = G::Vertex>,
    {
        let graph = self.graph;
        let mut dist = graph.construct_vmap(S::inf);
        let mut parent = P::init(graph);
        let mut deq = VecDeque::new();
        for source in sources.into_iter() {
            *graph.vmap_get_mut(&mut dist, source) = S::source();
            deq.push_back(source);
        }
        let zero = S::source();
        while let Some(u) = deq.pop_front() {
            for neighbor in graph.neighbors(u) {
                let v = neighbor.to;
                let w = weight(neighbor.label);
                let nd = S::mul(graph.vmap_get(&dist, u), &w);
                if S::add_assign(graph.vmap_get_mut(&mut dist, v), &nd) {
                    P::save_parent(graph, &mut parent, u, v);
                    if w == zero {
                        deq.push_front(v);
                    } else {
                        deq.push_back(v);
                    }
                }
            }
        }
        ShortestPathWithParent { dist, parent }
    }

    fn dijkstra_core<M, I>(&self, sources: I, weight: &M) -> ShortestPathWithParent<G, S, P>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T + ?Sized,
        I: IntoIterator<Item = G::Vertex>,
    {
        let graph = self.graph;
        let mut dist = graph.construct_vmap(S::inf);
        let mut parent = P::init(graph);
        let mut heap = BinaryHeap::new();
        for source in sources.into_iter() {
            *graph.vmap_get_mut(&mut dist, source) = S::source();
            heap.push(PartialIgnoredOrd(Reverse(S::source()), source));
        }
        while let Some(PartialIgnoredOrd(Reverse(d), u)) = heap.pop() {
            let current = graph.vmap_get(&dist, u);
            if current != &d {
                continue;
            }
            let d = current.clone();
            for neighbor in graph.neighbors(u) {
                let v = neighbor.to;
                let weight = weight(neighbor.label);
                let nd = S::mul(&d, &weight);
                if S::add_assign(graph.vmap_get_mut(&mut dist, v), &nd) {
                    P::save_parent(graph, &mut parent, u, v);
                    heap.push(PartialIgnoredOrd(Reverse(nd), v));
                }
            }
        }
        ShortestPathWithParent { dist, parent }
    }

    fn bellman_ford_core<M, I>(
        &self,
        sources: I,
        weight: &M,
        check: bool,
    ) -> Option<ShortestPathWithParent<G, S, P>>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T + ?Sized,
        I: IntoIterator<Item = G::Vertex>,
        P: ParentPolicy<G>,
    {
        let graph = self.graph;
        let mut dist = graph.construct_vmap(S::inf);
        let mut parent = P::init(graph);
        for source in sources.into_iter() {
            *graph.vmap_get_mut(&mut dist, source) = S::source();
        }
        let vsize = graph.vsize();
        for _ in 1..vsize {
            let mut updated = false;
            for u in graph.vertices() {
                for neighbor in graph.neighbors(u) {
                    let v = neighbor.to;
                    let weight = weight(neighbor.label);
                    let nd = S::mul(graph.vmap_get(&dist, u), &weight);
                    if S::add_assign(graph.vmap_get_mut(&mut dist, v), &nd) {
                        P::save_parent(graph, &mut parent, u, v);
                        updated = true;
                    }
                }
            }
            if !updated {
                break;
            }
        }
        if check {
            for u in graph.vertices() {
                for neighbor in graph.neighbors(u) {
                    let v = neighbor.to;
                    let weight = weight(neighbor.label);
                    let nd = S::mul(graph.vmap_get(&dist, u), &weight);
                    if S::add_assign(graph.vmap_get_mut(&mut dist, v), &nd) {
                        return None;
                    }
                }
            }
        }
        Some(ShortestPathWithParent { dist, parent })
    }
}

impl<'a, G, S> ShortestPathBuilder<'a, G, S, NoParent>
where
    G: Graph,
    S: ShortestPathSemiRing,
{
    pub fn with_parent(self) -> ShortestPathBuilder<'a, G, S, RecordParent>
    where
        G: VertexMap<Option<<G as Graph>::Vertex>>,
    {
        ShortestPathBuilder {
            graph: self.graph,
            _marker: PhantomData,
        }
    }

    pub fn bfs_distance<M, I>(&self, sources: I, weight: M) -> <G as VertexMap<S::T>>::Vmap
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T,
        I: IntoIterator<Item = G::Vertex>,
    {
        self.bfs_distance_core(sources, &weight).dist
    }

    pub fn dijkstra<M, I>(&self, sources: I, weight: M) -> <G as VertexMap<S::T>>::Vmap
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T,
        I: IntoIterator<Item = G::Vertex>,
    {
        self.dijkstra_core(sources, &weight).dist
    }

    pub fn bellman_ford<M, I>(
        &self,
        sources: I,
        weight: M,
        check: bool,
    ) -> Option<<G as VertexMap<S::T>>::Vmap>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T,
        I: IntoIterator<Item = G::Vertex>,
    {
        self.bellman_ford_core(sources, &weight, check)
            .map(|sp| sp.dist)
    }

    pub fn warshall_floyd_ap<M>(
        &self,
        weight: M,
    ) -> <G as VertexMap<<G as VertexMap<S::T>>::Vmap>>::Vmap
    where
        G: VertexMap<S::T, Vmap: Clone> + VertexMap<<G as VertexMap<S::T>>::Vmap>,
        M: Fn(G::Label) -> S::T,
    {
        let graph = self.graph;
        let mut dist = graph.construct_vmap(|| graph.construct_vmap(S::inf));
        for u in graph.vertices() {
            *graph.vmap_get_mut(graph.vmap_get_mut(&mut dist, u), u) = S::source();
        }
        for u in graph.vertices() {
            for neighbor in graph.neighbors(u) {
                let weight = weight(neighbor.label);
                S::add_assign(
                    graph.vmap_get_mut(graph.vmap_get_mut(&mut dist, u), neighbor.to),
                    &weight,
                );
            }
        }
        for k in graph.vertices() {
            for i in graph.vertices() {
                for j in graph.vertices() {
                    let d1 = graph.vmap_get(graph.vmap_get(&dist, i), k);
                    let d2 = graph.vmap_get(graph.vmap_get(&dist, k), j);
                    let nd = S::mul(d1, d2);
                    S::add_assign(graph.vmap_get_mut(graph.vmap_get_mut(&mut dist, i), j), &nd);
                }
            }
        }
        dist
    }
}

impl<'a, G, S> ShortestPathBuilder<'a, G, S, RecordParent>
where
    G: Graph + VertexMap<Option<<G as Graph>::Vertex>>,
    S: ShortestPathSemiRing,
{
    pub fn bfs_distance<M, I>(&self, sources: I, weight: M) -> ShortestPathWithParent<G, S>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T,
        I: IntoIterator<Item = G::Vertex>,
    {
        self.bfs_distance_core(sources, &weight)
    }

    pub fn dijkstra<M, I>(&self, sources: I, weight: M) -> ShortestPathWithParent<G, S>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T,
        I: IntoIterator<Item = G::Vertex>,
    {
        self.dijkstra_core(sources, &weight)
    }

    pub fn bellman_ford<M, I>(
        &self,
        sources: I,
        weight: M,
        check: bool,
    ) -> Option<ShortestPathWithParent<G, S>>
    where
        G: VertexMap<S::T>,
        M: Fn(G::Label) -> S::T,
        I: IntoIterator<Item = G::Vertex>,
    {
        self.bellman_ford_core(sources, &weight, check)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        num::Saturating,
        rand,
        tools::{PartialOrdExt, Xorshift},
    };
    use std::collections::HashMap;

    #[test]
    fn test_shortest_path() {
        struct Label(usize);

        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..30, m: 0..80, edges: [(0..n, 0..n); m], w: [0..100_000i64; m]);
            let g = DirectedSparseGraph::from_edges(n, edges.clone());
            let closure = UsizeGraph::new(n, |u| {
                edges
                    .iter()
                    .enumerate()
                    .filter(move |&(_, &(from, _))| from == u)
                    .map(|(eid, &(_, to))| (to, Label(eid)))
            });
            let weight: &dyn Fn(usize) -> Option<i64> = &|eid| Some(w[eid]);
            let dijkstra: Vec<_> = (0..n)
                .map(|src| g.option_sp_additive().dijkstra([src], weight))
                .collect();
            let closure_dijkstra: Vec<_> = (0..n)
                .map(|src| {
                    closure
                        .option_sp_additive()
                        .dijkstra([src], |label| Some(w[label.0]))
                })
                .collect();
            let bellman_ford: Vec<_> = (0..n)
                .map(|src| {
                    g.option_sp_additive()
                        .bellman_ford([src], |eid| Some(w[eid]), false)
                        .unwrap()
                })
                .collect();
            let warshall_floyd = g.option_sp_additive().warshall_floyd_ap(|eid| Some(w[eid]));
            assert_eq!(dijkstra, closure_dijkstra);
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
                        .bfs_distance([src], |eid| Some(w[eid]))
                })
                .collect();
            let warshall_floyd = g.option_sp_additive().warshall_floyd_ap(|eid| Some(w[eid]));
            assert_eq!(bfs, warshall_floyd);
        }
    }

    #[test]
    fn test_shortest_path_with_parent() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            rand!(rng, n: 1..100, m: 1..200, edges: [(0..n, 0..n); m], ub: 0..=1i64, w: [0..=ub * 100_000i64; m]);
            let mut cost: HashMap<_, _> = HashMap::new();
            for (&(u, v), &w) in edges.iter().zip(w.iter()) {
                cost.entry((u, v)).or_insert(w).chmin(w);
            }
            let g = DirectedSparseGraph::from_edges(n, edges);
            for src in 0..n {
                let bfs = g
                    .option_sp_additive()
                    .with_parent()
                    .bfs_distance([src], |eid| Some(w[eid]));
                let dijkstra = g
                    .option_sp_additive()
                    .with_parent()
                    .dijkstra([src], |eid| Some(w[eid]));
                let bellman_ford = g
                    .option_sp_additive()
                    .with_parent()
                    .bellman_ford([src], |eid| Some(w[eid]), false)
                    .unwrap();
                let dist = g.option_sp_additive().dijkstra([src], |eid| Some(w[eid]));
                assert_eq!(bfs.dist, dist);
                assert_eq!(dijkstra.dist, dist);
                assert_eq!(bellman_ford.dist, dist);
                for (target, &dist) in dist.iter().enumerate() {
                    match dist {
                        None => {
                            assert!(bfs.path_to(&g, target).is_none());
                            assert!(dijkstra.path_to(&g, target).is_none());
                            assert!(bellman_ford.path_to(&g, target).is_none());
                        }
                        Some(dist) => {
                            let path_bfs = bfs.path_to(&g, target).unwrap();
                            assert_eq!(*path_bfs.first().unwrap(), src);
                            assert_eq!(*path_bfs.last().unwrap(), target);
                            assert_eq!(
                                path_bfs
                                    .windows(2)
                                    .map(|w| cost[&(w[0], w[1])])
                                    .sum::<i64>(),
                                dist
                            );
                            let path_dijkstra = dijkstra.path_to(&g, target).unwrap();
                            assert_eq!(*path_dijkstra.first().unwrap(), src);
                            assert_eq!(*path_dijkstra.last().unwrap(), target);
                            assert_eq!(
                                path_dijkstra
                                    .windows(2)
                                    .map(|w| cost[&(w[0], w[1])])
                                    .sum::<i64>(),
                                dist
                            );
                            let path_bellman_ford = bellman_ford.path_to(&g, target).unwrap();
                            assert_eq!(*path_bellman_ford.first().unwrap(), src);
                            assert_eq!(*path_bellman_ford.last().unwrap(), target);
                            assert_eq!(
                                path_bellman_ford
                                    .windows(2)
                                    .map(|w| cost[&(w[0], w[1])])
                                    .sum::<i64>(),
                                dist
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_path_folding() {
        const N: usize = 5;
        for n in 1..=N {
            let all_edges: Vec<_> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            for bits in 0..1 << all_edges.len() {
                let edges: Vec<_> = all_edges
                    .iter()
                    .enumerate()
                    .filter_map(|(eid, &edge)| ((bits >> eid) & 1 == 1).then_some(edge))
                    .collect();
                let graph = DirectedSparseGraph::from_edges(n, edges.clone());
                for src in 0..n {
                    let mut expected = vec![(usize::MAX, 0u64); n];
                    expected[src] = (0, 1);
                    for u in 0..n {
                        if expected[u].0 == usize::MAX {
                            continue;
                        }
                        for &(_, v) in edges.iter().filter(|&&(from, _)| from == u) {
                            let nd = expected[u].0 + 1;
                            if nd < expected[v].0 {
                                expected[v] = (nd, expected[u].1);
                            } else if nd == expected[v].0 {
                                expected[v].1 += expected[u].1;
                            }
                        }
                    }
                    let actual = graph
                        .path_folding_sp_additive_addmul()
                        .dijkstra([src], |_| PartialIgnoredOrd(Saturating(1usize), 1u64));
                    for (actual, expected) in actual.iter().zip(expected) {
                        assert_eq!(actual.0, Saturating(expected.0));
                        assert_eq!(actual.1, expected.1);
                    }
                }
            }
        }
    }
}
