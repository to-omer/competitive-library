use super::{
    AdditiveOperation, BitDpExt, Bounded, Graph, Monoid, PartialIgnoredOrd, ShortestPathSemiRing,
    UnionFind, VertexMap, Zero,
    shortest_path::{NoParent, OptionSp, RecordParent, StandardSp},
};
use std::{
    cmp::Reverse, collections::BinaryHeap, iter::repeat_with, marker::PhantomData, ops::Add,
};

pub enum SteinerTreeParent<V, L> {
    None,
    Split(usize),
    Edge(V, L),
}

pub trait SteinerTreeParentPolicy<G: Graph> {
    type State;
    type Label;
    fn init(graph: &G) -> Self::State;
    fn label(label: &G::Label) -> Self::Label;
    fn save_split(graph: &G, state: &mut Self::State, vertex: G::Vertex, subset: usize);
    fn save_parent(
        graph: &G,
        state: &mut Self::State,
        from: G::Vertex,
        to: G::Vertex,
        label: Self::Label,
    );
}

impl<G: Graph> SteinerTreeParentPolicy<G> for NoParent {
    type State = ();
    type Label = ();
    fn init(_graph: &G) {}
    fn label(_label: &G::Label) {}
    fn save_split(_graph: &G, _state: &mut (), _vertex: G::Vertex, _subset: usize) {}
    fn save_parent(_graph: &G, _state: &mut (), _from: G::Vertex, _to: G::Vertex, _label: ()) {}
}

impl<G> SteinerTreeParentPolicy<G> for RecordParent
where
    G: Graph<Label: Clone>
        + VertexMap<SteinerTreeParent<<G as Graph>::Vertex, <G as Graph>::Label>>,
{
    type State = <G as VertexMap<SteinerTreeParent<G::Vertex, G::Label>>>::Vmap;
    type Label = G::Label;
    fn init(graph: &G) -> Self::State {
        graph.construct_vmap(|| SteinerTreeParent::None)
    }
    fn label(label: &G::Label) -> Self::Label {
        label.clone()
    }
    fn save_split(graph: &G, state: &mut Self::State, vertex: G::Vertex, subset: usize) {
        *graph.vmap_get_mut(state, vertex) = SteinerTreeParent::Split(subset);
    }
    fn save_parent(
        graph: &G,
        state: &mut Self::State,
        from: G::Vertex,
        to: G::Vertex,
        label: Self::Label,
    ) {
        *graph.vmap_get_mut(state, to) = SteinerTreeParent::Edge(from, label);
    }
}

pub trait SteinerTreeExt: Graph {
    fn steiner_tree(&self) -> SteinerTreeBuilder<'_, Self>
    where
        Self: Sized,
    {
        SteinerTreeBuilder {
            graph: self,
            _marker: PhantomData,
        }
    }
}
impl<G> SteinerTreeExt for G where G: Graph + ?Sized {}

pub struct SteinerTreeBuilder<'g, G, S = (), P = NoParent>
where
    G: Graph,
    P: SteinerTreeParentPolicy<G>,
{
    graph: &'g G,
    _marker: PhantomData<fn() -> (S, P)>,
}

impl<'g, G, S, P> SteinerTreeBuilder<'g, G, S, P>
where
    G: Graph,
    P: SteinerTreeParentPolicy<G>,
{
    pub fn with_sp<T>(self) -> SteinerTreeBuilder<'g, G, T, P>
    where
        T: ShortestPathSemiRing,
    {
        SteinerTreeBuilder {
            graph: self.graph,
            _marker: PhantomData,
        }
    }

    pub fn with_standard_sp<M>(self) -> SteinerTreeBuilder<'g, G, StandardSp<M>, P>
    where
        M: Monoid<T: Bounded + Ord>,
    {
        self.with_sp()
    }

    pub fn with_standard_sp_additive<T>(
        self,
    ) -> SteinerTreeBuilder<'g, G, StandardSp<AdditiveOperation<T>>, P>
    where
        T: Clone + Zero + Add<Output = T> + Bounded + Ord,
    {
        self.with_sp()
    }

    pub fn with_option_sp<M>(self) -> SteinerTreeBuilder<'g, G, OptionSp<M>, P>
    where
        M: Monoid<T: Ord>,
    {
        self.with_sp()
    }

    pub fn with_option_sp_additive<T>(
        self,
    ) -> SteinerTreeBuilder<'g, G, OptionSp<AdditiveOperation<T>>, P>
    where
        T: Clone + Zero + Add<Output = T> + Ord,
    {
        self.with_sp()
    }
}

impl<'g, G, S> SteinerTreeBuilder<'g, G, S>
where
    G: Graph,
{
    pub fn with_parent(self) -> SteinerTreeBuilder<'g, G, S, RecordParent>
    where
        RecordParent: SteinerTreeParentPolicy<G>,
    {
        SteinerTreeBuilder {
            graph: self.graph,
            _marker: PhantomData,
        }
    }
}

impl<'g, G, S, P> SteinerTreeBuilder<'g, G, S, P>
where
    G: Graph + VertexMap<S::T>,
    S: ShortestPathSemiRing,
    P: SteinerTreeParentPolicy<G>,
{
    /// Requires commutative multiplication and nonnegative edge weights.
    pub fn solve<M, I>(&self, terminals: I, weight: M) -> SteinerTreeOutput<'g, S, G, P>
    where
        M: Fn(G::Label) -> S::T,
        I: ExactSizeIterator<Item = G::Vertex>,
    {
        let graph = self.graph;
        let tsize = terminals.len();
        let states = if tsize == 0 { 0 } else { 1 << tsize };
        let mut dp: Vec<_> = repeat_with(|| graph.construct_vmap(S::inf))
            .take(states)
            .collect();
        let mut parent: Vec<_> = repeat_with(|| P::init(graph)).take(states).collect();
        for (i, t) in terminals.enumerate() {
            *graph.vmap_get_mut(&mut dp[1 << i], t) = S::source();
        }
        let inf = S::inf();
        for bit in 1..states {
            let (prev, current) = dp.split_at_mut(bit);
            let dp = &mut current[0];
            for sub in bit.subsets().skip(1).take_while(|&sub| sub > bit ^ sub) {
                let left = &prev[sub];
                let right = &prev[bit ^ sub];
                for u in graph.vertices() {
                    let left = graph.vmap_get(left, u);
                    let right = graph.vmap_get(right, u);
                    if left != &inf && right != &inf {
                        let cost = S::mul(left, right);
                        if S::add_assign(graph.vmap_get_mut(dp, u), &cost) {
                            P::save_split(graph, &mut parent[bit], u, sub);
                        }
                    }
                }
            }
            let mut heap: BinaryHeap<_> = graph
                .vertices()
                .filter_map(|u| {
                    let d = graph.vmap_get(dp, u);
                    (d != &inf).then(|| PartialIgnoredOrd(Reverse(d.clone()), u))
                })
                .collect();
            while let Some(PartialIgnoredOrd(Reverse(d), u)) = heap.pop() {
                if graph.vmap_get(dp, u) != &d {
                    continue;
                }
                for neighbor in graph.neighbors(u) {
                    let v = neighbor.to;
                    let label = P::label(&neighbor.label);
                    let nd = S::mul(&d, &weight(neighbor.label));
                    if S::add_assign(graph.vmap_get_mut(dp, v), &nd) {
                        P::save_parent(graph, &mut parent[bit], u, v, label);
                        heap.push(PartialIgnoredOrd(Reverse(nd), v));
                    }
                }
            }
        }
        SteinerTreeOutput { graph, dp, parent }
    }
}

pub struct SteinerTreeOutput<'g, S, G, P = NoParent>
where
    G: Graph + VertexMap<S::T>,
    S: ShortestPathSemiRing,
    P: SteinerTreeParentPolicy<G>,
{
    graph: &'g G,
    dp: Vec<<G as VertexMap<S::T>>::Vmap>,
    parent: Vec<P::State>,
}

impl<S, G, P> SteinerTreeOutput<'_, S, G, P>
where
    G: Graph + VertexMap<S::T>,
    S: ShortestPathSemiRing,
    P: SteinerTreeParentPolicy<G>,
{
    pub fn minimum_from_source(&self, source: G::Vertex) -> S::T {
        match self.dp.last() {
            Some(dp) => self.graph.vmap_get(dp, source).clone(),
            None => S::source(),
        }
    }
}

impl<S, G> SteinerTreeOutput<'_, S, G, RecordParent>
where
    G: Graph<Label: Clone>
        + VertexMap<S::T>
        + VertexMap<usize>
        + VertexMap<SteinerTreeParent<<G as Graph>::Vertex, <G as Graph>::Label>>,
    S: ShortestPathSemiRing,
{
    /// For undirected graphs with nonnegative additive weights.
    /// Returns `None` when the terminals cannot be connected to `source`.
    pub fn edges_from_source(&self, source: G::Vertex) -> Option<Vec<G::Label>> {
        if self.dp.is_empty() {
            return Some(vec![]);
        }
        if self.minimum_from_source(source) == S::inf() {
            return None;
        }
        let graph = self.graph;
        let mut index = graph.construct_vmap(|| 0usize);
        for (i, u) in graph.vertices().enumerate() {
            *graph.vmap_get_mut(&mut index, u) = i;
        }
        let mut uf = UnionFind::new(graph.vsize());
        let mut edges = vec![];
        let mut stack = vec![(self.dp.len() - 1, source)];
        while let Some((bit, u)) = stack.pop() {
            match graph.vmap_get(&self.parent[bit], u) {
                SteinerTreeParent::None => {}
                &SteinerTreeParent::Split(sub) => {
                    stack.push((sub, u));
                    stack.push((bit ^ sub, u));
                }
                SteinerTreeParent::Edge(v, label) => {
                    if uf.unite(*graph.vmap_get(&index, u), *graph.vmap_get(&index, *v)) {
                        edges.push(label.clone());
                    }
                    stack.push((bit, *v));
                }
            }
        }
        Some(edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AdditiveOperation, graph::UndirectedSparseGraph, tools::Xorshift};

    #[test]
    fn test_steiner_tree() {
        let check = |n: usize, edges: &[(usize, usize, u64)], terminals: &[usize]| {
            let graph = UndirectedSparseGraph::from_edges(
                n,
                edges.iter().map(|&(u, v, _)| (u, v)).collect(),
            );
            let plain = graph
                .steiner_tree()
                .with_standard_sp::<AdditiveOperation<_>>()
                .solve(terminals.iter().copied(), |eid| edges[eid].2);
            let recorded = graph
                .steiner_tree()
                .with_standard_sp_additive()
                .with_parent()
                .solve(terminals.iter().copied(), |eid| edges[eid].2);
            let optional = graph
                .steiner_tree()
                .with_parent()
                .with_option_sp::<AdditiveOperation<_>>()
                .solve(terminals.iter().copied(), |eid| Some(edges[eid].2));
            for source in 0..n {
                let mut expected = None;
                for mask in 0..1usize << edges.len() {
                    let mut reachable = vec![false; n];
                    reachable[source] = true;
                    let mut stack = vec![source];
                    while let Some(u) = stack.pop() {
                        for (eid, &(a, b, _)) in edges.iter().enumerate() {
                            if mask >> eid & 1 == 0 {
                                continue;
                            }
                            let v = if a == u {
                                b
                            } else if b == u {
                                a
                            } else {
                                continue;
                            };
                            if !reachable[v] {
                                reachable[v] = true;
                                stack.push(v);
                            }
                        }
                    }
                    if terminals.iter().all(|&t| reachable[t]) {
                        let cost: u64 = edges
                            .iter()
                            .enumerate()
                            .filter(|&(eid, _)| mask >> eid & 1 != 0)
                            .map(|(_, e)| e.2)
                            .sum();
                        expected = Some(expected.map_or(cost, |best: u64| best.min(cost)));
                    }
                }
                assert_eq!(
                    plain.minimum_from_source(source),
                    expected.unwrap_or(u64::MAX)
                );
                assert_eq!(
                    recorded.minimum_from_source(source),
                    expected.unwrap_or(u64::MAX)
                );
                assert_eq!(optional.minimum_from_source(source), expected);
                for restored in [
                    recorded.edges_from_source(source),
                    optional.edges_from_source(source),
                ] {
                    assert_eq!(restored.is_some(), expected.is_some());
                    if let Some(restored) = restored {
                        let mut reachable = vec![false; n];
                        reachable[source] = true;
                        let mut stack = vec![source];
                        while let Some(u) = stack.pop() {
                            for &eid in &restored {
                                let (a, b, _) = edges[eid];
                                let v = if a == u {
                                    b
                                } else if b == u {
                                    a
                                } else {
                                    continue;
                                };
                                if !reachable[v] {
                                    reachable[v] = true;
                                    stack.push(v);
                                }
                            }
                        }
                        assert!(terminals.iter().all(|&t| reachable[t]));
                        assert_eq!(
                            restored.iter().map(|&eid| edges[eid].2).sum::<u64>(),
                            expected.unwrap()
                        );
                        assert_eq!(restored.len() + 1, reachable.iter().filter(|&&v| v).count());
                    }
                }
            }
        };
        for n in 1..=4 {
            let pairs: Vec<_> = (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect();
            for code in 0..3usize.pow(pairs.len() as u32) {
                let edges: Vec<_> = pairs
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &(u, v))| {
                        let value = code / 3usize.pow(i as u32) % 3;
                        (value != 0).then_some((u, v, value.saturating_sub(1) as u64))
                    })
                    .collect();
                for subset in 0..1usize << n {
                    let terminals: Vec<_> = (0..n).filter(|&u| subset >> u & 1 != 0).collect();
                    check(n, &edges, &terminals);
                }
            }
        }
        let mut rng = Xorshift::default();
        for n in 1..=7 {
            for case in 0..16 {
                let mut edges = match case % 4 {
                    0 => vec![],
                    1 => (1..n).map(|u| (u - 1, u, rng.rand(3))).collect(),
                    2 => (1..n).map(|u| (0, u, rng.rand(3))).collect(),
                    _ => (0..6)
                        .map(|_| {
                            (
                                rng.rand(n as u64) as usize,
                                rng.rand(n as u64) as usize,
                                rng.rand(3),
                            )
                        })
                        .collect(),
                };
                if case % 4 == 3 {
                    edges.push(edges[0]);
                    edges.push((0, 0, 0));
                }
                if case >= 8 {
                    for e in &mut edges {
                        e.2 *= 1_000_000_000;
                    }
                }
                let subset = if case % 4 == 0 {
                    0
                } else if case % 4 == 1 {
                    (1 << n) - 1
                } else {
                    rng.rand(1 << n)
                };
                let mut terminals: Vec<_> = (0..n).filter(|&u| subset >> u & 1 != 0).collect();
                if let Some(&t) = terminals.first() {
                    terminals.push(t);
                }
                check(n, &edges, &terminals);
            }
        }
    }
}
