use super::{Graph, Neighbor, VertexMap};
use std::{iter::Map, marker::PhantomData, ops::Range};
#[cfg(verus_keep_ghost)]
use vstd::{arithmetic::div_mod::*, prelude::*, std_specs::iter::IteratorSpec};

#[derive(Debug, Clone, Copy)]
pub struct GridGraph<A> {
    pub height: usize,
    pub width: usize,
    _marker: PhantomData<fn() -> A>,
}

impl GridGraph<Adj4> {
    pub fn new_adj4(height: usize, width: usize) -> Self {
        Self::new(height, width)
    }
    pub fn adj4(&self, vid: (usize, usize)) -> GridAdjacency<'_, Adj4> {
        GridAdjacency {
            g: self,
            xy: vid,
            diter: GridDirectionIter::default(),
            _marker: PhantomData,
        }
    }
}
impl GridGraph<Adj8> {
    pub fn new_adj8(height: usize, width: usize) -> Self {
        Self::new(height, width)
    }
    pub fn adj8(&self, vid: (usize, usize)) -> GridAdjacency<'_, Adj8> {
        GridAdjacency {
            g: self,
            xy: vid,
            diter: GridDirectionIter::default(),
            _marker: PhantomData,
        }
    }
}

impl<A> GridGraph<A> {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            _marker: PhantomData,
        }
    }
    #[inline]
    pub fn move_by_diff(&self, xy: (usize, usize), dxdy: (isize, isize)) -> Option<(usize, usize)> {
        let nx = xy.0.wrapping_add(dxdy.0 as usize);
        let ny = xy.1.wrapping_add(dxdy.1 as usize);
        if nx < self.height && ny < self.width {
            Some((nx, ny))
        } else {
            None
        }
    }
    #[inline]
    pub fn flat(&self, xy: (usize, usize)) -> usize {
        xy.0 * self.width + xy.1
    }
    #[inline]
    pub fn unflat(&self, pos: usize) -> (usize, usize) {
        (pos / self.width, pos % self.width)
    }
}

impl<A> Graph for GridGraph<A>
where
    GridDirectionIter<A>: Iterator<Item = GridDirection>,
{
    type Vertex = (usize, usize);
    type Label = GridDirection;
    type Vertices<'g>
        = GridVertices
    where
        A: 'g;
    type Neighbors<'g>
        = Map<
        GridAdjacency<'g, A>,
        fn(((usize, usize), GridDirection)) -> Neighbor<(usize, usize), GridDirection>,
    >
    where
        A: 'g;

    #[inline]
    fn vsize(&self) -> usize {
        self.height * self.width
    }

    #[inline]
    fn vertices(&self) -> Self::Vertices<'_> {
        GridVertices {
            xrange: 0..self.height,
            yrange: 0..self.width,
        }
    }

    #[inline]
    fn neighbors(&self, vertex: Self::Vertex) -> Self::Neighbors<'_> {
        GridAdjacency {
            g: self,
            xy: vertex,
            diter: GridDirectionIter::default(),
            _marker: PhantomData,
        }
        .map(Into::into)
    }
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verifier::allow(autoderive_clone_without_spec))]
#[derive(Debug, Clone)]
pub struct GridVertices {
    xrange: Range<usize>,
    yrange: Range<usize>,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Iterator for GridVertices {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        #[cfg_attr(verus_keep_ghost, verus_spec(
            invariant self@ == old(self)@,
            decreases self.xrange.end - self.xrange.start,
        ))]
        loop {
            if self.xrange.start >= self.xrange.end {
                return None;
            }
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost before = *self; }
            if let Some(ny) = self.yrange.next() {
                #[cfg(verus_keep_ghost)]
                proof! {
                    let x = self.xrange.start as int;
                    let end = self.xrange.end as int;
                    let width = self.yrange.end as int;
                    assert(x * width + ny < end * width) by(nonlinear_arith)
                        requires x < end, 0 <= ny < width;
                    lemma_fundamental_div_mod_converse(before.cursor_index(), width, x, ny as int);
                    assert(before@[0] == (self.xrange.start, ny));
                    assert(self@ =~= before@.drop_first());
                }
                return Some((self.xrange.start, ny));
            }
            self.yrange.start = 0;
            self.xrange.start += 1;
            #[cfg(verus_keep_ghost)]
            proof! {
                let x = before.xrange.start as int;
                let width = self.yrange.end as int;
                assert((x + 1) * width == x * width + width) by(nonlinear_arith);
                assert(self.cursor_index() == before.cursor_index());
                assert(self@ =~= before@);
            }
        }
    }
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GridDirection {
    U = 0isize,
    L = 1isize,
    R = 2isize,
    D = 3isize,
    UL = 4isize,
    UR = 5isize,
    DL = 6isize,
    DR = 7isize,
}

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl GridDirection {
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        ensures (result.0 as int, result.1 as int) == self.offset(1),
    ))]
    pub fn dxdy(self) -> (isize, isize) {
        match self {
            GridDirection::U => (-1, 0),
            GridDirection::L => (0, -1),
            GridDirection::R => (0, 1),
            GridDirection::D => (1, 0),
            GridDirection::UL => (-1, -1),
            GridDirection::UR => (-1, 1),
            GridDirection::DL => (1, -1),
            GridDirection::DR => (1, 1),
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires d <= isize::MAX,
        ensures (result.0 as int, result.1 as int) == self.offset(d as int),
    ))]
    pub fn ndxdy(self, d: usize) -> (isize, isize) {
        let d = d as isize;
        match self {
            GridDirection::U => (-d, 0),
            GridDirection::L => (0, -d),
            GridDirection::R => (0, d),
            GridDirection::D => (d, 0),
            GridDirection::UL => (-d, -d),
            GridDirection::UR => (-d, d),
            GridDirection::DL => (d, -d),
            GridDirection::DR => (d, d),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Adj4 {}
#[derive(Debug, Clone, Copy)]
pub enum Adj8 {}

#[derive(Debug, Clone)]
pub struct GridDirectionIter<A> {
    dir: Option<GridDirection>,
    _marker: PhantomData<fn() -> A>,
}
impl<A> Default for GridDirectionIter<A> {
    fn default() -> Self {
        Self {
            dir: Some(GridDirection::U),
            _marker: PhantomData,
        }
    }
}

impl Iterator for GridDirectionIter<Adj4> {
    type Item = GridDirection;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dir) = &mut self.dir {
            let cdir = Some(*dir);
            self.dir = match dir {
                GridDirection::U => Some(GridDirection::L),
                GridDirection::L => Some(GridDirection::R),
                GridDirection::R => Some(GridDirection::D),
                _ => None,
            };
            cdir
        } else {
            None
        }
    }
}
impl Iterator for GridDirectionIter<Adj8> {
    type Item = GridDirection;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dir) = &mut self.dir {
            let cdir = Some(*dir);
            self.dir = match dir {
                GridDirection::U => Some(GridDirection::L),
                GridDirection::L => Some(GridDirection::R),
                GridDirection::R => Some(GridDirection::D),
                GridDirection::D => Some(GridDirection::UL),
                GridDirection::UL => Some(GridDirection::UR),
                GridDirection::UR => Some(GridDirection::DL),
                GridDirection::DL => Some(GridDirection::DR),
                GridDirection::DR => None,
            };
            cdir
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridAdjacency<'g, A> {
    g: &'g GridGraph<A>,
    xy: (usize, usize),
    diter: GridDirectionIter<A>,
    _marker: PhantomData<fn() -> A>,
}

impl<A> Iterator for GridAdjacency<'_, A>
where
    GridDirectionIter<A>: Iterator<Item = GridDirection>,
{
    type Item = ((usize, usize), GridDirection);
    fn next(&mut self) -> Option<Self::Item> {
        for dir in self.diter.by_ref() {
            match self.g.move_by_diff(self.xy, dir.dxdy()) {
                Some(nxy) => return Some((nxy, dir)),
                None => continue,
            }
        }
        None
    }
}

impl<A, T> VertexMap<T> for GridGraph<A>
where
    Self: Graph<Vertex = (usize, usize)>,
{
    type Vmap = Vec<T>;

    #[inline]
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        let mut map = Vec::with_capacity(self.height * self.width);
        map.resize_with(self.height * self.width, f);
        map
    }

    #[inline]
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, (x, y): Self::Vertex) -> &'a T {
        assert!(x < self.height, "expected 0..{}, but {}", self.height, x);
        assert!(y < self.width, "expected 0..{}, but {}", self.width, y);
        &map[x * self.width + y]
    }

    #[inline]
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, (x, y): Self::Vertex) -> &'a mut T {
        assert!(x < self.height, "expected 0..{}, but {}", self.height, x);
        assert!(y < self.width, "expected 0..{}, but {}", self.width, y);
        &mut map[x * self.width + y]
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl GridVertices {
        spec fn cursor_index(&self) -> int {
            self.xrange.start as int * self.yrange.end
                + if self.yrange.start < self.yrange.end { self.yrange.start as int } else { self.yrange.end as int }
        }
    }

    impl View for GridVertices {
        type V = Seq<(usize, usize)>;

        closed spec fn view(&self) -> Self::V {
            let position = self.cursor_index();
            let width = self.yrange.end as int;
            let length = if self.xrange.start < self.xrange.end {
                (self.xrange.end as int * width - position) as nat
            } else { 0 };
            Seq::new(length, |i: int| (((position + i) / width) as usize, ((position + i) % width) as usize))
        }
    }

    impl vstd::std_specs::iter::IteratorSpecImpl for GridVertices {
        open spec fn obeys_prophetic_iter_laws(&self) -> bool { true }
        open spec fn remaining(&self) -> Seq<(usize, usize)> { self@ }
        open spec fn will_return_none(&self) -> bool { true }
        open spec fn decrease(&self) -> Option<nat> { Some(self@.len()) }
        open spec fn peek(&self, index: int) -> Option<(usize, usize)> {
            if 0 <= index < self@.len() { Some(self@[index]) } else { None }
        }
    }

    impl GridDirection {
        pub open spec fn offset(self, distance: int) -> (int, int) {
            match self {
                GridDirection::U => (-distance, 0), GridDirection::L => (0, -distance),
                GridDirection::R => (0, distance), GridDirection::D => (distance, 0),
                GridDirection::UL => (-distance, -distance), GridDirection::UR => (-distance, distance),
                GridDirection::DL => (distance, -distance), GridDirection::DR => (distance, distance),
            }
        }
    }
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
        proof fn row_major_coordinates(height: usize, width: usize, x: usize, y: usize)
            requires x < height, y < width,
        {
            let vertices = GridVertices { xrange: 0..height, yrange: 0..width };
            let index = x as int * width + y;
            assert(0 <= x as int * width + y < height as int * width) by(nonlinear_arith)
                requires 0 <= x < height, 0 <= y < width;
            lemma_fundamental_div_mod_converse(index, width as int, x as int, y as int);
            #[verifier::proof_note("For arbitrary grid dimensions, the vertex iterator has exactly height times width elements in row-major order, and every valid coordinate occurs at its flattened index; the model does not require the cell count to fit usize")]
            assert(vertices@.len() == height as int * width && vertices@[index] == (x, y));
        }

        proof fn row_major_index(height: usize, width: usize, index: int)
            requires 0 <= index < height as int * width,
        {
            let vertices = GridVertices { xrange: 0..height, yrange: 0..width };
            assert(width > 0 && height > 0) by(nonlinear_arith)
                requires 0 <= height, 0 <= width, 0 <= index < height as int * width;
            lemma_div_pos_is_pos(index, width as int);
            lemma_mod_bound(index, width as int);
            lemma_fundamental_div_mod(index, width as int);
            assert(index / (width as int) < height) by(nonlinear_arith)
                requires width > 0, index < height as int * width,
                    index == width as int * (index / (width as int)) + index % (width as int),
                    index % (width as int) >= 0;
            let xy = vertices@[index];
            assert(xy.0 as int == index / (width as int));
            assert(xy.1 as int == index % (width as int));
            assert(xy.0 as int * width + xy.1 == index) by(nonlinear_arith)
                requires xy.0 as int == index / (width as int),
                    xy.1 as int == index % (width as int),
                    index == width as int * (index / (width as int)) + index % (width as int);
            #[verifier::proof_note("Every position in the arbitrary-length row-major vertex sequence yields an in-range coordinate whose flattened index is that position, so the enumeration contains no extra or duplicate vertices")]
            assert(xy.0 < height && xy.1 < width && xy.0 as int * width + xy.1 == index);
        }

        fn first_vertex(height: usize, width: usize) {
            let mut vertices = GridVertices { xrange: 0..height, yrange: 0..width };
            let first = vertices.next();
            proof {
                if height > 0 && width > 0 {
                    assert(height as int * width > 0) by(nonlinear_arith)
                        requires height > 0, width > 0;
                    lemma_fundamental_div_mod_converse(0, width as int, 0, 0);
                }
                #[verifier::proof_note("A grid with either dimension zero yields no vertices; every nonempty grid starts at coordinate (0, 0)")]
                assert(first == if height == 0 || width == 0 { None } else { Some((0usize, 0usize)) });
            }
        }

        fn direction_components(direction: GridDirection, distance: usize)
            requires distance <= isize::MAX,
        {
            let unit = direction.dxdy();
            let scaled = direction.ndxdy(distance);
            proof {
                #[verifier::proof_note("All eight grid directions have the stated unit offsets, and ndxdy scales both components by every nonnegative distance representable as isize, including zero")]
                assert(scaled.0 as int == unit.0 as int * distance && scaled.1 as int == unit.1 as int * distance);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GridGraph;
    use crate::{
        graph::{ShortestPathExt, VertexMap},
        num::Saturating,
        tools::Xorshift,
    };

    #[test]
    fn grid_graph_apsp() {
        let mut rng = Xorshift::default();
        const A: u64 = 1_000_000_000;
        for _ in 0..30 {
            let h = rng.rand(8) as usize + 1;
            let w = rng.rand(8) as usize + 1;

            let weight: Vec<_> = std::iter::repeat_with(|| Saturating(rng.rand(A - 1) + 1))
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
            let cost2: Vec<Vec<_>> = g
                .standard_sp_additive()
                .warshall_floyd_ap(|dir| weight[dir as usize]);
            for (i, row) in cost.iter().enumerate() {
                for (j, source_cost) in row.iter().enumerate() {
                    for ni in 0..h {
                        for nj in 0..w {
                            assert_eq!(
                                g.vmap_get(source_cost, (ni, nj)),
                                g.vmap_get(g.vmap_get(&cost2, (i, j)), (ni, nj))
                            );
                        }
                    }
                }
            }

            let g = GridGraph::new_adj8(h, w);
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
            let cost2: Vec<Vec<_>> = g
                .standard_sp_additive()
                .warshall_floyd_ap(|dir| weight[dir as usize]);
            for (i, row) in cost.iter().enumerate() {
                for (j, source_cost) in row.iter().enumerate() {
                    for ni in 0..h {
                        for nj in 0..w {
                            assert_eq!(
                                g.vmap_get(source_cost, (ni, nj)),
                                g.vmap_get(g.vmap_get(&cost2, (i, j)), (ni, nj))
                            );
                        }
                    }
                }
            }
        }
    }
}
