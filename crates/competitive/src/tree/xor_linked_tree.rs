use super::{IterScan, MarkedIterScan};
use std::{marker::PhantomData, ops::Range};

type Marker<T> = PhantomData<fn() -> T>;
type BuilderMarker<P, D, H, PE, EC, X, B, E> = Marker<((P, D, H), (PE, EC), (X, B, E))>;
type ScannerMarker<U, T, P, D, H, PE, EC, X, B, E> =
    Marker<((U, T), (P, D, H), (PE, EC), (X, B, E))>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoParent {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RecordParent {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoDfsPreorder {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RecordDfsPreorder {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoDepth {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RecordDepth {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoParentEdge {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RecordParentEdge {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoEdgeChild {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RecordEdgeChild {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoEIndexed {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EIndexed {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoXorBottomUpOrder {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RecordXorBottomUpOrder {}

pub trait ParentComponent {
    type Data;
    fn build(parent: Vec<usize>) -> Self::Data;
}
impl ParentComponent for NoParent {
    type Data = ();
    fn build(_parent: Vec<usize>) {}
}
impl ParentComponent for RecordParent {
    type Data = Vec<usize>;
    fn build(parent: Vec<usize>) -> Vec<usize> {
        parent
    }
}

pub trait XorBottomUpOrderBuffer {
    type Data;
    fn new(n: usize) -> Self::Data;
    fn push(data: &mut Self::Data, v: usize);
    fn as_slice(data: &Self::Data) -> &[usize];
}
impl XorBottomUpOrderBuffer for NoXorBottomUpOrder {
    type Data = ();
    fn new(_n: usize) {}
    fn push(_data: &mut Self::Data, _v: usize) {}
    fn as_slice(_data: &Self::Data) -> &[usize] {
        &[]
    }
}
impl XorBottomUpOrderBuffer for RecordXorBottomUpOrder {
    type Data = Vec<usize>;
    fn new(n: usize) -> Vec<usize> {
        Vec::with_capacity(n.saturating_sub(1))
    }
    fn push(data: &mut Vec<usize>, v: usize) {
        data.push(v);
    }
    fn as_slice(data: &Vec<usize>) -> &[usize] {
        data
    }
}

pub trait XorBottomUpOrderComponent {
    type Data;
}
impl XorBottomUpOrderComponent for NoXorBottomUpOrder {
    type Data = ();
}
impl XorBottomUpOrderComponent for RecordXorBottomUpOrder {
    type Data = Vec<usize>;
}

pub trait BuildXorBottomUpOrder<B>: XorBottomUpOrderComponent
where
    B: XorBottomUpOrderBuffer,
{
    fn build(buffer: B::Data) -> Self::Data;
}
impl<B> BuildXorBottomUpOrder<B> for NoXorBottomUpOrder
where
    B: XorBottomUpOrderBuffer,
{
    fn build(_buffer: B::Data) {}
}
impl BuildXorBottomUpOrder<RecordXorBottomUpOrder> for RecordXorBottomUpOrder {
    fn build(buffer: Vec<usize>) -> Vec<usize> {
        buffer
    }
}

#[derive(Debug)]
pub struct DfsPreorder {
    order: Vec<usize>,
    preorder_index: Vec<usize>,
    subtree_end: Vec<usize>,
}

pub trait DfsPreorderComponent {
    type Data;
    fn build(n: usize, root: usize, parent: &[usize], xor_order: &[usize]) -> Self::Data;
}
impl DfsPreorderComponent for NoDfsPreorder {
    type Data = ();
    fn build(_n: usize, _root: usize, _parent: &[usize], _xor_order: &[usize]) {}
}
impl DfsPreorderComponent for RecordDfsPreorder {
    type Data = DfsPreorder;
    fn build(n: usize, root: usize, parent: &[usize], xor_order: &[usize]) -> DfsPreorder {
        let mut preorder_index = vec![1usize; n];
        for &v in xor_order {
            preorder_index[parent[v]] += preorder_index[v];
        }
        let mut subtree_end = vec![0usize; n];
        if n != 0 {
            subtree_end[root] = n;
        }
        for &v in xor_order.iter().rev() {
            let p = parent[v];
            let size = preorder_index[v];
            let r = preorder_index[p];
            subtree_end[v] = r;
            preorder_index[v] = r;
            preorder_index[p] = r - size;
        }
        for i in &mut preorder_index {
            *i -= 1;
        }
        let mut order = vec![0usize; n];
        for v in 0..n {
            order[preorder_index[v]] = v;
        }
        DfsPreorder {
            order,
            preorder_index,
            subtree_end,
        }
    }
}

pub trait DepthComponent {
    type Data;
    fn build(n: usize, root: usize, parent: &[usize], xor_order: &[usize]) -> Self::Data;
}
impl DepthComponent for NoDepth {
    type Data = ();
    fn build(_n: usize, _root: usize, _parent: &[usize], _xor_order: &[usize]) {}
}
impl DepthComponent for RecordDepth {
    type Data = Vec<usize>;
    fn build(n: usize, _root: usize, parent: &[usize], xor_order: &[usize]) -> Vec<usize> {
        let mut depth = vec![0usize; n];
        for &v in xor_order.iter().rev() {
            depth[v] = depth[parent[v]] + 1;
        }
        depth
    }
}

pub trait ParentEdgeComponent {
    type Data;
    fn build(parent_edge: Vec<usize>) -> Self::Data;
}
impl ParentEdgeComponent for NoParentEdge {
    type Data = ();
    fn build(_parent_edge: Vec<usize>) {}
}
impl ParentEdgeComponent for RecordParentEdge {
    type Data = Vec<usize>;
    fn build(parent_edge: Vec<usize>) -> Vec<usize> {
        parent_edge
    }
}

pub trait EdgeChildComponent {
    type Data;
    fn new(m: usize) -> Self::Data;
    fn set(data: &mut Self::Data, eid: usize, child: usize);
}
impl EdgeChildComponent for NoEdgeChild {
    type Data = ();
    fn new(_m: usize) {}
    fn set(_data: &mut Self::Data, _eid: usize, _child: usize) {}
}
impl EdgeChildComponent for RecordEdgeChild {
    type Data = Vec<usize>;
    fn new(m: usize) -> Vec<usize> {
        vec![0usize; m]
    }
    fn set(data: &mut Vec<usize>, eid: usize, child: usize) {
        data[eid] = child;
    }
}

pub struct XorLinkedRootedTree<
    P = NoParent,
    D = NoDfsPreorder,
    H = NoDepth,
    PE = NoParentEdge,
    EC = NoEdgeChild,
    X = NoXorBottomUpOrder,
> where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    X: XorBottomUpOrderComponent,
{
    n: usize,
    root: usize,
    parent: P::Data,
    dfs: D::Data,
    depth: H::Data,
    parent_edge: PE::Data,
    edge_child: EC::Data,
    xor_order: X::Data,
    _marker: Marker<(P, D, H, PE, EC, X)>,
}

pub struct XorLinkedRootedTreeBuilder<
    P = NoParent,
    D = NoDfsPreorder,
    H = NoDepth,
    PE = NoParentEdge,
    EC = NoEdgeChild,
    X = NoXorBottomUpOrder,
    B = NoXorBottomUpOrder,
    E = NoEIndexed,
> {
    n: usize,
    _marker: BuilderMarker<P, D, H, PE, EC, X, B, E>,
}

impl XorLinkedRootedTree {
    pub fn builder(n: usize) -> XorLinkedRootedTreeBuilder {
        XorLinkedRootedTreeBuilder {
            n,
            _marker: PhantomData,
        }
    }
}

impl<P, D, H, PE, EC, X, B, E> XorLinkedRootedTreeBuilder<P, D, H, PE, EC, X, B, E> {
    pub fn with_parent(self) -> XorLinkedRootedTreeBuilder<RecordParent, D, H, PE, EC, X, B, E> {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
    pub fn with_dfs_preorder(
        self,
    ) -> XorLinkedRootedTreeBuilder<P, RecordDfsPreorder, H, PE, EC, X, RecordXorBottomUpOrder, E>
    {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
    pub fn with_depth(
        self,
    ) -> XorLinkedRootedTreeBuilder<P, D, RecordDepth, PE, EC, X, RecordXorBottomUpOrder, E> {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
    pub fn with_xor_bottom_up_order(
        self,
    ) -> XorLinkedRootedTreeBuilder<
        P,
        D,
        H,
        PE,
        EC,
        RecordXorBottomUpOrder,
        RecordXorBottomUpOrder,
        E,
    > {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
}

impl<P, D, H, X, B>
    XorLinkedRootedTreeBuilder<P, D, H, NoParentEdge, NoEdgeChild, X, B, NoEIndexed>
{
    pub fn with_eindexed(
        self,
    ) -> XorLinkedRootedTreeBuilder<P, D, H, NoParentEdge, NoEdgeChild, X, B, EIndexed> {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
}

impl<P, D, H, PE, EC, X, B> XorLinkedRootedTreeBuilder<P, D, H, PE, EC, X, B, EIndexed> {
    pub fn with_parent_edge(
        self,
    ) -> XorLinkedRootedTreeBuilder<P, D, H, RecordParentEdge, EC, X, B, EIndexed> {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
    pub fn with_edge_child(
        self,
    ) -> XorLinkedRootedTreeBuilder<P, D, H, PE, RecordEdgeChild, X, B, EIndexed> {
        XorLinkedRootedTreeBuilder {
            n: self.n,
            _marker: PhantomData,
        }
    }
}

impl<P, D, H, X, B> XorLinkedRootedTreeBuilder<P, D, H, NoParentEdge, NoEdgeChild, X, B, NoEIndexed>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    X: BuildXorBottomUpOrder<B>,
    B: XorBottomUpOrderBuffer,
{
    pub fn build<I>(
        self,
        root: usize,
        edges: I,
    ) -> XorLinkedRootedTree<P, D, H, NoParentEdge, NoEdgeChild, X>
    where
        I: IntoIterator<Item = (usize, usize)>,
    {
        let mut acc = XorAccumulator::new(self.n);
        for (u, v) in edges {
            acc.add_edge(u, v);
        }
        finish_accumulator::<P, D, H, X, B>(self.n, root, acc)
    }
}

impl<P, D, H, PE, EC, X, B> XorLinkedRootedTreeBuilder<P, D, H, PE, EC, X, B, EIndexed>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    X: BuildXorBottomUpOrder<B>,
    B: XorBottomUpOrderBuffer,
{
    pub fn build<I>(self, root: usize, edges: I) -> XorLinkedRootedTree<P, D, H, PE, EC, X>
    where
        I: IntoIterator<Item = (usize, usize)>,
    {
        let mut acc = XorEIndexedAccumulator::new(self.n);
        for (eid, (u, v)) in edges.into_iter().enumerate() {
            acc.add_edge(eid, u, v);
        }
        finish_eindexed_accumulator::<P, D, H, PE, EC, X, B>(self.n, root, acc)
    }
}

impl XorLinkedRootedTreeBuilder {
    pub fn run<I, F>(self, root: usize, edges: I, f: F)
    where
        I: IntoIterator<Item = (usize, usize)>,
        F: FnMut(usize, usize),
    {
        let mut acc = XorAccumulator::new(self.n);
        for (u, v) in edges {
            acc.add_edge(u, v);
        }
        let mut order = ();
        acc.finish::<NoXorBottomUpOrder, _>(root, &mut order, f);
    }
}

impl
    XorLinkedRootedTreeBuilder<
        NoParent,
        NoDfsPreorder,
        NoDepth,
        NoParentEdge,
        NoEdgeChild,
        NoXorBottomUpOrder,
        NoXorBottomUpOrder,
        EIndexed,
    >
{
    pub fn run<I, F>(self, root: usize, edges: I, f: F)
    where
        I: IntoIterator<Item = (usize, usize)>,
        F: FnMut(usize, usize, usize),
    {
        let mut acc = XorEIndexedAccumulator::new(self.n);
        for (eid, (u, v)) in edges.into_iter().enumerate() {
            acc.add_edge(eid, u, v);
        }
        let mut order = ();
        let mut edge_child = ();
        acc.finish::<NoXorBottomUpOrder, NoEdgeChild, _>(root, &mut order, &mut edge_child, f);
    }
}

impl<P, D, H, PE, EC, O> XorLinkedRootedTree<P, D, H, PE, EC, O>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    O: XorBottomUpOrderComponent,
{
    pub fn vertices_size(&self) -> usize {
        self.n
    }
    pub fn edges_size(&self) -> usize {
        self.n.saturating_sub(1)
    }
    pub fn root(&self) -> usize {
        self.root
    }
}

impl<D, H, PE, EC, O> XorLinkedRootedTree<RecordParent, D, H, PE, EC, O>
where
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    O: XorBottomUpOrderComponent,
{
    pub fn parent(&self, v: usize) -> usize {
        self.parent[v]
    }
    pub fn parents(&self) -> &[usize] {
        &self.parent
    }
}

impl<P, D, H, PE, EC> XorLinkedRootedTree<P, D, H, PE, EC, RecordXorBottomUpOrder>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
{
    /// Returns the bottom-up XOR order, excluding the root.
    pub fn xor_bottom_up_order(&self) -> &[usize] {
        &self.xor_order
    }
    /// Returns the top-down XOR order, excluding the root.
    pub fn xor_top_down_order(
        &self,
    ) -> impl DoubleEndedIterator<Item = usize> + ExactSizeIterator + '_ {
        self.xor_order.iter().rev().copied()
    }
}

impl<P, H, PE, EC, O> XorLinkedRootedTree<P, RecordDfsPreorder, H, PE, EC, O>
where
    P: ParentComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    O: XorBottomUpOrderComponent,
{
    pub fn dfs_order(&self) -> &[usize] {
        &self.dfs.order
    }
    pub fn dfs_index(&self, v: usize) -> usize {
        self.dfs.preorder_index[v]
    }
    pub fn subtree_size(&self, v: usize) -> usize {
        self.dfs.subtree_end[v] - self.dfs.preorder_index[v]
    }
    pub fn subtree_range(&self, v: usize) -> Range<usize> {
        self.dfs.preorder_index[v]..self.dfs.subtree_end[v]
    }
    pub fn children(&self, v: usize) -> Children<'_> {
        Children {
            dfs: &self.dfs,
            next: self.dfs.preorder_index[v] + 1,
            end: self.dfs.subtree_end[v],
        }
    }
}

impl<P, D, PE, EC, O> XorLinkedRootedTree<P, D, RecordDepth, PE, EC, O>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    O: XorBottomUpOrderComponent,
{
    pub fn depth(&self, v: usize) -> usize {
        self.depth[v]
    }
    pub fn depths(&self) -> &[usize] {
        &self.depth
    }
}

impl<P, D, H, EC, O> XorLinkedRootedTree<P, D, H, RecordParentEdge, EC, O>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    EC: EdgeChildComponent,
    O: XorBottomUpOrderComponent,
{
    pub fn parent_edge(&self, v: usize) -> usize {
        self.parent_edge[v]
    }
    pub fn parent_edges(&self) -> &[usize] {
        &self.parent_edge
    }
}

impl<P, D, H, PE, O> XorLinkedRootedTree<P, D, H, PE, RecordEdgeChild, O>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    O: XorBottomUpOrderComponent,
{
    pub fn edge_child(&self, eid: usize) -> usize {
        self.edge_child[eid]
    }
    pub fn edge_children(&self) -> &[usize] {
        &self.edge_child
    }
}

pub struct Children<'a> {
    dfs: &'a DfsPreorder,
    next: usize,
    end: usize,
}

impl Iterator for Children<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.next == self.end {
            None
        } else {
            let v = self.dfs.order[self.next];
            self.next = self.dfs.subtree_end[v];
            Some(v)
        }
    }
}

pub struct XorLinkedRootedTreeScanner<
    U,
    T = (),
    P = NoParent,
    D = NoDfsPreorder,
    H = NoDepth,
    PE = NoParentEdge,
    EC = NoEdgeChild,
    X = NoXorBottomUpOrder,
    B = NoXorBottomUpOrder,
    E = NoEIndexed,
> where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    n: usize,
    root: usize,
    _marker: ScannerMarker<U, T, P, D, H, PE, EC, X, B, E>,
}

impl<U, T> XorLinkedRootedTreeScanner<U, T>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    pub fn new(n: usize, root: usize) -> Self {
        Self {
            n,
            root,
            _marker: PhantomData,
        }
    }
}

impl<U, T, P, D, H, PE, EC, X, B, E> XorLinkedRootedTreeScanner<U, T, P, D, H, PE, EC, X, B, E>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    pub fn with_parent(
        self,
    ) -> XorLinkedRootedTreeScanner<U, T, RecordParent, D, H, PE, EC, X, B, E> {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
    pub fn with_dfs_preorder(
        self,
    ) -> XorLinkedRootedTreeScanner<
        U,
        T,
        P,
        RecordDfsPreorder,
        H,
        PE,
        EC,
        X,
        RecordXorBottomUpOrder,
        E,
    > {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
    pub fn with_depth(
        self,
    ) -> XorLinkedRootedTreeScanner<U, T, P, D, RecordDepth, PE, EC, X, RecordXorBottomUpOrder, E>
    {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
    pub fn with_xor_bottom_up_order(
        self,
    ) -> XorLinkedRootedTreeScanner<
        U,
        T,
        P,
        D,
        H,
        PE,
        EC,
        RecordXorBottomUpOrder,
        RecordXorBottomUpOrder,
        E,
    > {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
}

impl<U, T, P, D, H, X, B>
    XorLinkedRootedTreeScanner<U, T, P, D, H, NoParentEdge, NoEdgeChild, X, B, NoEIndexed>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    pub fn with_eindexed(
        self,
    ) -> XorLinkedRootedTreeScanner<U, T, P, D, H, NoParentEdge, NoEdgeChild, X, B, EIndexed> {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
}

impl<U, T, P, D, H, PE, EC, X, B> XorLinkedRootedTreeScanner<U, T, P, D, H, PE, EC, X, B, EIndexed>
where
    U: IterScan<Output = usize>,
    T: IterScan,
{
    pub fn with_parent_edge(
        self,
    ) -> XorLinkedRootedTreeScanner<U, T, P, D, H, RecordParentEdge, EC, X, B, EIndexed> {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
    pub fn with_edge_child(
        self,
    ) -> XorLinkedRootedTreeScanner<U, T, P, D, H, PE, RecordEdgeChild, X, B, EIndexed> {
        XorLinkedRootedTreeScanner {
            n: self.n,
            root: self.root,
            _marker: PhantomData,
        }
    }
}

impl<U, T, P, D, H, X, B> MarkedIterScan
    for XorLinkedRootedTreeScanner<U, T, P, D, H, NoParentEdge, NoEdgeChild, X, B, NoEIndexed>
where
    U: IterScan<Output = usize>,
    T: IterScan,
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    X: BuildXorBottomUpOrder<B>,
    B: XorBottomUpOrderBuffer,
{
    type Output = (
        XorLinkedRootedTree<P, D, H, NoParentEdge, NoEdgeChild, X>,
        Vec<<T as IterScan>::Output>,
    );

    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut acc = XorAccumulator::new(self.n);
        let mut weights = Vec::with_capacity(self.n.saturating_sub(1));
        for _ in 0..self.n.saturating_sub(1) {
            let u = U::scan(iter)?;
            let v = U::scan(iter)?;
            acc.add_edge(u, v);
            weights.push(T::scan(iter)?);
        }
        Some((
            finish_accumulator::<P, D, H, X, B>(self.n, self.root, acc),
            weights,
        ))
    }
}

impl<U, T, P, D, H, PE, EC, X, B> MarkedIterScan
    for XorLinkedRootedTreeScanner<U, T, P, D, H, PE, EC, X, B, EIndexed>
where
    U: IterScan<Output = usize>,
    T: IterScan,
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    X: BuildXorBottomUpOrder<B>,
    B: XorBottomUpOrderBuffer,
{
    type Output = (
        XorLinkedRootedTree<P, D, H, PE, EC, X>,
        Vec<<T as IterScan>::Output>,
    );

    fn mscan<'a, I: Iterator<Item = &'a str>>(self, iter: &mut I) -> Option<Self::Output> {
        let mut acc = XorEIndexedAccumulator::new(self.n);
        let mut weights = Vec::with_capacity(self.n.saturating_sub(1));
        for eid in 0..self.n.saturating_sub(1) {
            let u = U::scan(iter)?;
            let v = U::scan(iter)?;
            acc.add_edge(eid, u, v);
            weights.push(T::scan(iter)?);
        }
        Some((
            finish_eindexed_accumulator::<P, D, H, PE, EC, X, B>(self.n, self.root, acc),
            weights,
        ))
    }
}

fn finish_accumulator<P, D, H, X, B>(
    n: usize,
    root: usize,
    acc: XorAccumulator,
) -> XorLinkedRootedTree<P, D, H, NoParentEdge, NoEdgeChild, X>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    X: BuildXorBottomUpOrder<B>,
    B: XorBottomUpOrderBuffer,
{
    let mut xor_order = B::new(n);
    let parent = acc.finish::<B, _>(root, &mut xor_order, |_, _| {});
    let order = B::as_slice(&xor_order);
    let dfs = D::build(n, root, &parent, order);
    let depth = H::build(n, root, &parent, order);
    XorLinkedRootedTree {
        n,
        root,
        parent: P::build(parent),
        dfs,
        depth,
        parent_edge: (),
        edge_child: (),
        xor_order: X::build(xor_order),
        _marker: PhantomData,
    }
}

fn finish_eindexed_accumulator<P, D, H, PE, EC, X, B>(
    n: usize,
    root: usize,
    acc: XorEIndexedAccumulator,
) -> XorLinkedRootedTree<P, D, H, PE, EC, X>
where
    P: ParentComponent,
    D: DfsPreorderComponent,
    H: DepthComponent,
    PE: ParentEdgeComponent,
    EC: EdgeChildComponent,
    X: BuildXorBottomUpOrder<B>,
    B: XorBottomUpOrderBuffer,
{
    let mut xor_order = B::new(n);
    let mut edge_child = EC::new(n.saturating_sub(1));
    let (parent, parent_edge) =
        acc.finish::<B, EC, _>(root, &mut xor_order, &mut edge_child, |_, _, _| {});
    let order = B::as_slice(&xor_order);
    let dfs = D::build(n, root, &parent, order);
    let depth = H::build(n, root, &parent, order);
    XorLinkedRootedTree {
        n,
        root,
        parent: P::build(parent),
        dfs,
        depth,
        parent_edge: PE::build(parent_edge),
        edge_child,
        xor_order: X::build(xor_order),
        _marker: PhantomData,
    }
}

struct XorAccumulator {
    deg: Vec<isize>,
    xor: Vec<usize>,
}

impl XorAccumulator {
    fn new(n: usize) -> Self {
        Self {
            deg: vec![0; n],
            xor: vec![0; n],
        }
    }
    fn add_edge(&mut self, u: usize, v: usize) {
        self.deg[u] += 1;
        self.deg[v] += 1;
        self.xor[u] ^= v;
        self.xor[v] ^= u;
    }
    fn finish<O, F>(mut self, root: usize, xor_order: &mut O::Data, mut f: F) -> Vec<usize>
    where
        O: XorBottomUpOrderBuffer,
        F: FnMut(usize, usize),
    {
        self.deg[root] = 0;
        for i in 0..self.deg.len() {
            let mut v = i;
            while self.deg[v] == 1 {
                let p = self.xor[v];
                O::push(xor_order, v);
                f(v, p);
                self.deg[v] = 0;
                self.deg[p] -= 1;
                self.xor[p] ^= v;
                v = p;
            }
        }
        self.xor[root] = usize::MAX;
        self.xor
    }
}

struct XorEIndexedAccumulator {
    deg: Vec<isize>,
    xor: Vec<usize>,
    edge_xor: Vec<usize>,
}

impl XorEIndexedAccumulator {
    fn new(n: usize) -> Self {
        Self {
            deg: vec![0; n],
            xor: vec![0; n],
            edge_xor: vec![0; n],
        }
    }
    fn add_edge(&mut self, eid: usize, u: usize, v: usize) {
        self.deg[u] += 1;
        self.deg[v] += 1;
        self.xor[u] ^= v;
        self.xor[v] ^= u;
        self.edge_xor[u] ^= eid;
        self.edge_xor[v] ^= eid;
    }
    fn finish<O, EC, F>(
        mut self,
        root: usize,
        xor_order: &mut O::Data,
        edge_child: &mut EC::Data,
        mut f: F,
    ) -> (Vec<usize>, Vec<usize>)
    where
        O: XorBottomUpOrderBuffer,
        EC: EdgeChildComponent,
        F: FnMut(usize, usize, usize),
    {
        self.deg[root] = 0;
        for i in 0..self.deg.len() {
            let mut v = i;
            while self.deg[v] == 1 {
                let p = self.xor[v];
                let e = self.edge_xor[v];
                O::push(xor_order, v);
                EC::set(edge_child, e, v);
                f(v, p, e);
                self.deg[v] = 0;
                self.deg[p] -= 1;
                self.xor[p] ^= v;
                self.edge_xor[p] ^= e;
                v = p;
            }
        }
        self.xor[root] = usize::MAX;
        self.edge_xor[root] = usize::MAX;
        (self.xor, self.edge_xor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::UndirectedSparseGraph,
        scan,
        tools::{Scanner, Xorshift},
        tree::MixedTree,
    };

    fn expected_parent_depth(
        graph: &UndirectedSparseGraph,
        root: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let n = graph.vertices_size();
        let mut parent = vec![usize::MAX; n];
        let mut depth = vec![0usize; n];
        let mut stack = vec![root];
        while let Some(u) = stack.pop() {
            for a in graph.adjacencies(u) {
                if a.to != parent[u] {
                    parent[a.to] = u;
                    depth[a.to] = depth[u] + 1;
                    stack.push(a.to);
                }
            }
        }
        (parent, depth)
    }

    fn assert_rooted_tree<O>(
        graph: &UndirectedSparseGraph,
        root: usize,
        tree: &XorLinkedRootedTree<
            RecordParent,
            RecordDfsPreorder,
            RecordDepth,
            NoParentEdge,
            NoEdgeChild,
            O,
        >,
    ) where
        O: XorBottomUpOrderComponent,
    {
        let n = graph.vertices_size();
        let (parent, depth) = expected_parent_depth(graph, root);
        assert_eq!(tree.vertices_size(), n);
        assert_eq!(tree.edges_size(), n.saturating_sub(1));
        assert_eq!(tree.root(), root);
        assert_eq!(tree.parents(), parent);
        assert_eq!(tree.depths(), depth);

        let mut seen = vec![false; n];
        for (i, &v) in tree.dfs_order().iter().enumerate() {
            assert!(!seen[v]);
            seen[v] = true;
            assert_eq!(tree.dfs_index(v), i);
        }
        assert!(seen.into_iter().all(|x| x));

        let mut children = vec![vec![]; n];
        for v in 0..n {
            if v != root {
                children[parent[v]].push(v);
            }
        }
        for (v, expected_children) in children.iter_mut().enumerate() {
            let mut actual: Vec<_> = tree.children(v).collect();
            actual.sort_unstable();
            expected_children.sort_unstable();
            assert_eq!(actual, *expected_children);
            assert_eq!(
                tree.subtree_size(v),
                expected_children
                    .iter()
                    .map(|&u| tree.subtree_size(u))
                    .sum::<usize>()
                    + 1
            );
            let range = tree.subtree_range(v);
            for &u in &tree.dfs_order()[range.clone()] {
                let mut x = u;
                while x != v && x != usize::MAX {
                    x = parent[x];
                }
                assert_eq!(x, v);
            }
            assert_eq!(range.len(), tree.subtree_size(v));
        }
    }

    #[test]
    fn xor_linked_tree_rooted_properties() {
        let mut rng = Xorshift::default();
        for n in 1..=200 {
            for _ in 0..3 {
                let graph = rng.random(MixedTree(n));
                let root = rng.random(0..n);
                let tree = XorLinkedRootedTree::builder(n)
                    .with_parent()
                    .with_dfs_preorder()
                    .with_depth()
                    .build(root, graph.edges.iter().copied());
                assert_rooted_tree(&graph, root, &tree);
            }
        }
    }

    #[test]
    fn xor_linked_tree_eindexed_properties() {
        let mut rng = Xorshift::default();
        for n in 1..=200 {
            for _ in 0..3 {
                let graph = rng.random(MixedTree(n));
                let root = rng.random(0..n);
                let tree = XorLinkedRootedTree::builder(n)
                    .with_parent()
                    .with_eindexed()
                    .with_parent_edge()
                    .with_edge_child()
                    .build(root, graph.edges.iter().copied());
                assert_eq!(tree.parent(root), usize::MAX);
                assert_eq!(tree.parent_edge(root), usize::MAX);
                for (eid, &(u, v)) in graph.edges.iter().enumerate() {
                    let child = if tree.parent(u) == v {
                        u
                    } else {
                        assert_eq!(tree.parent(v), u);
                        v
                    };
                    assert_eq!(tree.parent_edge(child), eid);
                    assert_eq!(tree.edge_child(eid), child);
                }
            }
        }
    }

    #[test]
    fn xor_linked_tree_xor_bottom_up_order() {
        let mut rng = Xorshift::default();
        for n in 1..=200 {
            for _ in 0..3 {
                let graph = rng.random(MixedTree(n));
                let root = rng.random(0..n);
                let tree = XorLinkedRootedTree::builder(n)
                    .with_parent()
                    .with_xor_bottom_up_order()
                    .build(root, graph.edges.iter().copied());
                let bottom_up = tree.xor_bottom_up_order();
                assert_eq!(bottom_up.len(), n - 1);
                assert!(!bottom_up.contains(&root));

                let mut bottom_up_index = vec![usize::MAX; n];
                for (i, &v) in bottom_up.iter().enumerate() {
                    assert_eq!(bottom_up_index[v], usize::MAX);
                    bottom_up_index[v] = i;
                }
                for v in 0..n {
                    if v != root && tree.parent(v) != root {
                        assert!(bottom_up_index[v] < bottom_up_index[tree.parent(v)]);
                    }
                }

                let top_down: Vec<_> = tree.xor_top_down_order().collect();
                assert_eq!(
                    top_down,
                    bottom_up.iter().rev().copied().collect::<Vec<_>>()
                );
                let mut depth = vec![0usize; n];
                for v in top_down {
                    depth[v] = depth[tree.parent(v)] + 1;
                }
                let (_, expected_depth) = expected_parent_depth(&graph, root);
                assert_eq!(depth, expected_depth);
            }
        }
    }

    #[test]
    fn xor_linked_tree_visitor() {
        let graph = UndirectedSparseGraph::from_edges(5, vec![(0, 1), (1, 2), (1, 3), (3, 4)]);
        let mut seen = vec![];
        XorLinkedRootedTree::builder(graph.vertices_size()).run(
            0,
            graph.edges.iter().copied(),
            |u, p| {
                seen.push((u, p));
            },
        );
        assert_eq!(seen.len(), graph.edges_size());
        let mut seen_e = vec![];
        XorLinkedRootedTree::builder(graph.vertices_size())
            .with_eindexed()
            .run(0, graph.edges.iter().copied(), |u, p, e| {
                seen_e.push((u, p, e));
            });
        assert_eq!(seen_e.len(), graph.edges_size());
        for (u, p, e) in seen_e {
            assert!(graph.edges[e] == (u, p) || graph.edges[e] == (p, u));
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct NonCloneWeight(usize);

    impl IterScan for NonCloneWeight {
        type Output = NonCloneWeight;
        fn scan<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<NonCloneWeight> {
            Some(NonCloneWeight(usize::scan(iter)?))
        }
    }

    #[test]
    fn xor_linked_tree_scanner() {
        let mut scanner = Scanner::new("0 1 10 1 2 20 1 3 30");
        scan!(
            scanner,
            (tree, weights): @XorLinkedRootedTreeScanner::<usize, NonCloneWeight>::new(4, 0)
                .with_parent()
                .with_eindexed()
                .with_parent_edge()
        );
        assert_eq!(tree.parent(0), usize::MAX);
        assert_eq!(tree.parent(1), 0);
        assert_eq!(tree.parent(2), 1);
        assert_eq!(tree.parent(3), 1);
        assert_eq!(
            weights,
            vec![NonCloneWeight(10), NonCloneWeight(20), NonCloneWeight(30)]
        );
        assert_eq!(tree.parent_edge(1), 0);
        assert_eq!(tree.parent_edge(2), 1);
        assert_eq!(tree.parent_edge(3), 2);
    }
}
