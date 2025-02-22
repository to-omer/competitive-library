use std::marker::PhantomData;

pub trait GraphBase {
    type VIndex: Copy + Eq;
}
pub trait EIndexedGraph: GraphBase {
    type EIndex: Copy + Eq;
}

pub trait VertexSize: GraphBase {
    fn vsize(&self) -> usize;
}
pub trait EdgeSize: GraphBase {
    fn esize(&self) -> usize;
}

pub trait Vertices: GraphBase {
    type VIter<'g>: Iterator<Item = Self::VIndex>
    where
        Self: 'g;
    fn vertices(&self) -> Self::VIter<'_>;
}
pub trait Edges: EIndexedGraph {
    type EIter<'g>: Iterator<Item = Self::EIndex>
    where
        Self: 'g;
    fn edges<'g>(&self) -> Self::EIter<'g>;
}

pub trait AdjacencyIndex {
    type VIndex: Copy + Eq;
    fn vindex(&self) -> Self::VIndex;
}
pub trait Adjacencies: GraphBase {
    type AIndex: AdjacencyIndex<VIndex = Self::VIndex>;
    type AIter<'g>: Iterator<Item = Self::AIndex>
    where
        Self: 'g;
    fn adjacencies(&self, vid: Self::VIndex) -> Self::AIter<'_>;
}

pub trait AdjacenciesWithEindex: EIndexedGraph {
    type AIndex: AdjacencyIndexWithEindex<VIndex = Self::VIndex, EIndex = Self::EIndex>;
    type AIter<'g>: Iterator<Item = Self::AIndex>
    where
        Self: 'g;
    fn adjacencies_with_eindex(&self, vid: Self::VIndex) -> Self::AIter<'_>;
}
pub trait AdjacencyIndexWithEindex: AdjacencyIndex {
    type EIndex: Copy + Eq;
    fn eindex(&self) -> Self::EIndex;
}

pub trait AdjacencyIndexWithValue: AdjacencyIndex {
    type AValue: Clone;
    fn avalue(&self) -> Self::AValue;
}
pub trait AdjacenciesWithValue<T>: GraphBase
where
    T: Clone,
{
    type AIndex: AdjacencyIndexWithValue<VIndex = Self::VIndex, AValue = T>;
    type AIter<'g>: Iterator<Item = Self::AIndex>
    where
        Self: 'g;
    fn adjacencies_with_value(&self, vid: Self::VIndex) -> Self::AIter<'_>;
}

impl AdjacencyIndex for usize {
    type VIndex = usize;
    fn vindex(&self) -> Self::VIndex {
        *self
    }
}
impl<V, E> AdjacencyIndex for (V, E)
where
    V: Copy + Eq,
    E: Copy + Eq,
{
    type VIndex = V;
    fn vindex(&self) -> Self::VIndex {
        self.0
    }
}
impl<V, E> AdjacencyIndexWithEindex for (V, E)
where
    V: Copy + Eq,
    E: Copy + Eq,
{
    type EIndex = E;
    fn eindex(&self) -> Self::EIndex {
        self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VIndex<V>(V);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VIndexWithEIndex<V, E>(V, E);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VIndexWithValue<V, T>(V, T);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VIndexWithEIndexValue<V, E, T>(V, E, T);

impl<V> AdjacencyIndex for VIndex<V>
where
    V: Eq + Copy,
{
    type VIndex = V;
    fn vindex(&self) -> Self::VIndex {
        self.0
    }
}
impl<V, E> AdjacencyIndex for VIndexWithEIndex<V, E>
where
    V: Eq + Copy,
{
    type VIndex = V;
    fn vindex(&self) -> Self::VIndex {
        self.0
    }
}
impl<V, E> AdjacencyIndexWithEindex for VIndexWithEIndex<V, E>
where
    V: Eq + Copy,
    E: Eq + Copy,
{
    type EIndex = E;
    fn eindex(&self) -> Self::EIndex {
        self.1
    }
}
impl<V, T> AdjacencyIndex for VIndexWithValue<V, T>
where
    V: Eq + Copy,
{
    type VIndex = V;
    fn vindex(&self) -> Self::VIndex {
        self.0
    }
}
impl<V, T> AdjacencyIndexWithValue for VIndexWithValue<V, T>
where
    V: Eq + Copy,
    T: Clone,
{
    type AValue = T;
    fn avalue(&self) -> Self::AValue {
        self.1.clone()
    }
}
impl<V, E, T> AdjacencyIndex for VIndexWithEIndexValue<V, E, T>
where
    V: Eq + Copy,
{
    type VIndex = V;
    fn vindex(&self) -> Self::VIndex {
        self.0
    }
}
impl<V, E, T> AdjacencyIndexWithEindex for VIndexWithEIndexValue<V, E, T>
where
    V: Eq + Copy,
    E: Eq + Copy,
{
    type EIndex = E;
    fn eindex(&self) -> Self::EIndex {
        self.1
    }
}
impl<V, E, T> AdjacencyIndexWithValue for VIndexWithEIndexValue<V, E, T>
where
    V: Eq + Copy,
    T: Clone,
{
    type AValue = T;
    fn avalue(&self) -> Self::AValue {
        self.2.clone()
    }
}
impl<V> From<V> for VIndex<V> {
    fn from(vid: V) -> Self {
        VIndex(vid)
    }
}
impl<V, E> From<(V, E)> for VIndexWithEIndex<V, E> {
    fn from((vid, eid): (V, E)) -> Self {
        VIndexWithEIndex(vid, eid)
    }
}
impl<V, T> From<(V, T)> for VIndexWithValue<V, T> {
    fn from((vid, value): (V, T)) -> Self {
        VIndexWithValue(vid, value)
    }
}
impl<V, E, T> From<(V, E, T)> for VIndexWithEIndexValue<V, E, T> {
    fn from((vid, eid, value): (V, E, T)) -> Self {
        VIndexWithEIndexValue(vid, eid, value)
    }
}
impl<V, T> VIndexWithValue<V, T> {
    pub fn map<U, F>(self, mut f: F) -> VIndexWithValue<V, U>
    where
        F: FnMut(T) -> U,
    {
        VIndexWithValue(self.0, f(self.1))
    }
}
impl<V, E, T> VIndexWithEIndexValue<V, E, T> {
    pub fn map<U, F>(self, mut f: F) -> VIndexWithEIndexValue<V, E, U>
    where
        F: FnMut(T) -> U,
    {
        VIndexWithEIndexValue(self.0, self.1, f(self.2))
    }
}

pub trait VertexMap<T>: GraphBase {
    type Vmap;
    fn construct_vmap<F>(&self, f: F) -> Self::Vmap
    where
        F: FnMut() -> T;
    fn vmap_get<'a>(&self, map: &'a Self::Vmap, vid: Self::VIndex) -> &'a T;
    fn vmap_get_mut<'a>(&self, map: &'a mut Self::Vmap, vid: Self::VIndex) -> &'a mut T;
    fn vmap_set(&self, map: &mut Self::Vmap, vid: Self::VIndex, x: T) {
        *self.vmap_get_mut(map, vid) = x;
    }
}
pub trait VertexView<M, T>: GraphBase
where
    M: ?Sized,
{
    fn vview(&self, map: &M, vid: Self::VIndex) -> T;
}
pub trait EdgeMap<T>: EIndexedGraph {
    type Emap;
    fn construct_emap<F>(&self, f: F) -> Self::Emap
    where
        F: FnMut() -> T;
    fn emap_get<'a>(&self, map: &'a Self::Emap, eid: Self::EIndex) -> &'a T;
    fn emap_get_mut<'a>(&self, map: &'a mut Self::Emap, eid: Self::EIndex) -> &'a mut T;
    fn emap_set(&self, map: &mut Self::Emap, eid: Self::EIndex, x: T) {
        *self.emap_get_mut(map, eid) = x;
    }
}
pub trait EdgeView<M, T>: EIndexedGraph
where
    M: ?Sized,
{
    fn eview(&self, map: &M, eid: Self::EIndex) -> T;
}

impl<G, F, T> VertexView<F, T> for G
where
    G: GraphBase,
    F: Fn(Self::VIndex) -> T,
{
    fn vview(&self, map: &F, vid: Self::VIndex) -> T {
        (map)(vid)
    }
}
impl<G, F, T> EdgeView<F, T> for G
where
    G: EIndexedGraph,
    F: Fn(Self::EIndex) -> T,
{
    fn eview(&self, map: &F, eid: Self::EIndex) -> T {
        (map)(eid)
    }
}

pub trait AdjacencyView<'a, M, T>: GraphBase
where
    M: ?Sized,
{
    type AViewIter<'g>: Iterator<Item = VIndexWithValue<Self::VIndex, T>>
    where
        M: 'a,
        Self: 'g;
    fn aviews<'g>(&'g self, map: &'a M, vid: Self::VIndex) -> Self::AViewIter<'g>;
}

pub struct AdjacencyViewIterFromEindex<'g, 'a, G, M, T>
where
    G: AdjacenciesWithEindex,
{
    iter: G::AIter<'g>,
    g: &'g G,
    map: &'a M,
    _marker: PhantomData<fn() -> T>,
}
impl<'g, 'a, G, M, T> AdjacencyViewIterFromEindex<'g, 'a, G, M, T>
where
    G: AdjacenciesWithEindex,
{
    pub fn new(iter: G::AIter<'g>, g: &'g G, map: &'a M) -> Self {
        Self {
            iter,
            g,
            map,
            _marker: PhantomData,
        }
    }
}
impl<'a, G, M, T> Iterator for AdjacencyViewIterFromEindex<'_, 'a, G, M, T>
where
    G: AdjacenciesWithEindex + EdgeView<M, T>,
    M: 'a,
{
    type Item = VIndexWithValue<G::VIndex, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|adj| (adj.vindex(), self.g.eview(self.map, adj.eindex())).into())
    }
}

pub struct AdjacencyViewIterFromValue<'g, 'a, G, M, T, U>
where
    G: 'g + AdjacenciesWithValue<T>,
    T: Clone,
{
    iter: G::AIter<'g>,
    map: &'a M,
    _marker: PhantomData<fn() -> U>,
}
impl<'g, 'a, G, M, T, U> AdjacencyViewIterFromValue<'g, 'a, G, M, T, U>
where
    G: AdjacenciesWithValue<T>,
    T: Clone,
{
    pub fn new(iter: G::AIter<'g>, map: &'a M) -> Self {
        Self {
            iter,
            map,
            _marker: PhantomData,
        }
    }
}
impl<'a, G, M, T, U> Iterator for AdjacencyViewIterFromValue<'_, 'a, G, M, T, U>
where
    G: AdjacenciesWithValue<T>,
    T: Clone,
    M: 'a + Fn(T) -> U,
{
    type Item = VIndexWithValue<G::VIndex, U>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|adj| (adj.vindex(), (self.map)(adj.avalue())).into())
    }
}
