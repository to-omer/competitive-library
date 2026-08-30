use super::{BitDpExt, Graph, PartialIgnoredOrd, ShortestPathSemiRing, VertexMap};
use std::{cmp::Reverse, collections::BinaryHeap, iter::repeat_with};

pub trait SteinerTreeExt: Graph {
    fn steiner_tree<S, M, I>(&self, terminals: I, weight: &M) -> SteinerTreeOutput<'_, S, Self>
    where
        Self: VertexMap<S::T>,
        S: ShortestPathSemiRing,
        M: Fn(Self::Label) -> S::T + ?Sized,
        I: IntoIterator<Item = Self::Vertex> + ExactSizeIterator,
    {
        let tsize = terminals.len();
        if tsize == 0 {
            return SteinerTreeOutput {
                g: self,
                dp: vec![],
            };
        }
        let mut dp: Vec<_> = repeat_with(|| self.construct_vmap(S::inf))
            .take(1 << tsize)
            .collect();
        for (i, t) in terminals.into_iter().enumerate() {
            *self.vmap_get_mut(&mut dp[1 << i], t) = S::source();
        }
        for bit in 1..1 << tsize {
            for u in self.vertices() {
                for sub in bit.subsets() {
                    if sub != 0 {
                        let cost =
                            S::mul(self.vmap_get(&dp[sub], u), self.vmap_get(&dp[bit ^ sub], u));
                        S::add_assign(self.vmap_get_mut(&mut dp[bit], u), &cost);
                    }
                }
            }
            let dp = &mut dp[bit];
            let mut heap: BinaryHeap<_> = self
                .vertices()
                .map(|u| PartialIgnoredOrd(Reverse(self.vmap_get(dp, u).clone()), u))
                .collect();
            while let Some(PartialIgnoredOrd(Reverse(d), u)) = heap.pop() {
                if self.vmap_get(dp, u) != &d {
                    continue;
                }
                for neighbor in self.neighbors(u) {
                    let v = neighbor.to;
                    let weight = weight(neighbor.label);
                    let nd = S::mul(&d, &weight);
                    if S::add_assign(self.vmap_get_mut(dp, v), &nd) {
                        heap.push(PartialIgnoredOrd(Reverse(nd), v));
                    }
                }
            }
        }
        SteinerTreeOutput { g: self, dp }
    }
}
impl<G> SteinerTreeExt for G where G: Graph + ?Sized {}
pub struct SteinerTreeOutput<'g, S, G>
where
    G: Graph + VertexMap<S::T> + ?Sized,
    S: ShortestPathSemiRing,
{
    g: &'g G,
    dp: Vec<G::Vmap>,
}
impl<S, G> SteinerTreeOutput<'_, S, G>
where
    G: Graph + VertexMap<S::T> + ?Sized,
    S: ShortestPathSemiRing,
{
    pub fn minimum_from_source(&self, source: G::Vertex) -> S::T {
        match self.dp.last() {
            Some(dp) => self.g.vmap_get(dp, source).clone(),
            None => S::source(),
        }
    }
}
