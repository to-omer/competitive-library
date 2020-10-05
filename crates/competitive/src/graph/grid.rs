#[snippet::entry("GridGraph")]
pub use grid_graph::GridGraph;
#[snippet::entry("GridGraph")]
pub mod grid_graph {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct GridGraph {
        height: usize,
        width: usize,
    }
    impl GridGraph {
        pub fn new(height: usize, width: usize) -> Self {
            Self { height, width }
        }
        pub fn adjacency4(&self, x: usize, y: usize) -> Adjacency4<'_> {
            Adjacency4 {
                grid: self,
                x,
                y,
                state: 0,
            }
        }
        pub fn adjacency8(&self, x: usize, y: usize) -> Adjacency8<'_> {
            Adjacency8 {
                grid: self,
                x,
                y,
                state: 0,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Adjacency4<'a> {
        grid: &'a GridGraph,
        x: usize,
        y: usize,
        state: usize,
    }
    impl<'a> Iterator for Adjacency4<'a> {
        type Item = (usize, usize);
        fn next(&mut self) -> Option<Self::Item> {
            const D: [(usize, usize); 4] = [(1, 0), (0, 1), (!0, 0), (0, !0)];
            for &(dx, dy) in D[self.state..].iter() {
                self.state += 1;
                let nx = self.x.wrapping_add(dx);
                let ny = self.y.wrapping_add(dy);
                if nx < self.grid.height && ny < self.grid.width {
                    return Some((nx, ny));
                }
            }
            None
        }
    }
    #[derive(Debug, Clone)]
    pub struct Adjacency8<'a> {
        grid: &'a GridGraph,
        x: usize,
        y: usize,
        state: usize,
    }
    impl<'a> Iterator for Adjacency8<'a> {
        type Item = (usize, usize);
        fn next(&mut self) -> Option<Self::Item> {
            const D: [(usize, usize); 8] = [
                (1, 0),
                (1, 1),
                (0, 1),
                (!0, 1),
                (!0, 0),
                (!0, !0),
                (0, !0),
                (1, !0),
            ];
            for &(dx, dy) in D[self.state..].iter() {
                self.state += 1;
                let nx = self.x.wrapping_add(dx);
                let ny = self.y.wrapping_add(dy);
                if nx < self.grid.height && ny < self.grid.width {
                    return Some((nx, ny));
                }
            }
            None
        }
    }
}
