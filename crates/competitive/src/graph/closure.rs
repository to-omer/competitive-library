use super::{DirectedGraph, Graph, Neighbor, VertexMap};
use std::{collections::HashMap, hash::Hash, iter::Map, marker::PhantomData, ops::Range};

pub struct UsizeGraph<Fa> {
    vsize: usize,
    adj: Fa,
}
impl<Fa> UsizeGraph<Fa> {
    pub fn new(vsize: usize, adj: Fa) -> Self {
        Self { vsize, adj }
    }
}

impl<Fa, I, T> Graph for UsizeGraph<Fa>
where
    I: Iterator<Item = (usize, T)>,
    Fa: Fn(usize) -> I,
{
    type Vertex = usize;
    type Label = T;
    type Vertices<'g>
        = Range<usize>
    where
        Fa: 'g;
    type Neighbors<'g>
        = Map<I, fn((usize, T)) -> Neighbor<usize, T>>
    where
        Fa: 'g;

    #[inline]
    fn vsize(&self) -> usize {
        self.vsize
    }

    #[inline]
    fn vertices(&self) -> Self::Vertices<'_> {
        0..self.vsize
    }

    #[inline]
    fn neighbors(&self, vertex: Self::Vertex) -> Self::Neighbors<'_> {
        (self.adj)(vertex).map(Into::into)
    }
}

impl<Fa> DirectedGraph for UsizeGraph<Fa> where UsizeGraph<Fa>: Graph {}

impl<Fa, T> VertexMap<T> for UsizeGraph<Fa>
where
    Self: Graph<Vertex = usize>,
{
    type Vmap = Vec<T>;
    #[inline]
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        let mut v = Vec::with_capacity(self.vsize);
        v.resize_with(self.vsize, f);
        v
    }
    #[inline]
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vid: Self::Vertex) -> &'a T {
        &map[vid]
    }
    #[inline]
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vid: Self::Vertex) -> &'a mut T {
        &mut map[vid]
    }
}

pub struct ClosureGraph<V, Fv, Fa> {
    vs: Fv,
    adj: Fa,
    _marker: PhantomData<fn() -> V>,
}

impl<V, Fv, Fa> ClosureGraph<V, Fv, Fa> {
    pub fn new(vs: Fv, adj: Fa) -> Self {
        Self {
            vs,
            adj,
            _marker: PhantomData,
        }
    }
}

impl<V, Fv, Fa, Iv, Ia, T> Graph for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy,
    Iv: Iterator<Item = V>,
    Fv: Fn() -> Iv,
    Ia: Iterator<Item = (V, T)>,
    Fa: Fn(V) -> Ia,
{
    type Vertex = V;
    type Label = T;
    type Vertices<'g>
        = Iv
    where
        V: 'g,
        Fv: 'g,
        Fa: 'g;
    type Neighbors<'g>
        = Map<Ia, fn((V, T)) -> Neighbor<V, T>>
    where
        V: 'g,
        Fv: 'g,
        Fa: 'g;

    #[inline]
    fn vsize(&self) -> usize {
        (self.vs)().count()
    }

    #[inline]
    fn vertices(&self) -> Self::Vertices<'_> {
        (self.vs)()
    }

    #[inline]
    fn neighbors(&self, vertex: Self::Vertex) -> Self::Neighbors<'_> {
        (self.adj)(vertex).map(Into::into)
    }
}

impl<V, Fv, Fa> DirectedGraph for ClosureGraph<V, Fv, Fa> where Self: Graph {}

impl<V, Fv, Fa, T> VertexMap<T> for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy + Hash,
    T: Clone,
    Self: Graph<Vertex = V>,
{
    type Vmap = (HashMap<V, T>, T);
    #[inline]
    fn construct_vmap<F>(&self, mut f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        (HashMap::new(), f())
    }
    #[inline]
    fn vmap_get<'a>(&self, (map, val): &'a Self::Vmap, vid: Self::Vertex) -> &'a T {
        map.get(&vid).unwrap_or(val)
    }
    #[inline]
    fn vmap_get_mut<'a>(&self, (map, val): &'a mut Self::Vmap, vid: Self::Vertex) -> &'a mut T {
        map.entry(vid).or_insert_with(|| val.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{ClosureGraph, GridGraph, ShortestPathExt, UsizeGraph, VertexMap},
        num::Saturating,
        tools::Xorshift,
    };
    use std::iter::repeat_with;

    #[test]
    fn closure_graph_sssp() {
        let mut rng = Xorshift::default();
        const A: u64 = 1_000_000_000;
        let h = rng.rand(15) as usize + 1;
        let w = rng.rand(15) as usize + 1;

        let weight: Vec<_> = repeat_with(|| Saturating(rng.rand(A - 1) + 1))
            .take(8)
            .collect();
        let visitable: Vec<Vec<bool>> =
            repeat_with(|| repeat_with(|| rng.gen_bool(0.8)).take(w).collect())
                .take(h)
                .collect();

        let g = GridGraph::new_adj8(h, w);
        let g1 = UsizeGraph::new(h * w, |u| {
            g.adj8(g.unflat(u)).filter_map(|a| {
                if visitable[a.0.0][a.0.1] {
                    Some((g.flat(a.0), a.1))
                } else {
                    None
                }
            })
        });
        let g2 = ClosureGraph::new(
            || {
                (0..h)
                    .flat_map(|i| (0..w).map(move |j| (i, j)))
                    .filter(|&(i, j)| visitable[i][j])
            },
            |u| g.adj8(u).filter(|&((i, j), _)| visitable[i][j]),
        );
        for (i, visitable) in visitable.iter().enumerate() {
            for (j, visitable) in visitable.iter().enumerate() {
                assert_eq!((i, j), g.unflat(g.flat((i, j))));
                if !visitable {
                    continue;
                }
                let cost1 = g1
                    .standard_sp_additive()
                    .dijkstra([g.flat((i, j))], |dir| weight[dir as usize]);
                let cost2 = g2
                    .standard_sp_additive()
                    .dijkstra([(i, j)], |dir| weight[dir as usize]);
                for ni in 0..h {
                    for nj in 0..w {
                        assert_eq!(
                            g1.vmap_get(&cost1, g.flat((ni, nj))),
                            g2.vmap_get(&cost2, (ni, nj))
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn closure_graph_apsp() {
        let mut rng = Xorshift::default();
        const A: u64 = 1_000_000_000;
        let h = rng.rand(15) as usize + 1;
        let w = rng.rand(15) as usize + 1;

        let weight: Vec<_> = repeat_with(|| Saturating(rng.rand(A - 1) + 1))
            .take(8)
            .collect();

        let g = GridGraph::new_adj4(h, w);
        let cost: Vec<Vec<Vec<_>>> = (0..h)
            .map(|i| {
                (0..w)
                    .map(|j| {
                        g.standard_sp_additive()
                            .dijkstra([(i, j)], |dir| weight[dir as usize])
                    })
                    .collect()
            })
            .collect();
        let g2 = ClosureGraph::new(
            || (0..h).flat_map(|i| (0..w).map(move |j| (i, j))),
            |u| g.adj4(u),
        );
        let cost2 = g2
            .standard_sp_additive()
            .warshall_floyd_ap(|dir| weight[dir as usize]);
        for (i, row) in cost.iter().enumerate() {
            for (j, source_cost) in row.iter().enumerate() {
                for ni in 0..h {
                    for nj in 0..w {
                        assert_eq!(
                            g.vmap_get(source_cost, (ni, nj)),
                            g2.vmap_get(g2.vmap_get(&cost2, (i, j)), (ni, nj))
                        );
                    }
                }
            }
        }
    }
}
