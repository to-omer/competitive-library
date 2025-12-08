use super::{
    Adjacencies, AdjacenciesWithValue, AdjacencyView, AdjacencyViewIterFromValue, GraphBase,
    VIndexWithValue, VertexMap, VertexView, Vertices,
};
use std::{iter::Map, marker::PhantomData, ops::Range};

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
    pub fn move_by_diff(&self, xy: (usize, usize), dxdy: (isize, isize)) -> Option<(usize, usize)> {
        let nx = xy.0.wrapping_add(dxdy.0 as usize);
        let ny = xy.1.wrapping_add(dxdy.1 as usize);
        if nx < self.height && ny < self.width {
            Some((nx, ny))
        } else {
            None
        }
    }
    pub fn flat(&self, xy: (usize, usize)) -> usize {
        xy.0 * self.width + xy.1
    }
    pub fn unflat(&self, pos: usize) -> (usize, usize) {
        (pos / self.width, pos % self.width)
    }
}

impl<A> GraphBase for GridGraph<A> {
    type VIndex = (usize, usize);
}

impl<A> Vertices for GridGraph<A> {
    type VIter<'g>
        = GridVertices
    where
        A: 'g;
    fn vertices(&self) -> Self::VIter<'_> {
        GridVertices {
            xrange: 0..self.height,
            yrange: 0..self.width,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridVertices {
    xrange: Range<usize>,
    yrange: Range<usize>,
}

impl Iterator for GridVertices {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.xrange.start >= self.xrange.end {
            None
        } else if let Some(ny) = self.yrange.next() {
            Some((self.xrange.start, ny))
        } else {
            self.yrange.start = 0;
            self.xrange.start += 1;
            self.next()
        }
    }
}

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

impl GridDirection {
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

impl Adjacencies for GridGraph<Adj4> {
    type AIndex = VIndexWithValue<(usize, usize), GridDirection>;
    type AIter<'g> = Map<
        GridAdjacency<'g, Adj4>,
        fn(((usize, usize), GridDirection)) -> VIndexWithValue<(usize, usize), GridDirection>,
    >;
    fn adjacencies(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        self.adj4(vid).map(Into::into)
    }
}
impl Adjacencies for GridGraph<Adj8> {
    type AIndex = VIndexWithValue<(usize, usize), GridDirection>;
    type AIter<'g> = Map<
        GridAdjacency<'g, Adj8>,
        fn(((usize, usize), GridDirection)) -> VIndexWithValue<(usize, usize), GridDirection>,
    >;
    fn adjacencies(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        self.adj8(vid).map(Into::into)
    }
}

impl AdjacenciesWithValue<GridDirection> for GridGraph<Adj4> {
    type AIndex = VIndexWithValue<(usize, usize), GridDirection>;
    type AIter<'g> = Map<
        GridAdjacency<'g, Adj4>,
        fn(((usize, usize), GridDirection)) -> VIndexWithValue<(usize, usize), GridDirection>,
    >;
    fn adjacencies_with_value(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        self.adjacencies(vid)
    }
}
impl AdjacenciesWithValue<GridDirection> for GridGraph<Adj8> {
    type AIndex = VIndexWithValue<(usize, usize), GridDirection>;
    type AIter<'g> = Map<
        GridAdjacency<'g, Adj8>,
        fn(((usize, usize), GridDirection)) -> VIndexWithValue<(usize, usize), GridDirection>,
    >;
    fn adjacencies_with_value(&self, vid: Self::VIndex) -> Self::AIter<'_> {
        self.adjacencies(vid)
    }
}

impl<'a, M, T> AdjacencyView<'a, M, T> for GridGraph<Adj4>
where
    M: 'a + Fn(GridDirection) -> T,
{
    type AViewIter<'g> = AdjacencyViewIterFromValue<'g, 'a, Self, M, GridDirection, T>;
    fn aviews<'g>(&'g self, map: &'a M, vid: Self::VIndex) -> Self::AViewIter<'g> {
        AdjacencyViewIterFromValue::new(self.adjacencies(vid), map)
    }
}
impl<'a, M, T> AdjacencyView<'a, M, T> for GridGraph<Adj8>
where
    M: 'a + Fn(GridDirection) -> T,
{
    type AViewIter<'g> = AdjacencyViewIterFromValue<'g, 'a, Self, M, GridDirection, T>;
    fn aviews<'g>(&'g self, map: &'a M, vid: Self::VIndex) -> Self::AViewIter<'g> {
        AdjacencyViewIterFromValue::new(self.adjacencies(vid), map)
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

impl<A, T> VertexMap<T> for GridGraph<A> {
    type Vmap = Vec<Vec<T>>;
    fn construct_vmap<F>(&self, mut f: F) -> Self::Vmap
    where
        F: FnMut() -> T,
    {
        (0..self.height)
            .map(|_| (0..self.width).map(|_| f()).collect())
            .collect()
    }
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, (x, y): Self::VIndex) -> &'a T {
        assert!(x < self.height, "expected 0..{}, but {}", self.height, x);
        assert!(y < self.width, "expected 0..{}, but {}", self.width, y);
        unsafe { map.get_unchecked(x).get_unchecked(y) }
    }
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, (x, y): Self::VIndex) -> &'a mut T {
        assert!(x < self.height, "expected 0..{}, but {}", self.height, x);
        assert!(y < self.width, "expected 0..{}, but {}", self.width, y);
        unsafe { map.get_unchecked_mut(x).get_unchecked_mut(y) }
    }
}
impl<A, T> VertexView<Vec<Vec<T>>, T> for GridGraph<A>
where
    T: Clone,
{
    fn vview(&self, map: &Vec<Vec<T>>, vid: Self::VIndex) -> T {
        self.vmap_get(map, vid).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::GridGraph;
    use crate::{graph::ShortestPathExt, num::Saturating, tools::Xorshift};

    #[test]
    fn grid_graph_apsp() {
        let mut rng = Xorshift::default();
        const A: u64 = 1_000_000_000;
        let h = rng.rand(15) as usize + 1;
        let w = rng.rand(15) as usize + 1;

        let weight: Vec<_> = std::iter::repeat_with(|| Saturating(rng.rand(A - 1) + 1))
            .take(8)
            .collect();

        let g = GridGraph::new_adj4(h, w);
        let cost: Vec<Vec<Vec<Vec<_>>>> = (0..h)
            .map(|i| {
                (0..w)
                    .map(|j| {
                        g.standard_sp_additive()
                            .dijkstra([(i, j)], &|dir| weight[dir as usize])
                    })
                    .collect()
            })
            .collect();
        let cost2: Vec<Vec<_>> = g
            .standard_sp_additive()
            .warshall_floyd_ap(&|dir| weight[dir as usize]);
        assert_eq!(cost, cost2);

        let g = GridGraph::new_adj8(h, w);
        let cost: Vec<Vec<Vec<Vec<_>>>> = (0..h)
            .map(|i| {
                (0..w)
                    .map(|j| {
                        g.standard_sp_additive()
                            .dijkstra([(i, j)], &|dir| weight[dir as usize])
                    })
                    .collect()
            })
            .collect();
        let cost2: Vec<Vec<_>> = g
            .standard_sp_additive()
            .warshall_floyd_ap(&|dir| weight[dir as usize]);
        assert_eq!(cost, cost2);
    }
}
