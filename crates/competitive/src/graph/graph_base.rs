/// A neighboring vertex and the label of the arc used to reach it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Neighbor<V, L> {
    pub to: V,
    pub label: L,
}

impl<V, L> Neighbor<V, L> {
    #[inline]
    pub fn new(to: V, label: L) -> Self {
        Self { to, label }
    }
}

impl<V, L> From<(V, L)> for Neighbor<V, L> {
    #[inline]
    fn from((to, label): (V, L)) -> Self {
        Self::new(to, label)
    }
}

/// A finite graph whose outgoing arcs can be iterated without allocation.
pub trait Graph {
    type Vertex: Copy + Eq;
    type Label;
    type Vertices<'g>: Iterator<Item = Self::Vertex>
    where
        Self: 'g;
    type Neighbors<'g>: Iterator<Item = Neighbor<Self::Vertex, Self::Label>>
    where
        Self: 'g;

    fn vsize(&self) -> usize;
    fn vertices(&self) -> Self::Vertices<'_>;
    fn neighbors(&self, vertex: Self::Vertex) -> Self::Neighbors<'_>;
}

/// A graph whose adjacency relation represents directed arcs.
pub trait DirectedGraph: Graph {}

pub trait VertexMap<T>: Graph {
    type Vmap;

    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T;

    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vertex: Self::Vertex) -> &'a T;
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vertex: Self::Vertex) -> &'a mut T;

    #[inline]
    fn vmap_set(&self, map: &mut Self::Vmap, vertex: Self::Vertex, value: T) {
        *self.vmap_get_mut(map, vertex) = value;
    }
}

pub trait EdgeMap<T>: Graph {
    type Emap;

    fn construct_emap<F>(&self, f: F) -> Self::Emap
    where
        F: FnMut() -> T;

    fn emap_get<'a>(&self, map: &'a Self::Emap, eid: Self::Label) -> &'a T;
    fn emap_get_mut<'a>(&self, map: &'a mut Self::Emap, eid: Self::Label) -> &'a mut T;

    #[inline]
    fn emap_set(&self, map: &mut Self::Emap, eid: Self::Label, value: T) {
        *self.emap_get_mut(map, eid) = value;
    }
}
