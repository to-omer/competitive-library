use super::{
    Adjacencies, AdjacenciesWithValue, AdjacencyView, AdjacencyViewIterFromValue, GraphBase,
    VIndexWithValue, VertexMap, VertexView, Vertices,
};
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

impl<Fa> GraphBase for UsizeGraph<Fa> {
    type VIndex = usize;
}

impl<Fa> Vertices for UsizeGraph<Fa> {
    type VIter<'g>
        = Range<usize>
    where
        Fa: 'g;
    fn vertices(&self) -> Self::VIter<'_> {
        0..self.vsize
    }
}

impl<Fa, I, T> Adjacencies for UsizeGraph<Fa>
where
    I: Iterator<Item = (usize, T)>,
    Fa: Fn(usize) -> I,
    T: Clone,
{
    type AIndex = VIndexWithValue<usize, T>;
    type AIter<'g>
        = Map<I, fn((usize, T)) -> VIndexWithValue<usize, T>>
    where
        Fa: 'g;

    fn adjacencies(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        (self.adj)(vid).map(|a| a.into())
    }
}
impl<Fa, I, T> AdjacenciesWithValue<T> for UsizeGraph<Fa>
where
    I: Iterator<Item = (usize, T)>,
    Fa: Fn(usize) -> I,
    T: Clone,
{
    type AIndex = VIndexWithValue<usize, T>;
    type AIter<'g>
        = Map<I, fn((usize, T)) -> VIndexWithValue<usize, T>>
    where
        Fa: 'g;

    fn adjacencies_with_value(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        (self.adj)(vid).map(|a| a.into())
    }
}

impl<Fa, T> VertexMap<T> for UsizeGraph<Fa> {
    type Vmap = Vec<T>;
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        let mut v = Vec::with_capacity(self.vsize);
        v.resize_with(self.vsize, f);
        v
    }
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vid: Self::VIndex) -> &'a T {
        assert!(vid < self.vsize, "expected 0..{}, but {}", self.vsize, vid);
        unsafe { map.get_unchecked(vid) }
    }
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vid: Self::VIndex) -> &'a mut T {
        assert!(vid < self.vsize, "expected 0..{}, but {}", self.vsize, vid);
        unsafe { map.get_unchecked_mut(vid) }
    }
}
impl<Fa, T> VertexView<Vec<T>, T> for UsizeGraph<Fa>
where
    T: Clone,
{
    fn vview(&self, map: &Vec<T>, vid: Self::VIndex) -> T {
        self.vmap_get(map, vid).clone()
    }
}
impl<Fa, T> VertexView<[T], T> for UsizeGraph<Fa>
where
    T: Clone,
{
    fn vview(&self, map: &[T], vid: Self::VIndex) -> T {
        assert!(vid < self.vsize, "expected 0..{}, but {}", self.vsize, vid);
        unsafe { map.get_unchecked(vid) }.clone()
    }
}
impl<'a, Fa, M, I, T, U> AdjacencyView<'a, M, U> for UsizeGraph<Fa>
where
    I: Iterator<Item = (usize, T)>,
    Fa: Fn(usize) -> I,
    T: Clone,
    M: 'a + Fn(T) -> U,
{
    type AViewIter<'g>
        = AdjacencyViewIterFromValue<'g, 'a, Self, M, T, U>
    where
        Fa: 'g;
    fn aviews<'g>(&'g self, map: &'a M, vid: Self::VIndex) -> Self::AViewIter<'g> {
        AdjacencyViewIterFromValue::new(self.adjacencies_with_value(vid), map)
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

impl<V, Fv, Fa> GraphBase for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy,
{
    type VIndex = V;
}

impl<V, Fv, Fa, Iv> Vertices for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy,
    Iv: Iterator<Item = V>,
    Fv: Fn() -> Iv,
{
    type VIter<'g>
        = Iv
    where
        V: 'g,
        Fv: 'g,
        Fa: 'g;
    fn vertices(&self) -> Self::VIter<'_> {
        (self.vs)()
    }
}

impl<V, Fv, Fa, Ia, T> Adjacencies for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy,
    Ia: Iterator<Item = (V, T)>,
    Fa: Fn(V) -> Ia,
    T: Clone,
{
    type AIndex = VIndexWithValue<V, T>;
    type AIter<'g>
        = Map<Ia, fn((V, T)) -> VIndexWithValue<V, T>>
    where
        V: 'g,
        Fv: 'g,
        Fa: 'g;

    fn adjacencies(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        (self.adj)(vid).map(|a| a.into())
    }
}
impl<V, Fv, Fa, Ia, T> AdjacenciesWithValue<T> for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy,
    Ia: Iterator<Item = (V, T)>,
    Fa: Fn(V) -> Ia,
    T: Clone,
{
    type AIndex = VIndexWithValue<V, T>;
    type AIter<'g>
        = Map<Ia, fn((V, T)) -> VIndexWithValue<V, T>>
    where
        V: 'g,
        Fv: 'g,
        Fa: 'g;

    fn adjacencies_with_value(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        (self.adj)(vid).map(|a| a.into())
    }
}

impl<V, Fv, Fa, T> VertexMap<T> for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy + Hash,
    T: Clone,
{
    type Vmap = (HashMap<V, T>, T);
    fn construct_vmap<F>(&self, mut f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        (HashMap::new(), f())
    }
    fn vmap_get<'a>(&self, (map, val): &'a Self::Vmap, vid: Self::VIndex) -> &'a T {
        map.get(&vid).unwrap_or(val)
    }
    fn vmap_get_mut<'a>(&self, (map, val): &'a mut Self::Vmap, vid: Self::VIndex) -> &'a mut T {
        map.entry(vid).or_insert_with(|| val.clone())
    }
}
impl<V, Fv, Fa, T> VertexView<(HashMap<V, T>, T), T> for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy + Hash,
    T: Clone,
{
    fn vview(&self, map: &(HashMap<V, T>, T), vid: Self::VIndex) -> T {
        self.vmap_get(map, vid).clone()
    }
}
impl<'a, V, Fv, Fa, M, Ia, T, U> AdjacencyView<'a, M, U> for ClosureGraph<V, Fv, Fa>
where
    V: Eq + Copy,
    Ia: Iterator<Item = (V, T)>,
    Fa: Fn(V) -> Ia,
    T: Clone,
    M: 'a + Fn(T) -> U,
{
    type AViewIter<'g>
        = AdjacencyViewIterFromValue<'g, 'a, Self, M, T, U>
    where
        Fa: 'g,
        Fv: 'g,
        V: 'g;
    fn aviews<'g>(&'g self, map: &'a M, vid: Self::VIndex) -> Self::AViewIter<'g> {
        AdjacencyViewIterFromValue::new(self.adjacencies_with_value(vid), map)
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
                if *g.vmap_get(&visitable, a.0) {
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
                    .dijkstra_ss(g.flat((i, j)), &|dir| weight[dir as usize]);
                let cost2 = g2
                    .standard_sp_additive()
                    .dijkstra_ss((i, j), &|dir| weight[dir as usize]);
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
        let cost: Vec<Vec<Vec<Vec<_>>>> = (0..h)
            .map(|i| {
                (0..w)
                    .map(|j| {
                        g.standard_sp_additive()
                            .dijkstra_ss((i, j), &|dir| weight[dir as usize])
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
            .warshall_floyd_ap(&|dir| weight[dir as usize]);
        for i in 0..h {
            for j in 0..w {
                for ni in 0..h {
                    for nj in 0..w {
                        assert_eq!(
                            g.vmap_get(g.vmap_get(&cost, (i, j)), (ni, nj)),
                            g2.vmap_get(g2.vmap_get(&cost2, (i, j)), (ni, nj))
                        );
                    }
                }
            }
        }
    }
}
