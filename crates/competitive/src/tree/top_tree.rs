use super::{
    Allocator, Magma, MemoryPool, Monoid, Unital,
    binary_search_tree::{
        BstDataMutRef, BstNode, BstRoot, BstSeeker, BstSpec, EqualSide, node::WithParent,
    },
    splay_operations,
};
use std::{marker::PhantomData, mem::replace, ptr::NonNull};

/// `compress` must be associative and `rake` must be associative and
/// commutative. `reverse` must be an involution and reverse the operand order
/// of `compress`.
pub trait TopTreeSpec: Sized {
    type Info;
    type Point: Clone;
    type Path: Clone;

    fn vertex(info: &Self::Info) -> Self::Path;
    fn add_vertex(point: &Self::Point, info: &Self::Info) -> Self::Path;
    fn add_edge(path: &Self::Path) -> Self::Point;
    fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point;
    fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path;
    fn reverse(path: &mut Self::Path);
}

/// `act_path` applies the heavy contribution and `act_path_light` applies the
/// light contribution represented in the same path aggregate.
pub trait TopTreeAction<S>
where
    S: TopTreeSpec,
{
    type Action: PartialEq;
    /// Composes actions in application order: `operate(a, b)` applies `a`
    /// before `b`.
    type ActionMonoid: Monoid<T = Self::Action>;

    fn act_info(info: &mut S::Info, action: &Self::Action);
    fn act_point(point: &mut S::Point, action: &Self::Action);
    fn act_path(path: &mut S::Path, action: &Self::Action);
    fn act_path_light(path: &mut S::Path, action: &Self::Action);
}

pub struct NoTopTreeAction;

impl<S> TopTreeAction<S> for NoTopTreeAction
where
    S: TopTreeSpec,
{
    type Action = ();
    type ActionMonoid = ();

    fn act_info(_info: &mut S::Info, _action: &Self::Action) {}
    fn act_point(_point: &mut S::Point, _action: &Self::Action) {}
    fn act_path(_path: &mut S::Path, _action: &Self::Action) {}
    fn act_path_light(_path: &mut S::Path, _action: &Self::Action) {}
}

struct TopTreeData<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    info: S::Info,
    sum: S::Path,
    reverse: bool,
    light: Option<RakePtr<S, A>>,
    belong: Option<RakePtr<S, A>>,
    heavy_action: A::Action,
    light_action: A::Action,
    index: usize,
}

struct RakeData<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    key: S::Point,
    sum: S::Point,
    action: A::Action,
    buffer: A::Action,
}

struct TopBstSpec<S, A>(PhantomData<fn() -> (S, A)>);
struct RakeBstSpec<S, A>(PhantomData<fn() -> (S, A)>);

type TopNode<S, A> = BstNode<TopTreeData<S, A>, WithParent<TopTreeData<S, A>>>;
type RakeNode<S, A> = BstNode<RakeData<S, A>, WithParent<RakeData<S, A>>>;
type TopPtr<S, A> = NonNull<TopNode<S, A>>;
type RakePtr<S, A> = NonNull<RakeNode<S, A>>;

impl<S, A> TopBstSpec<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    #[inline]
    fn is_unit(action: &A::Action) -> bool {
        <A::ActionMonoid as Unital>::is_unit(action)
    }

    #[inline]
    fn compose(target: &mut A::Action, action: &A::Action) {
        <A::ActionMonoid as Magma>::operate_assign(target, action);
    }

    #[inline]
    unsafe fn toggle(mut node: TopPtr<S, A>) {
        unsafe { node.as_mut().child.swap(0, 1) };
        let data = unsafe { &mut node.as_mut().data };
        data.reverse ^= true;
        S::reverse(&mut data.sum);
    }

    #[inline]
    unsafe fn apply_heavy(mut node: TopPtr<S, A>, action: &A::Action) {
        let data = unsafe { &mut node.as_mut().data };
        Self::compose(&mut data.heavy_action, action);
        A::act_info(&mut data.info, action);
        A::act_path(&mut data.sum, action);
    }

    #[inline]
    unsafe fn apply_light(mut node: TopPtr<S, A>, action: &A::Action) {
        let data = unsafe { &mut node.as_mut().data };
        Self::compose(&mut data.light_action, action);
        A::act_path_light(&mut data.sum, action);
    }

    #[inline]
    unsafe fn apply_all(mut node: TopPtr<S, A>, action: &A::Action) {
        if Self::is_unit(action) {
            return;
        }
        let data = unsafe { &mut node.as_mut().data };
        Self::compose(&mut data.heavy_action, action);
        Self::compose(&mut data.light_action, action);
        A::act_info(&mut data.info, action);
        A::act_path(&mut data.sum, action);
        A::act_path_light(&mut data.sum, action);
    }
}

impl<S, A> BstSpec for TopBstSpec<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    type Parent = WithParent<Self::Data>;
    type Data = TopTreeData<S, A>;

    #[inline]
    fn top_down(mut node: BstDataMutRef<'_, Self>) {
        let pointer = node.node;
        if node.reborrow().into_data().reverse {
            node.data_mut().reverse = false;
            for child in unsafe { pointer.as_ref().child }.into_iter().flatten() {
                unsafe { Self::toggle(child) };
            }
        }

        if !Self::is_unit(&node.reborrow().into_data().heavy_action) {
            let action = replace(
                &mut node.data_mut().heavy_action,
                <A::ActionMonoid as Unital>::unit(),
            );
            for child in unsafe { pointer.as_ref().child }.into_iter().flatten() {
                unsafe { Self::apply_heavy(child, &action) };
            }
        }
        if !Self::is_unit(&node.reborrow().into_data().light_action) {
            let action = replace(
                &mut node.data_mut().light_action,
                <A::ActionMonoid as Unital>::unit(),
            );
            for child in unsafe { pointer.as_ref().child }.into_iter().flatten() {
                unsafe { Self::apply_light(child, &action) };
            }
            if let Some(light) = node.reborrow().into_data().light {
                unsafe { RakeBstSpec::<S, A>::apply(light, &action) };
            }
        }
    }

    #[inline]
    fn bottom_up(node: BstDataMutRef<'_, Self>) {
        let pointer = node.node;
        let data = unsafe { &mut (*pointer.as_ptr()).data };
        let mut sum = if let Some(light) = data.light {
            S::add_vertex(unsafe { &light.as_ref().data.sum }, &data.info)
        } else {
            S::vertex(&data.info)
        };
        if let Some(left) = unsafe { pointer.as_ref().child[0] } {
            sum = S::compress(unsafe { &left.as_ref().data.sum }, &sum);
        }
        if let Some(right) = unsafe { pointer.as_ref().child[1] } {
            sum = S::compress(&sum, unsafe { &right.as_ref().data.sum });
        }
        data.sum = sum;
    }

    fn merge(_left: Option<BstRoot<Self>>, _right: Option<BstRoot<Self>>) -> Option<BstRoot<Self>> {
        unreachable!("top trees do not merge auxiliary trees through BstSpec")
    }

    fn split<Seeker>(
        _node: Option<BstRoot<Self>>,
        _seeker: Seeker,
        _equal_side: EqualSide,
    ) -> (Option<BstRoot<Self>>, Option<BstRoot<Self>>)
    where
        Seeker: BstSeeker<Spec = Self>,
    {
        unreachable!("top trees do not split auxiliary trees through BstSpec")
    }
}

impl<S, A> RakeBstSpec<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    #[inline]
    unsafe fn apply(mut node: RakePtr<S, A>, action: &A::Action) {
        let data = unsafe { &mut node.as_mut().data };
        A::act_point(&mut data.key, action);
        A::act_point(&mut data.sum, action);
        TopBstSpec::<S, A>::compose(&mut data.action, action);
        TopBstSpec::<S, A>::compose(&mut data.buffer, action);
    }
}

impl<S, A> BstSpec for RakeBstSpec<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    type Parent = WithParent<Self::Data>;
    type Data = RakeData<S, A>;

    #[inline]
    fn top_down(mut node: BstDataMutRef<'_, Self>) {
        if TopBstSpec::<S, A>::is_unit(&node.reborrow().into_data().action) {
            return;
        }
        let pointer = node.node;
        let action = replace(
            &mut node.data_mut().action,
            <A::ActionMonoid as Unital>::unit(),
        );
        for child in unsafe { pointer.as_ref().child }.into_iter().flatten() {
            unsafe { Self::apply(child, &action) };
        }
    }

    #[inline]
    fn bottom_up(node: BstDataMutRef<'_, Self>) {
        let pointer = node.node;
        let data = unsafe { &mut (*pointer.as_ptr()).data };
        let mut sum = data.key.clone();
        if let Some(left) = unsafe { pointer.as_ref().child[0] } {
            sum = S::rake(&sum, unsafe { &left.as_ref().data.sum });
        }
        if let Some(right) = unsafe { pointer.as_ref().child[1] } {
            sum = S::rake(&sum, unsafe { &right.as_ref().data.sum });
        }
        data.sum = sum;
    }

    fn merge(_left: Option<BstRoot<Self>>, _right: Option<BstRoot<Self>>) -> Option<BstRoot<Self>> {
        unreachable!("rake trees use their dedicated merge operation")
    }

    fn split<Seeker>(
        _node: Option<BstRoot<Self>>,
        _seeker: Seeker,
        _equal_side: EqualSide,
    ) -> (Option<BstRoot<Self>>, Option<BstRoot<Self>>)
    where
        Seeker: BstSeeker<Spec = Self>,
    {
        unreachable!("rake trees do not use BstSpec::split")
    }
}

/// A self-adjusting top tree, also called a strong link-cut tree.
///
/// This is not a classical worst-case-balanced top tree. Circular order and
/// `select` are not supported.
pub struct TopTree<S, A = NoTopTreeAction>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    nodes: Vec<TopPtr<S, A>>,
    node_allocator: MemoryPool<TopNode<S, A>>,
    rake_allocator: MemoryPool<RakeNode<S, A>>,
}

impl<S, A> TopTree<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            node_allocator: MemoryPool::with_capacity(capacity),
            rake_allocator: MemoryPool::with_capacity(capacity),
        }
    }

    pub fn add_node(&mut self, info: S::Info) -> usize {
        let index = self.nodes.len();
        let sum = S::vertex(&info);
        let node = self.node_allocator.allocate(BstNode::new(TopTreeData {
            info,
            sum,
            reverse: false,
            light: None,
            belong: None,
            heavy_action: <A::ActionMonoid as Unital>::unit(),
            light_action: <A::ActionMonoid as Unital>::unit(),
            index,
        }));
        self.nodes.push(node);
        index
    }

    fn node(&self, index: usize) -> TopPtr<S, A> {
        self.nodes[index]
    }

    #[inline]
    unsafe fn pull_top(node: TopPtr<S, A>) {
        unsafe { TopBstSpec::<S, A>::bottom_up(BstDataMutRef::new_unchecked(node)) };
    }

    #[inline]
    unsafe fn pull_rake(node: RakePtr<S, A>) {
        unsafe { RakeBstSpec::<S, A>::bottom_up(BstDataMutRef::new_unchecked(node)) };
    }

    #[inline]
    unsafe fn splay_top(node: TopPtr<S, A>) {
        let root = unsafe {
            splay_operations::with_parent::splay::<TopBstSpec<S, A>, TopTreeData<S, A>>(node)
        };
        if root != node {
            unsafe {
                (*node.as_ptr()).data.belong = (*root.as_ptr()).data.belong.take();
            }
        }
    }

    #[inline]
    unsafe fn splay_rake(node: RakePtr<S, A>) {
        unsafe {
            splay_operations::with_parent::splay_with_local_top_down::<
                RakeBstSpec<S, A>,
                RakeData<S, A>,
            >(node)
        };
    }

    unsafe fn rake_rightmost(mut node: RakePtr<S, A>) -> RakePtr<S, A> {
        loop {
            unsafe { RakeBstSpec::<S, A>::top_down(BstDataMutRef::new_unchecked(node)) };
            match unsafe { node.as_ref().child[1] } {
                Some(right) => node = right,
                None => return node,
            }
        }
    }

    unsafe fn rake_insert(
        &mut self,
        root: Option<RakePtr<S, A>>,
        key: S::Point,
    ) -> (RakePtr<S, A>, RakePtr<S, A>) {
        let node = self.rake_allocator.allocate(BstNode::new(RakeData {
            sum: key.clone(),
            key,
            action: <A::ActionMonoid as Unital>::unit(),
            buffer: <A::ActionMonoid as Unital>::unit(),
        }));
        if let Some(root) = root {
            let mut rightmost = unsafe { Self::rake_rightmost(root) };
            unsafe {
                Self::splay_rake(rightmost);
                rightmost.as_mut().child[1] = Some(node);
                (*node.as_ptr()).parent.parent = Some(rightmost);
                Self::pull_rake(rightmost);
            }
            (rightmost, node)
        } else {
            (node, node)
        }
    }

    unsafe fn rake_remove(
        &mut self,
        mut node: RakePtr<S, A>,
    ) -> (Option<RakePtr<S, A>>, A::Action) {
        unsafe {
            Self::splay_rake(node);
            RakeBstSpec::<S, A>::top_down(BstDataMutRef::new_unchecked(node));
        }
        let left = unsafe { node.as_mut().child[0].take() };
        let right = unsafe { node.as_mut().child[1].take() };
        for mut child in [left, right].into_iter().flatten() {
            unsafe { child.as_mut().parent.parent = None };
        }
        let root = match (left, right) {
            (None, right) => right,
            (left, None) => left,
            (Some(left), Some(right)) => {
                let mut root = unsafe { Self::rake_rightmost(left) };
                unsafe {
                    Self::splay_rake(root);
                    root.as_mut().child[1] = Some(right);
                    (*right.as_ptr()).parent.parent = Some(root);
                    Self::pull_rake(root);
                }
                Some(root)
            }
        };
        let node = self.rake_allocator.deallocate(node);
        (root, node.data.buffer)
    }

    fn access_node(&mut self, node: TopPtr<S, A>) {
        unsafe {
            let mut previous: Option<TopPtr<S, A>> = None;
            let mut current = Some(node);
            while let Some(mut cursor) = current {
                Self::splay_top(cursor);
                let next = cursor.as_ref().parent.parent;
                if let Some(right) = cursor.as_mut().child[1].take() {
                    let point = S::add_edge(&right.as_ref().data.sum);
                    let (light, entry) = self.rake_insert(cursor.as_ref().data.light, point);
                    cursor.as_mut().data.light = Some(light);
                    (*right.as_ptr()).data.belong = Some(entry);
                }
                if let Some(previous) = previous {
                    let entry = (*previous.as_ptr())
                        .data
                        .belong
                        .take()
                        .expect("a virtual path must have a rake-tree entry");
                    let (light, action) = self.rake_remove(entry);
                    cursor.as_mut().data.light = light;
                    TopBstSpec::<S, A>::apply_all(previous, &action);
                    cursor.as_mut().child[1] = Some(previous);
                    (*previous.as_ptr()).parent.parent = Some(cursor);
                }
                Self::pull_top(cursor);
                previous = Some(cursor);
                current = next;
            }
            Self::splay_top(node);
        }
    }

    pub fn get(&mut self, node: usize) -> &S::Info {
        let node = self.node(node);
        self.access_node(node);
        unsafe { &node.as_ref().data.info }
    }

    pub fn set(&mut self, node: usize, info: S::Info) {
        let mut node = self.node(node);
        self.access_node(node);
        unsafe {
            node.as_mut().data.info = info;
            Self::pull_top(node);
        }
    }

    pub fn reroot(&mut self, node: usize) {
        let node = self.node(node);
        self.access_node(node);
        unsafe { TopBstSpec::<S, A>::toggle(node) };
    }

    /// `child` and `parent` must belong to different trees.
    pub fn link(&mut self, child: usize, parent: usize) {
        assert_ne!(child, parent);
        self.reroot(child);
        let child = self.node(child);
        let mut parent = self.node(parent);
        self.access_node(parent);
        unsafe {
            (*child.as_ptr()).parent.parent = Some(parent);
            let point = S::add_edge(&child.as_ref().data.sum);
            let (light, entry) = self.rake_insert(parent.as_ref().data.light, point);
            parent.as_mut().data.light = Some(light);
            (*child.as_ptr()).data.belong = Some(entry);
            Self::pull_top(parent);
        }
    }

    /// `(u, v)` must be an edge.
    pub fn cut(&mut self, u: usize, v: usize) {
        assert_ne!(u, v);
        self.reroot(u);
        let mut v = self.node(v);
        self.access_node(v);
        unsafe {
            let mut left = v.as_mut().child[0]
                .take()
                .expect("the specified edge must exist");
            left.as_mut().parent.parent = None;
            Self::pull_top(v);
        }
    }

    pub fn root(&mut self, node: usize) -> usize {
        let mut root = self.node(node);
        self.access_node(root);
        unsafe {
            loop {
                TopBstSpec::<S, A>::top_down(BstDataMutRef::new_unchecked(root));
                match root.as_ref().child[0] {
                    Some(left) => root = left,
                    None => break,
                }
            }
            Self::splay_top(root);
            root.as_ref().data.index
        }
    }

    pub fn is_connected(&mut self, u: usize, v: usize) -> bool {
        self.root(u) == self.root(v)
    }

    /// `u` and `v` must be connected.
    pub fn fold_path(&mut self, u: usize, v: usize) -> S::Path {
        self.reroot(u);
        let v = self.node(v);
        self.access_node(v);
        unsafe { v.as_ref().data.sum.clone() }
    }

    /// `u` and `v` must be connected.
    pub fn update_path(&mut self, u: usize, v: usize, action: &A::Action) {
        self.reroot(u);
        let v = self.node(v);
        self.access_node(v);
        if !TopBstSpec::<S, A>::is_unit(action) {
            unsafe { TopBstSpec::<S, A>::apply_heavy(v, action) };
        }
    }

    fn detach_left<R>(mut node: TopPtr<S, A>, f: impl FnOnce(TopPtr<S, A>) -> R) -> R {
        unsafe {
            let left = node.as_mut().child[0].take();
            if let Some(mut left) = left {
                left.as_mut().parent.parent = None;
            }
            Self::pull_top(node);
            let result = f(node);
            node.as_mut().child[0] = left;
            if let Some(mut left) = left {
                left.as_mut().parent.parent = Some(node);
            }
            Self::pull_top(node);
            result
        }
    }

    /// `(node, parent)` must be an edge.
    pub fn fold_subtree(&mut self, node: usize, parent: usize) -> S::Path {
        self.reroot(parent);
        let node = self.node(node);
        self.access_node(node);
        Self::detach_left(node, |node| unsafe { node.as_ref().data.sum.clone() })
    }

    /// `(node, parent)` must be an edge.
    pub fn update_subtree(&mut self, node: usize, parent: usize, action: &A::Action) {
        self.reroot(parent);
        let node = self.node(node);
        self.access_node(node);
        Self::detach_left(node, |node| unsafe {
            TopBstSpec::<S, A>::apply_all(node, action);
            TopBstSpec::<S, A>::top_down(BstDataMutRef::new_unchecked(node));
        });
    }
}

impl<S, A> FromIterator<S::Info> for TopTree<S, A>
where
    S: TopTreeSpec,
    A: TopTreeAction<S>,
{
    fn from_iter<T: IntoIterator<Item = S::Info>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut tree = Self::with_capacity(lower);
        for info in iter {
            tree.add_node(info);
        }
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::LastOperation,
        graph::UndirectedSparseGraph,
        tools::Xorshift,
        tree::{MixedTree, PathTree, StarTree},
    };

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

    struct SumTopTree;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct SumPath {
        sum: i64,
        path_sum: i64,
        path_size: i64,
        size: i64,
    }

    impl TopTreeSpec for SumTopTree {
        type Info = i64;
        type Point = (i64, i64);
        type Path = SumPath;

        fn vertex(info: &Self::Info) -> Self::Path {
            SumPath {
                sum: *info,
                path_sum: *info,
                path_size: 1,
                size: 1,
            }
        }

        fn add_vertex(point: &Self::Point, info: &Self::Info) -> Self::Path {
            SumPath {
                sum: point.0 + *info,
                path_sum: *info,
                path_size: 1,
                size: point.1 + 1,
            }
        }

        fn add_edge(path: &Self::Path) -> Self::Point {
            (path.sum, path.size)
        }

        fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point {
            (left.0 + right.0, left.1 + right.1)
        }

        fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path {
            SumPath {
                sum: left.sum + right.sum,
                path_sum: left.path_sum + right.path_sum,
                path_size: left.path_size + right.path_size,
                size: left.size + right.size,
            }
        }

        fn reverse(_path: &mut Self::Path) {}
    }

    struct AssignAction;

    impl TopTreeAction<SumTopTree> for AssignAction {
        type Action = Option<i64>;
        type ActionMonoid = LastOperation<i64>;

        fn act_info(info: &mut i64, action: &Self::Action) {
            if let Some(action) = action {
                *info = *action;
            }
        }

        fn act_point(point: &mut (i64, i64), action: &Self::Action) {
            if let Some(action) = action {
                point.0 = point.1 * *action;
            }
        }

        fn act_path(path: &mut SumPath, action: &Self::Action) {
            if let Some(action) = action {
                let path_sum = path.path_size * *action;
                path.sum += path_sum - path.path_sum;
                path.path_sum = path_sum;
            }
        }

        fn act_path_light(path: &mut SumPath, action: &Self::Action) {
            if let Some(action) = action {
                path.sum = path.path_sum + (path.size - path.path_size) * *action;
            }
        }
    }

    struct ConcatenatePath;

    impl TopTreeSpec for ConcatenatePath {
        type Info = char;
        type Point = ();
        type Path = String;

        fn vertex(info: &Self::Info) -> Self::Path {
            info.to_string()
        }

        fn add_vertex(_point: &Self::Point, info: &Self::Info) -> Self::Path {
            info.to_string()
        }

        fn add_edge(_path: &Self::Path) -> Self::Point {}
        fn rake(_left: &Self::Point, _right: &Self::Point) -> Self::Point {}

        fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path {
            let mut result = String::with_capacity(left.len() + right.len());
            result.push_str(left);
            result.push_str(right);
            result
        }

        fn reverse(path: &mut Self::Path) {
            *path = path.chars().rev().collect();
        }
    }

    fn run_top_tree_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let mut adjacency = graph
            .vertices()
            .map(|u| graph.adjacencies(u).map(|a| a.to).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut edges = graph.edges.clone();
        let mut values = (0..n).map(|_| rng.random(-20i64..=20)).collect::<Vec<_>>();
        let mut tree = TopTree::<SumTopTree, AssignAction>::from_iter(values.iter().copied());
        for &(u, v) in &edges {
            tree.link(u, v);
        }
        let root = rng.random(0..n);
        tree.reroot(root);
        for u in 0..n {
            assert_eq!(tree.root(u), root);
        }

        for _ in 0..rounds {
            match rng.random(0..if edges.is_empty() { 2 } else { 4 }) {
                0 => {
                    let u = rng.random(0..n);
                    values[u] = rng.random(-20i64..=20);
                    tree.set(u, values[u]);
                }
                1 => {
                    let u = rng.random(0..n);
                    let v = rng.random(0..n);
                    let path = naive_path(&adjacency, u, v);
                    let action = rng.random(-20i64..=20);
                    for &x in &path {
                        values[x] = action;
                    }
                    tree.update_path(u, v, &Some(action));
                }
                2 => {
                    let (u, v, a, b) = rewire(&mut adjacency, &mut edges, rng);
                    tree.cut(u, v);
                    assert!(!tree.is_connected(u, v));
                    tree.link(a, b);
                    assert!(tree.is_connected(u, v));
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
                        values[x] = action;
                    }
                    tree.update_subtree(node, parent, &Some(action));
                }
            }

            let u = rng.random(0..n);
            let v = rng.random(0..n);
            let path = naive_path(&adjacency, u, v);
            assert_eq!(
                tree.fold_path(u, v),
                SumPath {
                    sum: values.iter().sum(),
                    path_sum: path.iter().map(|&x| values[x]).sum(),
                    path_size: path.len() as i64,
                    size: n as i64,
                }
            );

            if !edges.is_empty() {
                let &(u, v) = &edges[rng.random(0..edges.len())];
                for (node, parent) in [(u, v), (v, u)] {
                    let subtree = naive_subtree(&adjacency, node, parent);
                    assert_eq!(
                        tree.fold_subtree(node, parent),
                        SumPath {
                            sum: subtree.iter().map(|&x| values[x]).sum(),
                            path_sum: values[node],
                            path_size: 1,
                            size: subtree.len() as i64,
                        }
                    );
                }
            }
            let u = rng.random(0..n);
            assert_eq!(*tree.get(u), values[u]);
        }
    }

    fn run_ordered_path_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let mut adjacency = graph
            .vertices()
            .map(|u| graph.adjacencies(u).map(|a| a.to).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut edges = graph.edges.clone();
        let mut values = (0..n)
            .map(|_| rng.random(b'a'..=b'z') as char)
            .collect::<Vec<_>>();
        let mut tree = TopTree::<ConcatenatePath>::from_iter(values.iter().copied());
        for &(u, v) in &edges {
            tree.link(u, v);
        }

        for _ in 0..rounds {
            match rng.random(0..if edges.is_empty() { 2 } else { 3 }) {
                0 => {
                    let u = rng.random(0..n);
                    values[u] = rng.random(b'a'..=b'z') as char;
                    tree.set(u, values[u]);
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
                tree.fold_path(u, v),
                naive_path(&adjacency, u, v)
                    .into_iter()
                    .map(|u| values[u])
                    .collect::<String>()
            );
        }
    }

    #[test]
    fn top_tree_noncommutative_path() {
        let mut rng = Xorshift::default();
        for n in 1..=14 {
            for graph in [rng.random(PathTree(n)), rng.random(StarTree(n))] {
                run_ordered_path_case(&graph, 300, &mut rng);
            }
        }
        for _ in 0..20 {
            let graph = rng.random(MixedTree(1..=14usize));
            run_ordered_path_case(&graph, 300, &mut rng);
        }
    }

    #[test]
    fn top_tree_lazy() {
        let mut rng = Xorshift::default();
        for n in 1..=14 {
            for graph in [rng.random(PathTree(n)), rng.random(StarTree(n))] {
                run_top_tree_case(&graph, 300, &mut rng);
            }
        }
        for _ in 0..20 {
            let graph = rng.random(MixedTree(1..=14usize));
            run_top_tree_case(&graph, 300, &mut rng);
        }
    }
}
