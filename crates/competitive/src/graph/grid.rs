use super::{Graph, Neighbor, VertexMap};
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

#[derive(Debug, Clone)]
pub struct GridVertices {
    xrange: Range<usize>,
    yrange: Range<usize>,
}

impl Iterator for GridVertices {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.xrange.start >= self.xrange.end {
                return None;
            }
            if let Some(ny) = self.yrange.next() {
                return Some((self.xrange.start, ny));
            }
            self.yrange.start = 0;
            self.xrange.start += 1;
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
        let h = rng.rand(15) as usize + 1;
        let w = rng.rand(15) as usize + 1;

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
