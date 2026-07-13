use super::{
    Allocator, LazyMapMonoid, MemoryPool,
    binary_search_tree::{
        BstDataMutRef, BstNode, BstRoot, BstSeeker, BstSpec, EqualSide, data::LazyMapElement,
        node::WithParent,
    },
    splay_operations,
};
use std::{marker::PhantomData, mem::replace, ptr::NonNull};

pub trait LinkCutTreeSpec: Sized {
    type Value;
    type Data;

    /// Whether splay must propagate from the auxiliary root before rotations.
    const ROOT_TO_NODE_TOP_DOWN: bool = true;

    fn new(value: Self::Value) -> Self::Data;
    fn value(data: &Self::Data) -> &Self::Value;
    fn value_mut(data: &mut Self::Data) -> &mut Self::Value;
    fn top_down(_data: &mut Self::Data, _children: [Option<&mut Self::Data>; 2]) {}
    fn bottom_up(data: &mut Self::Data, children: [Option<&Self::Data>; 2]);
    fn reverse(data: &mut Self::Data);
    fn attach_virtual(_parent: &mut Self::Data, _child: &mut Self::Data) {}
    fn detach_virtual(_parent: &mut Self::Data, _child: &mut Self::Data) {}
    /// Moves state associated with the same path-parent edge after a splay.
    fn transfer_path_parent(_old_root: &mut Self::Data, _new_root: &mut Self::Data) {}
}

pub trait LinkCutTreePathFold: LinkCutTreeSpec {
    type Path;
    fn fold_path(data: &Self::Data) -> Self::Path;
}

pub trait LinkCutTreePathUpdate: LinkCutTreeSpec {
    type PathAction;
    fn update_path(data: &mut Self::Data, action: &Self::PathAction);
}

pub trait LinkCutTreeSubtreeFold: LinkCutTreeSpec {
    type Subtree;
    fn fold_subtree(data: &Self::Data) -> Self::Subtree;
}

/// Pending updates must be propagated through `top_down`, `attach_virtual`,
/// `detach_virtual`, and `transfer_path_parent`.
pub trait LinkCutTreeSubtreeUpdate: LinkCutTreeSpec {
    type SubtreeAction;
    fn update_subtree(data: &mut Self::Data, action: &Self::SubtreeAction);
}

struct LinkCutData<S>
where
    S: LinkCutTreeSpec,
{
    inner: S::Data,
    index: usize,
    reverse: bool,
}

struct LinkCutBstSpec<S>(PhantomData<fn() -> S>);

type LinkCutNode<S> = BstNode<LinkCutData<S>, WithParent<LinkCutData<S>>>;
type LinkCutPtr<S> = NonNull<LinkCutNode<S>>;

impl<S> LinkCutBstSpec<S>
where
    S: LinkCutTreeSpec,
{
    #[inline]
    unsafe fn toggle(mut node: LinkCutPtr<S>) {
        unsafe { node.as_mut().child.swap(0, 1) };
        let data = unsafe { &mut node.as_mut().data };
        data.reverse ^= true;
        S::reverse(&mut data.inner);
    }

    #[inline]
    unsafe fn with_two_inner_mut<R>(
        mut left: LinkCutPtr<S>,
        mut right: LinkCutPtr<S>,
        f: impl FnOnce(&mut S::Data, &mut S::Data) -> R,
    ) -> R {
        let left = unsafe { &mut left.as_mut().data.inner };
        let right = unsafe { &mut right.as_mut().data.inner };
        f(left, right)
    }
}

impl<S> BstSpec for LinkCutBstSpec<S>
where
    S: LinkCutTreeSpec,
{
    type Parent = WithParent<Self::Data>;
    type Data = LinkCutData<S>;

    #[inline]
    fn top_down(mut node: BstDataMutRef<'_, Self>) {
        let pointer = node.node;
        if node.reborrow().into_data().reverse {
            node.data_mut().reverse = false;
            let children = unsafe { pointer.as_ref().child };
            for child in children.into_iter().flatten() {
                unsafe { Self::toggle(child) };
            }
        }
        let children = unsafe { pointer.as_ref().child };
        let data = unsafe { &mut (*pointer.as_ptr()).data.inner };
        let children =
            children.map(|child| child.map(|child| unsafe { &mut (*child.as_ptr()).data.inner }));
        S::top_down(data, children);
    }

    #[inline]
    fn bottom_up(node: BstDataMutRef<'_, Self>) {
        let pointer = node.node;
        let children = unsafe { pointer.as_ref().child };
        let data = unsafe { &mut (*pointer.as_ptr()).data.inner };
        let children =
            children.map(|child| child.map(|child| unsafe { &(*child.as_ptr()).data.inner }));
        S::bottom_up(data, children);
    }

    fn merge(_left: Option<BstRoot<Self>>, _right: Option<BstRoot<Self>>) -> Option<BstRoot<Self>> {
        unreachable!("link-cut trees do not merge auxiliary trees through BstSpec")
    }

    fn split<Seeker>(
        _node: Option<BstRoot<Self>>,
        _seeker: Seeker,
        _equal_side: EqualSide,
    ) -> (Option<BstRoot<Self>>, Option<BstRoot<Self>>)
    where
        Seeker: BstSeeker<Spec = Self>,
    {
        unreachable!("link-cut trees do not split auxiliary trees through BstSpec")
    }
}

/// A link-cut forest with stable insertion-order node identifiers.
///
/// Its dynamic-tree operations take amortized `O(log n)` time when the spec
/// hooks take constant time.
pub struct LinkCutTree<S>
where
    S: LinkCutTreeSpec,
{
    nodes: Vec<LinkCutPtr<S>>,
    allocator: MemoryPool<LinkCutNode<S>>,
}

impl<S> LinkCutTree<S>
where
    S: LinkCutTreeSpec,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            allocator: MemoryPool::with_capacity(capacity),
        }
    }

    /// `edges` must form a tree over the values in iteration order.
    pub fn from_edges<T>(values: T, edges: &[(usize, usize)]) -> Self
    where
        T: IntoIterator<Item = S::Value>,
    {
        let tree: Self = values.into_iter().collect();
        for (child, parent, preferred) in
            splay_operations::rooted_heavy_order(tree.nodes.len(), edges)
                .into_iter()
                .rev()
        {
            let child = tree.node(child);
            let mut parent = tree.node(parent);
            unsafe {
                (*child.as_ptr()).parent.parent = Some(parent);
                if preferred {
                    parent.as_mut().child[1] = Some(child);
                } else {
                    LinkCutBstSpec::<S>::with_two_inner_mut(parent, child, S::attach_virtual);
                }
                Self::pull(parent);
            }
        }
        tree
    }

    pub fn add_node(&mut self, value: S::Value) -> usize {
        let index = self.nodes.len();
        let node = self.allocator.allocate(BstNode::new(LinkCutData {
            inner: S::new(value),
            index,
            reverse: false,
        }));
        self.nodes.push(node);
        index
    }

    #[inline]
    fn node(&self, index: usize) -> LinkCutPtr<S> {
        self.nodes[index]
    }

    #[inline]
    unsafe fn pull(node: LinkCutPtr<S>) {
        unsafe {
            LinkCutBstSpec::<S>::bottom_up(BstDataMutRef::new_unchecked(node));
        }
    }

    #[inline]
    unsafe fn splay(node: LinkCutPtr<S>) {
        let root = if S::ROOT_TO_NODE_TOP_DOWN {
            unsafe {
                splay_operations::with_parent::splay::<LinkCutBstSpec<S>, LinkCutData<S>>(node)
            }
        } else {
            unsafe {
                splay_operations::with_parent::splay_with_local_top_down::<
                    LinkCutBstSpec<S>,
                    LinkCutData<S>,
                >(node)
            }
        };
        if root != node {
            unsafe {
                LinkCutBstSpec::<S>::with_two_inner_mut(root, node, S::transfer_path_parent);
            }
        }
    }

    fn access_node(mut node: LinkCutPtr<S>) {
        unsafe {
            Self::splay(node);
            if let Some(right) = node.as_mut().child[1].take() {
                LinkCutBstSpec::<S>::with_two_inner_mut(node, right, S::attach_virtual);
            }
            Self::pull(node);
            while let Some(mut parent) = node.as_ref().parent.parent {
                Self::splay(parent);
                if let Some(right) = parent.as_mut().child[1].take() {
                    LinkCutBstSpec::<S>::with_two_inner_mut(parent, right, S::attach_virtual);
                }
                LinkCutBstSpec::<S>::with_two_inner_mut(parent, node, S::detach_virtual);
                parent.as_mut().child[1] = Some(node);
                node.as_mut().parent.parent = Some(parent);
                Self::pull(parent);
                Self::splay(node);
            }
        }
    }

    pub fn get(&mut self, node: usize) -> &S::Value {
        let node = self.node(node);
        Self::access_node(node);
        unsafe { S::value(&node.as_ref().data.inner) }
    }

    pub fn set(&mut self, node: usize, value: S::Value) {
        let node = self.node(node);
        Self::access_node(node);
        unsafe {
            *S::value_mut(&mut (*node.as_ptr()).data.inner) = value;
            Self::pull(node);
        }
    }

    pub fn modify<F>(&mut self, node: usize, f: F)
    where
        F: FnOnce(&S::Value) -> S::Value,
    {
        let node = self.node(node);
        Self::access_node(node);
        unsafe {
            let data = &mut (*node.as_ptr()).data.inner;
            *S::value_mut(data) = f(S::value(data));
            Self::pull(node);
        }
    }

    pub fn reroot(&mut self, node: usize) {
        let node = self.node(node);
        Self::access_node(node);
        unsafe { LinkCutBstSpec::<S>::toggle(node) };
    }

    /// `child` and `parent` must belong to different trees.
    pub fn link(&mut self, child: usize, parent: usize) {
        assert_ne!(child, parent);
        self.reroot(child);
        let child = self.node(child);
        let parent = self.node(parent);
        Self::access_node(parent);
        unsafe {
            (*child.as_ptr()).parent.parent = Some(parent);
            LinkCutBstSpec::<S>::with_two_inner_mut(parent, child, S::attach_virtual);
            Self::pull(parent);
        }
    }

    /// `(u, v)` must be an edge.
    pub fn cut(&mut self, u: usize, v: usize) {
        assert_ne!(u, v);
        self.reroot(u);
        let mut v = self.node(v);
        Self::access_node(v);
        unsafe {
            let mut left = v.as_mut().child[0]
                .take()
                .expect("the specified edge must exist");
            left.as_mut().parent.parent = None;
            Self::pull(v);
        }
    }

    pub fn root(&mut self, node: usize) -> usize {
        let mut root = self.node(node);
        Self::access_node(root);
        unsafe {
            loop {
                LinkCutBstSpec::<S>::top_down(BstDataMutRef::new_unchecked(root));
                match root.as_ref().child[0] {
                    Some(left) => root = left,
                    None => break,
                }
            }
            Self::splay(root);
            root.as_ref().data.index
        }
    }

    pub fn is_connected(&mut self, u: usize, v: usize) -> bool {
        self.root(u) == self.root(v)
    }

    fn detach_left<R>(node: LinkCutPtr<S>, f: impl FnOnce(&mut S::Data) -> R) -> R {
        unsafe {
            let left = (*node.as_ptr()).child[0].take();
            if let Some(mut left) = left {
                left.as_mut().parent.parent = None;
            }
            Self::pull(node);
            let result = f(&mut (*node.as_ptr()).data.inner);
            LinkCutBstSpec::<S>::top_down(BstDataMutRef::new_unchecked(node));
            (*node.as_ptr()).child[0] = left;
            if let Some(mut left) = left {
                left.as_mut().parent.parent = Some(node);
            }
            Self::pull(node);
            result
        }
    }
}

impl<S> FromIterator<S::Value> for LinkCutTree<S>
where
    S: LinkCutTreeSpec,
{
    fn from_iter<T: IntoIterator<Item = S::Value>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut tree = Self::with_capacity(lower);
        for value in iter {
            tree.add_node(value);
        }
        tree
    }
}

impl<S> LinkCutTree<S>
where
    S: LinkCutTreePathFold,
{
    /// `u` and `v` must be connected.
    pub fn fold_path(&mut self, u: usize, v: usize) -> S::Path {
        self.reroot(u);
        let v = self.node(v);
        Self::access_node(v);
        unsafe { S::fold_path(&v.as_ref().data.inner) }
    }
}

impl<S> LinkCutTree<S>
where
    S: LinkCutTreePathUpdate,
{
    /// `u` and `v` must be connected.
    pub fn update_path(&mut self, u: usize, v: usize, action: &S::PathAction) {
        self.reroot(u);
        let v = self.node(v);
        Self::access_node(v);
        unsafe { S::update_path(&mut (*v.as_ptr()).data.inner, action) };
    }
}

impl<S> LinkCutTree<S>
where
    S: LinkCutTreeSubtreeFold,
{
    /// `(node, parent)` must be an edge.
    pub fn fold_subtree(&mut self, node: usize, parent: usize) -> S::Subtree {
        self.reroot(parent);
        let node = self.node(node);
        Self::access_node(node);
        Self::detach_left(node, |data| S::fold_subtree(data))
    }
}

impl<S> LinkCutTree<S>
where
    S: LinkCutTreeSubtreeUpdate,
{
    /// `(node, parent)` must be an edge.
    pub fn update_subtree(&mut self, node: usize, parent: usize, action: &S::SubtreeAction) {
        self.reroot(parent);
        let node = self.node(node);
        Self::access_node(node);
        Self::detach_left(node, |data| S::update_subtree(data, action));
    }
}

pub struct PathLinkCutTreeData<L>
where
    L: LazyMapMonoid,
{
    value: LazyMapElement<L>,
}

pub struct PathLinkCutTreeSpec<L>(PhantomData<fn() -> L>);

impl<L> PathLinkCutTreeSpec<L>
where
    L: LazyMapMonoid,
{
    #[inline]
    fn apply_non_unit(data: &mut PathLinkCutTreeData<L>, action: &L::Act) {
        L::act_operate_assign(&mut data.value.act, action);
        data.value.key = L::act_key(&data.value.key, action);
        data.value.agg = L::act_agg(&data.value.agg, action)
            .expect("a path link-cut tree action must update aggregates lazily");
    }
}

impl<L> LinkCutTreeSpec for PathLinkCutTreeSpec<L>
where
    L: LazyMapMonoid,
{
    type Value = L::Key;
    type Data = PathLinkCutTreeData<L>;

    const ROOT_TO_NODE_TOP_DOWN: bool = false;

    fn new(value: Self::Value) -> Self::Data {
        Self::Data {
            value: LazyMapElement::from_key(value),
        }
    }

    fn value(data: &Self::Data) -> &Self::Value {
        &data.value.key
    }

    fn value_mut(data: &mut Self::Data) -> &mut Self::Value {
        &mut data.value.key
    }

    fn top_down(data: &mut Self::Data, children: [Option<&mut Self::Data>; 2]) {
        if L::is_act_unit(&data.value.act) {
            return;
        }
        let action = replace(&mut data.value.act, L::act_unit());
        for child in children.into_iter().flatten() {
            Self::apply_non_unit(child, &action);
        }
    }

    fn bottom_up(data: &mut Self::Data, children: [Option<&Self::Data>; 2]) {
        let mut aggregate = L::single_agg(&data.value.key);
        if let Some(left) = children[0] {
            aggregate = L::agg_operate(&left.value.agg, &aggregate);
        }
        if let Some(right) = children[1] {
            aggregate = L::agg_operate(&aggregate, &right.value.agg);
        }
        data.value.agg = aggregate;
    }

    fn reverse(data: &mut Self::Data) {
        L::toggle(&mut data.value.agg);
    }
}

impl<L> LinkCutTreePathFold for PathLinkCutTreeSpec<L>
where
    L: LazyMapMonoid,
{
    type Path = L::Agg;

    fn fold_path(data: &Self::Data) -> Self::Path {
        data.value.agg.clone()
    }
}

impl<L> LinkCutTreePathUpdate for PathLinkCutTreeSpec<L>
where
    L: LazyMapMonoid,
{
    type PathAction = L::Act;

    fn update_path(data: &mut Self::Data, action: &Self::PathAction) {
        if !L::is_act_unit(action) {
            Self::apply_non_unit(data, action);
        }
    }
}

/// `L::act_agg` must return `Some` for every action.
pub type PathLinkCutTree<L> = LinkCutTree<PathLinkCutTreeSpec<L>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{Associative, EmptyAct, Magma, RangeSumRangeLinear, Unital},
        graph::UndirectedSparseGraph,
        tools::Xorshift,
        tree::{MixedTree, PathTree, StarTree},
    };

    fn adjacency(graph: &UndirectedSparseGraph) -> Vec<Vec<usize>> {
        graph
            .vertices()
            .map(|u| graph.adjacencies(u).map(|a| a.to).collect())
            .collect()
    }

    fn naive_path(adjacency: &[Vec<usize>], start: usize, goal: usize) -> Vec<usize> {
        let mut parent = vec![usize::MAX; adjacency.len()];
        let mut stack = vec![start];
        while let Some(u) = stack.pop() {
            for &v in &adjacency[u] {
                if v != parent[u] {
                    parent[v] = u;
                    stack.push(v);
                }
            }
        }
        let mut current = goal;
        let mut path = Vec::new();
        loop {
            path.push(current);
            if current == start {
                break;
            }
            current = parent[current];
        }
        path.reverse();
        path
    }

    fn naive_subtree(adjacency: &[Vec<usize>], root: usize, parent: usize) -> Vec<usize> {
        let mut subtree = Vec::new();
        let mut stack = vec![(root, parent)];
        while let Some((u, parent)) = stack.pop() {
            subtree.push(u);
            stack.extend(
                adjacency[u]
                    .iter()
                    .filter(|&&v| v != parent)
                    .map(|&v| (v, u)),
            );
        }
        subtree
    }

    fn rewire(
        adjacency: &mut [Vec<usize>],
        edges: &mut [(usize, usize)],
        rng: &mut Xorshift,
    ) -> (usize, usize, usize, usize) {
        let edge = rng.random(0..edges.len());
        let (u, v) = edges[edge];
        adjacency[u].retain(|&to| to != v);
        adjacency[v].retain(|&to| to != u);

        let mut component = vec![false; adjacency.len()];
        let mut stack = vec![u];
        component[u] = true;
        while let Some(x) = stack.pop() {
            for &to in &adjacency[x] {
                if !component[to] {
                    component[to] = true;
                    stack.push(to);
                }
            }
        }
        let left = (0..adjacency.len())
            .filter(|&x| component[x])
            .collect::<Vec<_>>();
        let right = (0..adjacency.len())
            .filter(|&x| !component[x])
            .collect::<Vec<_>>();
        let a = left[rng.random(0..left.len())];
        let b = right[rng.random(0..right.len())];

        adjacency[a].push(b);
        adjacency[b].push(a);
        edges[edge] = (a, b);
        (u, v, a, b)
    }

    struct SubtreeSum;

    struct SubtreeSumData {
        value: i64,
        virtual_sum: i64,
        virtual_size: i64,
        sum: i64,
        size: i64,
        lazy: i64,
        virtual_lazy: i64,
        path_parent_lazy: i64,
    }

    impl SubtreeSum {
        fn apply(data: &mut SubtreeSumData, action: i64) {
            data.value += action;
            data.virtual_sum += data.virtual_size * action;
            data.sum += data.size * action;
            data.lazy += action;
            data.virtual_lazy += action;
        }
    }

    impl LinkCutTreeSpec for SubtreeSum {
        type Value = i64;
        type Data = SubtreeSumData;

        fn new(value: Self::Value) -> Self::Data {
            SubtreeSumData {
                value,
                virtual_sum: 0,
                virtual_size: 0,
                sum: value,
                size: 1,
                lazy: 0,
                virtual_lazy: 0,
                path_parent_lazy: 0,
            }
        }

        fn value(data: &Self::Data) -> &Self::Value {
            &data.value
        }

        fn value_mut(data: &mut Self::Data) -> &mut Self::Value {
            &mut data.value
        }

        fn top_down(data: &mut Self::Data, children: [Option<&mut Self::Data>; 2]) {
            let action = replace(&mut data.lazy, 0);
            for child in children.into_iter().flatten() {
                Self::apply(child, action);
            }
        }

        fn bottom_up(data: &mut Self::Data, children: [Option<&Self::Data>; 2]) {
            data.sum = data.value
                + data.virtual_sum
                + children
                    .into_iter()
                    .flatten()
                    .map(|child| child.sum)
                    .sum::<i64>();
            data.size = 1
                + data.virtual_size
                + children
                    .into_iter()
                    .flatten()
                    .map(|child| child.size)
                    .sum::<i64>();
        }

        fn reverse(_data: &mut Self::Data) {}

        fn attach_virtual(parent: &mut Self::Data, child: &mut Self::Data) {
            child.path_parent_lazy = parent.virtual_lazy;
            parent.virtual_sum += child.sum;
            parent.virtual_size += child.size;
        }

        fn detach_virtual(parent: &mut Self::Data, child: &mut Self::Data) {
            Self::apply(child, parent.virtual_lazy - child.path_parent_lazy);
            parent.virtual_sum -= child.sum;
            parent.virtual_size -= child.size;
        }

        fn transfer_path_parent(old_root: &mut Self::Data, new_root: &mut Self::Data) {
            new_root.path_parent_lazy = replace(&mut old_root.path_parent_lazy, 0);
        }
    }

    impl LinkCutTreeSubtreeFold for SubtreeSum {
        type Subtree = i64;

        fn fold_subtree(data: &Self::Data) -> Self::Subtree {
            data.sum
        }
    }

    impl LinkCutTreeSubtreeUpdate for SubtreeSum {
        type SubtreeAction = i64;

        fn update_subtree(data: &mut Self::Data, action: &Self::SubtreeAction) {
            Self::apply(data, *action);
        }
    }

    struct BidirectionalString;

    impl Magma for BidirectionalString {
        type T = (String, String);

        fn operate(left: &Self::T, right: &Self::T) -> Self::T {
            (left.0.clone() + &right.0, right.1.clone() + &left.1)
        }
    }

    impl Unital for BidirectionalString {
        fn unit() -> Self::T {
            (String::new(), String::new())
        }
    }

    impl Associative for BidirectionalString {}

    struct StringPath;

    impl LazyMapMonoid for StringPath {
        type Key = char;
        type Agg = (String, String);
        type Act = ();
        type AggMonoid = BidirectionalString;
        type ActMonoid = ();
        type KeyAct = EmptyAct<char>;

        fn single_agg(key: &Self::Key) -> Self::Agg {
            (key.to_string(), key.to_string())
        }

        fn toggle(value: &mut Self::Agg) {
            std::mem::swap(&mut value.0, &mut value.1);
        }

        fn act_agg(value: &Self::Agg, _action: &Self::Act) -> Option<Self::Agg> {
            Some(value.clone())
        }
    }

    fn run_path_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let mut adjacency = adjacency(graph);
        let mut edges = graph.edges.clone();
        let mut values = (0..n).map(|_| rng.random(-20i64..=20)).collect::<Vec<_>>();
        let mut tree =
            PathLinkCutTree::<RangeSumRangeLinear<i64>>::from_edges(values.iter().copied(), &edges);
        let root = rng.random(0..n);
        tree.reroot(root);
        for u in 0..n {
            assert_eq!(tree.root(u), root);
        }

        for _ in 0..rounds {
            match rng.random(0..if edges.is_empty() { 2 } else { 3 }) {
                0 => {
                    let u = rng.random(0..n);
                    if rng.random(0..2) == 0 {
                        values[u] = rng.random(-20i64..=20);
                        tree.set(u, values[u]);
                    } else {
                        let action = rng.random(-20i64..=20);
                        values[u] += action;
                        tree.modify(u, |value| *value + action);
                    }
                }
                1 => {
                    let u = rng.random(0..n);
                    let v = rng.random(0..n);
                    let action = (rng.random(0i64..=1), rng.random(-20i64..=20));
                    let path = naive_path(&adjacency, u, v);
                    for &x in &path {
                        values[x] = action.0 * values[x] + action.1;
                    }
                    tree.update_path(u, v, &action);
                }
                _ => {
                    let (u, v, a, b) = rewire(&mut adjacency, &mut edges, rng);
                    tree.cut(u, v);
                    assert!(!tree.is_connected(u, v));
                    tree.link(a, b);
                    assert!(tree.is_connected(u, v));
                }
            }

            let u = rng.random(0..n);
            let v = rng.random(0..n);
            let path = naive_path(&adjacency, u, v);
            assert_eq!(
                tree.fold_path(u, v),
                (path.iter().map(|&x| values[x]).sum(), path.len() as i64)
            );
            let u = rng.random(0..n);
            assert_eq!(*tree.get(u), values[u]);
        }
    }

    fn run_ordered_path_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let mut adjacency = adjacency(graph);
        let mut edges = graph.edges.clone();
        let mut values = (0..n)
            .map(|_| rng.random(b'a'..=b'z') as char)
            .collect::<Vec<_>>();
        let mut tree = PathLinkCutTree::<StringPath>::from_edges(values.iter().copied(), &edges);

        for _ in 0..rounds {
            match rng.random(0..if edges.is_empty() { 2 } else { 3 }) {
                0 => {
                    let u = rng.random(0..n);
                    values[u] = rng.random(b'a'..=b'z') as char;
                    if rng.random(0..2) == 0 {
                        tree.set(u, values[u]);
                    } else {
                        tree.modify(u, |_| values[u]);
                    }
                }
                1 => {
                    let u = rng.random(0..n);
                    tree.reroot(u);
                    tree.reroot(u);
                }
                _ => {
                    let (u, v, a, b) = rewire(&mut adjacency, &mut edges, rng);
                    tree.cut(u, v);
                    tree.link(a, b);
                }
            }

            let u = rng.random(0..n);
            let v = rng.random(0..n);
            assert_eq!(
                tree.fold_path(u, v).0,
                naive_path(&adjacency, u, v)
                    .into_iter()
                    .map(|u| values[u])
                    .collect::<String>()
            );
        }
    }

    fn run_subtree_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let mut adjacency = adjacency(graph);
        let mut edges = graph.edges.clone();
        let mut values = (0..n).map(|_| rng.random(-20i64..=20)).collect::<Vec<_>>();
        let mut tree = LinkCutTree::<SubtreeSum>::from_edges(values.iter().copied(), &edges);

        for _ in 0..rounds {
            match rng.random(0..3) {
                0 => {
                    let (u, v, a, b) = rewire(&mut adjacency, &mut edges, rng);
                    tree.cut(u, v);
                    tree.link(a, b);
                }
                1 => {
                    let u = rng.random(0..n);
                    if rng.random(0..2) == 0 {
                        values[u] = rng.random(-20i64..=20);
                        tree.set(u, values[u]);
                    } else {
                        let action = rng.random(-20i64..=20);
                        values[u] += action;
                        tree.modify(u, |value| *value + action);
                    }
                }
                _ => {
                    let &(u, v) = &edges[rng.random(0..edges.len())];
                    let (node, parent) = if rng.random(0..2) == 0 {
                        (u, v)
                    } else {
                        (v, u)
                    };
                    let subtree = naive_subtree(&adjacency, node, parent);
                    let action = rng.random(-20i64..=20);
                    for &x in &subtree {
                        values[x] += action;
                    }
                    tree.update_subtree(node, parent, &action);
                }
            }

            let &(u, v) = &edges[rng.random(0..edges.len())];
            for (node, parent) in [(u, v), (v, u)] {
                let subtree = naive_subtree(&adjacency, node, parent);
                assert_eq!(
                    tree.fold_subtree(node, parent),
                    subtree.iter().map(|&x| values[x]).sum()
                );
            }
            let u = rng.random(0..n);
            assert_eq!(*tree.get(u), values[u]);
        }
    }

    #[test]
    fn path_link_cut_tree() {
        let mut rng = Xorshift::default();
        for n in 1..=14 {
            for graph in [rng.random(PathTree(n)), rng.random(StarTree(n))] {
                run_path_case(&graph, 300, &mut rng);
                run_ordered_path_case(&graph, 300, &mut rng);
            }
        }
        for _ in 0..20 {
            let graph = rng.random(MixedTree(1..=14usize));
            run_path_case(&graph, 300, &mut rng);
            run_ordered_path_case(&graph, 300, &mut rng);
        }
    }

    #[test]
    fn link_cut_tree_subtree() {
        let mut rng = Xorshift::default();
        for n in 2..=14 {
            for graph in [rng.random(PathTree(n)), rng.random(StarTree(n))] {
                run_subtree_case(&graph, 300, &mut rng);
            }
        }
        for _ in 0..20 {
            let graph = rng.random(MixedTree(2..=14usize));
            run_subtree_case(&graph, 300, &mut rng);
        }
    }
}
