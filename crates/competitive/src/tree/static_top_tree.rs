use super::{Magma, Monoid, UndirectedSparseGraph, Unital};
use std::mem::MaybeUninit;

pub trait MonoidCluster {
    type Vertex;
    type Edge;
    type PathMonoid: Monoid;
    type PointMonoid: Monoid;

    fn add_edge(
        path: &<<Self as MonoidCluster>::PathMonoid as Magma>::T,
    ) -> <<Self as MonoidCluster>::PointMonoid as Magma>::T;
    fn add_vertex(
        point: &<<Self as MonoidCluster>::PointMonoid as Magma>::T,
        vertex: &Self::Vertex,
        parent_edge: Option<&Self::Edge>,
    ) -> <<Self as MonoidCluster>::PathMonoid as Magma>::T;
}

pub trait Cluster {
    type Vertex;
    type Edge;
    type Path: Clone;
    type Point: Clone;

    fn unit_path() -> Self::Path;
    fn unit_point() -> Self::Point;
    fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path;
    fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point;
    fn add_edge(path: &Self::Path) -> Self::Point;
    fn add_vertex(
        point: &Self::Point,
        vertex: &Self::Vertex,
        parent_edge: Option<&Self::Edge>,
    ) -> Self::Path;

    fn vertex(vertex: &Self::Vertex, parent_edge: Option<&Self::Edge>) -> Self::Path {
        Self::add_vertex(&Self::unit_point(), vertex, parent_edge)
    }
}

impl<C> Cluster for C
where
    C: MonoidCluster,
{
    type Vertex = C::Vertex;
    type Edge = C::Edge;
    type Path = <<C as MonoidCluster>::PathMonoid as Magma>::T;
    type Point = <<C as MonoidCluster>::PointMonoid as Magma>::T;

    fn unit_path() -> Self::Path {
        <C::PathMonoid as Unital>::unit()
    }

    fn unit_point() -> Self::Point {
        <C::PointMonoid as Unital>::unit()
    }

    fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path {
        <C::PathMonoid as Magma>::operate(left, right)
    }

    fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point {
        <C::PointMonoid as Magma>::operate(left, right)
    }

    fn add_edge(path: &Self::Path) -> Self::Point {
        <C as MonoidCluster>::add_edge(path)
    }

    fn add_vertex(
        point: &Self::Point,
        vertex: &Self::Vertex,
        parent_edge: Option<&Self::Edge>,
    ) -> Self::Path {
        <C as MonoidCluster>::add_vertex(point, vertex, parent_edge)
    }
}

#[derive(Clone)]
pub struct StaticTopTree {
    root: usize,
    n: usize,
    edge_child: Vec<usize>,
    parent_edge: Vec<usize>,
    compressed: Vec<InnerNode>,
    raked: Vec<InnerNode>,
    vertex_links: Vec<VertexLinks>,
    compress_roots: Vec<Option<Slot>>,
    rake_roots: Vec<Option<Slot>>,
}

#[derive(Clone)]
struct InnerNode {
    left: Slot,
    right: Slot,
    parent: usize,
}

#[derive(Clone)]
struct InnerValue<T> {
    left: T,
    right: T,
}

pub struct StaticTopTreeDp<'a, C>
where
    C: Cluster,
{
    tree: &'a StaticTopTree,
    vertices: Vec<<C as Cluster>::Vertex>,
    edges: Vec<<C as Cluster>::Edge>,
    compressed: Vec<InnerValue<<C as Cluster>::Path>>,
    raked: Vec<InnerValue<<C as Cluster>::Point>>,
    light_points: Vec<<C as Cluster>::Point>,
    all_point: <C as Cluster>::Point,
}

#[derive(Debug, Clone, Copy)]
struct VertexLinks {
    heavy_parent: usize,
    compress_parent: usize,
    rake_parent: usize,
}

#[derive(Debug)]
struct Node {
    depth: usize,
    slot: Slot,
}

#[derive(Debug, Clone, Copy)]
enum Slot {
    CompressLeaf(usize),
    CompressInner(usize),
    RakeLeaf(usize),
    RakeInner(usize),
}

struct RootedInfo {
    order: Vec<usize>,
    children_start: Vec<usize>,
    children: Vec<usize>,
    edge_child: Vec<usize>,
    parent_edge: Vec<usize>,
}

impl UndirectedSparseGraph {
    pub fn static_top_tree(&self, root: usize) -> StaticTopTree {
        StaticTopTree::new(root, self)
    }
}

impl StaticTopTree {
    pub fn new(root: usize, graph: &UndirectedSparseGraph) -> Self {
        let n = graph.vertices_size();
        assert!(n > 0);
        assert!(root < n);
        assert_eq!(graph.edges_size() + 1, n);

        let RootedInfo {
            order,
            children_start,
            children,
            edge_child,
            parent_edge,
        } = rooted_children(graph, root);
        let mut this = Self {
            root,
            n,
            edge_child,
            parent_edge,
            compressed: Vec::with_capacity(n.saturating_sub(1)),
            raked: Vec::with_capacity(n.saturating_sub(1)),
            vertex_links: vec![
                VertexLinks {
                    heavy_parent: usize::MAX,
                    compress_parent: usize::MAX,
                    rake_parent: usize::MAX,
                };
                n
            ],
            compress_roots: vec![None; n],
            rake_roots: vec![None; n],
        };

        let mut heavy_child = vec![usize::MAX; n];
        let mut mask = vec![1u64; n];
        let mut buckets: [Vec<Node>; 64] = std::array::from_fn(|_| Vec::new());

        for &u in order.iter().rev() {
            let children = &children[children_start[u]..children_start[u + 1]];
            let mut sum_rake = 0u64;
            for &v in children {
                sum_rake += bit_ceil(mask[v]) << 1;
            }
            mask[u] = bit_ceil(sum_rake);
            for &v in children {
                let child = bit_ceil(mask[v]) << 1;
                let depth = bit_ceil(sum_rake - child).trailing_zeros() as usize;
                let step = 1u64 << depth;
                let cand = ((mask[v] + step - 1) >> depth << depth) + step;
                if cand <= mask[u] {
                    mask[u] = cand;
                    heavy_child[u] = v;
                }
            }

            let mut has = 0u64;
            let mut num_light = 0usize;
            for &v in children {
                if v == heavy_child[u] {
                    continue;
                }
                num_light += 1;
                let child = bit_ceil(mask[v]) << 1;
                let depth = bit_ceil(sum_rake - child).trailing_zeros() as usize;
                this.build_compress(v, &heavy_child, &mask);
                buckets[depth].push(Node {
                    depth,
                    slot: Slot::RakeLeaf(v),
                });
                has |= 1u64 << depth;
            }
            if num_light == 0 {
                continue;
            }

            while num_light > 1 {
                let left = pop_bucket(&mut buckets, &mut has);
                let right = pop_bucket(&mut buckets, &mut has);
                let node = this.merge_rake(left, right);
                let depth = node.depth;
                buckets[depth].push(node);
                has |= 1u64 << depth;
                num_light -= 1;
            }

            let root = pop_bucket(&mut buckets, &mut has);
            this.rake_roots[u] = Some(root.slot);
            for &v0 in children {
                if v0 == heavy_child[u] {
                    continue;
                }
                let rake_parent = this.vertex_links[v0].rake_parent;
                let mut v = v0;
                while v != usize::MAX {
                    this.vertex_links[v].heavy_parent = u;
                    this.vertex_links[v].rake_parent = rake_parent;
                    v = heavy_child[v];
                }
            }
        }

        this.build_compress(root, &heavy_child, &mask);
        this
    }

    pub fn vertices_size(&self) -> usize {
        self.n
    }

    pub fn edges_size(&self) -> usize {
        self.edge_child.len()
    }

    pub fn dp<C>(
        &self,
        vertices: Vec<<C as Cluster>::Vertex>,
        edges: Vec<<C as Cluster>::Edge>,
    ) -> StaticTopTreeDp<'_, C>
    where
        C: Cluster,
    {
        StaticTopTreeDp::new(self, vertices, edges)
    }

    fn build_compress(&mut self, mut vertex: usize, heavy_child: &[usize], mask: &[u64]) -> Node {
        let start = vertex;
        let mut stack = Vec::new();
        while vertex != usize::MAX {
            stack.push(Node {
                depth: bit_ceil(mask[vertex]).trailing_zeros() as usize,
                slot: Slot::CompressLeaf(vertex),
            });
            loop {
                let len = stack.len();
                if len >= 3
                    && (stack[len - 3].depth == stack[len - 2].depth
                        || stack[len - 3].depth <= stack[len - 1].depth)
                {
                    let tail = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    let node = self.merge_compress(left, right);
                    stack.push(node);
                    stack.push(tail);
                } else if len >= 2 && stack[len - 2].depth <= stack[len - 1].depth {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(self.merge_compress(left, right));
                } else {
                    break;
                }
            }
            vertex = heavy_child[vertex];
        }
        while stack.len() > 1 {
            let right = stack.pop().unwrap();
            let left = stack.pop().unwrap();
            stack.push(self.merge_compress(left, right));
        }
        let root = stack.pop().unwrap();
        self.compress_roots[start] = Some(root.slot);
        root
    }

    fn merge_compress(&mut self, left: Node, right: Node) -> Node {
        let id = self.compressed.len();
        self.set_parent(left.slot, id << 1);
        self.set_parent(right.slot, id << 1 | 1);
        self.compressed.push(InnerNode {
            left: left.slot,
            right: right.slot,
            parent: usize::MAX,
        });
        Node {
            depth: left.depth.max(right.depth) + 1,
            slot: Slot::CompressInner(id),
        }
    }

    fn merge_rake(&mut self, left: Node, right: Node) -> Node {
        let id = self.raked.len();
        self.set_parent(left.slot, id << 1);
        self.set_parent(right.slot, id << 1 | 1);
        self.raked.push(InnerNode {
            left: left.slot,
            right: right.slot,
            parent: usize::MAX,
        });
        Node {
            depth: left.depth.max(right.depth) + 1,
            slot: Slot::RakeInner(id),
        }
    }

    fn set_parent(&mut self, slot: Slot, parent: usize) {
        match slot {
            Slot::CompressLeaf(v) => self.vertex_links[v].compress_parent = parent,
            Slot::CompressInner(i) => self.compressed[i].parent = parent,
            Slot::RakeLeaf(v) => self.vertex_links[v].rake_parent = parent,
            Slot::RakeInner(i) => self.raked[i].parent = parent,
        }
    }

    fn init_compress<C>(
        &self,
        data: &mut StaticTopTreeDataBuilder<C>,
        vertices: &[<C as Cluster>::Vertex],
        edges: &[<C as Cluster>::Edge],
        slot: Slot,
    ) -> <C as Cluster>::Path
    where
        C: Cluster,
    {
        match slot {
            Slot::CompressLeaf(vertex) => {
                let point = self.init_point(data, vertices, edges, vertex);
                C::add_vertex(
                    &point,
                    &vertices[vertex],
                    self.parent_edge_ref(edges, vertex),
                )
            }
            Slot::CompressInner(id) => {
                let node = &self.compressed[id];
                let left = self.init_compress(data, vertices, edges, node.left);
                let right = self.init_compress(data, vertices, edges, node.right);
                data.compressed[id].write(InnerValue {
                    left: left.clone(),
                    right: right.clone(),
                });
                C::compress(&left, &right)
            }
            Slot::RakeLeaf(_) | Slot::RakeInner(_) => unreachable!(),
        }
    }

    fn init_point<C>(
        &self,
        data: &mut StaticTopTreeDataBuilder<C>,
        vertices: &[<C as Cluster>::Vertex],
        edges: &[<C as Cluster>::Edge],
        vertex: usize,
    ) -> <C as Cluster>::Point
    where
        C: Cluster,
    {
        let point = if let Some(slot) = self.rake_roots[vertex] {
            self.init_rake(data, vertices, edges, slot)
        } else {
            C::unit_point()
        };
        data.light_points[vertex] = point.clone();
        point
    }

    fn init_rake<C>(
        &self,
        data: &mut StaticTopTreeDataBuilder<C>,
        vertices: &[<C as Cluster>::Vertex],
        edges: &[<C as Cluster>::Edge],
        slot: Slot,
    ) -> <C as Cluster>::Point
    where
        C: Cluster,
    {
        match slot {
            Slot::RakeLeaf(vertex) => {
                let path = self.init_compress(
                    data,
                    vertices,
                    edges,
                    self.compress_roots[vertex].expect("light child path must exist"),
                );
                C::add_edge(&path)
            }
            Slot::RakeInner(id) => {
                let node = &self.raked[id];
                let left = self.init_rake(data, vertices, edges, node.left);
                let right = self.init_rake(data, vertices, edges, node.right);
                data.raked[id].write(InnerValue {
                    left: left.clone(),
                    right: right.clone(),
                });
                C::rake(&left, &right)
            }
            Slot::CompressLeaf(_) | Slot::CompressInner(_) => unreachable!(),
        }
    }

    fn parent_edge_ref<'a, T>(&self, edges: &'a [T], vertex: usize) -> Option<&'a T> {
        let edge = self.parent_edge[vertex];
        if edge == usize::MAX {
            None
        } else {
            Some(&edges[edge])
        }
    }
}

impl<'a, C> StaticTopTreeDp<'a, C>
where
    C: Cluster,
{
    pub fn new(
        tree: &'a StaticTopTree,
        vertices: Vec<<C as Cluster>::Vertex>,
        edges: Vec<<C as Cluster>::Edge>,
    ) -> Self {
        assert_eq!(vertices.len(), tree.vertices_size());
        assert_eq!(edges.len(), tree.edges_size());

        let mut data: StaticTopTreeDataBuilder<C> = StaticTopTreeDataBuilder::new(tree);
        let path = tree.init_compress(
            &mut data,
            &vertices,
            &edges,
            tree.compress_roots[tree.root].expect("root compress tree must exist"),
        );
        let all_point = C::add_edge(&path);
        Self {
            tree,
            vertices,
            edges,
            compressed: unsafe { assume_init_vec(data.compressed) },
            raked: unsafe { assume_init_vec(data.raked) },
            light_points: data.light_points,
            all_point,
        }
    }

    pub fn set_vertex(&mut self, vertex: usize, value: <C as Cluster>::Vertex) {
        assert!(vertex < self.vertices.len());
        self.vertices[vertex] = value;
        self.update_from_vertex(vertex);
    }

    pub fn set_edge(&mut self, edge: usize, value: <C as Cluster>::Edge) {
        assert!(edge < self.edges.len());
        self.edges[edge] = value;
        self.update_from_vertex(self.tree.edge_child[edge]);
    }

    pub fn fold_all(&self) -> &<C as Cluster>::Point {
        &self.all_point
    }

    pub fn fold_path(&self, mut vertex: usize) -> <C as Cluster>::Path {
        assert!(vertex < self.tree.n);
        let mut path = C::unit_path();
        let mut point = self.light_points[vertex].clone();
        loop {
            let links = self.tree.vertex_links[vertex];
            let mut left = C::unit_path();
            let mut right = C::unit_path();
            let mut compress_parent = links.compress_parent;
            while compress_parent != usize::MAX {
                let inner = &self.compressed[compress_parent / 2];
                if compress_parent & 1 == 0 {
                    right = C::compress(&right, &inner.right);
                } else {
                    left = C::compress(&inner.left, &left);
                }
                compress_parent = self.tree.compressed[compress_parent / 2].parent;
            }
            let right_point = C::add_edge(&right);
            point = C::rake(&point, &right_point);
            let mid = C::add_vertex(
                &point,
                &self.vertices[vertex],
                self.tree.parent_edge_ref(&self.edges, vertex),
            );
            let mid = C::compress(&mid, &path);
            path = C::compress(&left, &mid);
            if links.heavy_parent == usize::MAX {
                return path;
            }

            point = C::unit_point();
            let mut rake_parent = links.rake_parent;
            while rake_parent != usize::MAX {
                let inner = &self.raked[rake_parent / 2];
                if rake_parent & 1 == 0 {
                    point = C::rake(&point, &inner.right);
                } else {
                    point = C::rake(&inner.left, &point);
                }
                rake_parent = self.tree.raked[rake_parent / 2].parent;
            }
            vertex = links.heavy_parent;
        }
    }

    fn update_from_vertex(&mut self, mut vertex: usize) {
        assert!(vertex < self.tree.n);
        while vertex != usize::MAX {
            let links = self.tree.vertex_links[vertex];
            let base = C::add_vertex(
                &self.light_points[vertex],
                &self.vertices[vertex],
                self.tree.parent_edge_ref(&self.edges, vertex),
            );
            let path = self.update_compress(links.compress_parent, base);
            let point = C::add_edge(&path);
            let point = self.update_rake(links.rake_parent, point);
            if links.heavy_parent == usize::MAX {
                self.all_point = point;
            } else {
                self.light_points[links.heavy_parent] = point;
            }
            vertex = links.heavy_parent;
        }
    }

    fn update_compress(
        &mut self,
        mut id: usize,
        mut path: <C as Cluster>::Path,
    ) -> <C as Cluster>::Path {
        while id != usize::MAX {
            let inner = &mut self.compressed[id / 2];
            if id & 1 == 0 {
                inner.left = path;
            } else {
                inner.right = path;
            }
            path = C::compress(&inner.left, &inner.right);
            id = self.tree.compressed[id / 2].parent;
        }
        path
    }

    fn update_rake(
        &mut self,
        mut id: usize,
        mut point: <C as Cluster>::Point,
    ) -> <C as Cluster>::Point {
        while id != usize::MAX {
            let inner = &mut self.raked[id / 2];
            if id & 1 == 0 {
                inner.left = point;
            } else {
                inner.right = point;
            }
            point = C::rake(&inner.left, &inner.right);
            id = self.tree.raked[id / 2].parent;
        }
        point
    }
}

struct StaticTopTreeDataBuilder<C>
where
    C: Cluster,
{
    compressed: Vec<MaybeUninit<InnerValue<<C as Cluster>::Path>>>,
    raked: Vec<MaybeUninit<InnerValue<<C as Cluster>::Point>>>,
    light_points: Vec<<C as Cluster>::Point>,
}

impl<C> StaticTopTreeDataBuilder<C>
where
    C: Cluster,
{
    fn new(tree: &StaticTopTree) -> Self {
        let mut compressed = Vec::with_capacity(tree.compressed.len());
        compressed.resize_with(tree.compressed.len(), MaybeUninit::uninit);
        let mut raked = Vec::with_capacity(tree.raked.len());
        raked.resize_with(tree.raked.len(), MaybeUninit::uninit);
        Self {
            compressed,
            raked,
            light_points: vec![C::unit_point(); tree.n],
        }
    }
}

unsafe fn assume_init_vec<T>(mut vec: Vec<MaybeUninit<T>>) -> Vec<T> {
    let len = vec.len();
    let cap = vec.capacity();
    let ptr = vec.as_mut_ptr() as *mut T;
    std::mem::forget(vec);
    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}

fn bit_ceil(x: u64) -> u64 {
    if x <= 1 { 1 } else { x.next_power_of_two() }
}

fn rooted_children(graph: &UndirectedSparseGraph, root: usize) -> RootedInfo {
    let n = graph.vertices_size();
    let mut order = Vec::with_capacity(n);
    let mut parent = vec![usize::MAX; n];
    let mut parent_edge = vec![usize::MAX; n];
    let mut edge_child = vec![0; graph.edges_size()];
    order.push(root);
    parent[root] = usize::MAX;
    for i in 0..n {
        let u = order[i];
        for a in graph.adjacencies(u) {
            if a.to == parent[u] {
                continue;
            }
            parent[a.to] = u;
            parent_edge[a.to] = a.id;
            edge_child[a.id] = a.to;
            order.push(a.to);
        }
    }
    let mut children_start = vec![0usize; n + 1];
    for &v in order.iter().skip(1) {
        children_start[parent[v] + 1] += 1;
    }
    for i in 1..=n {
        children_start[i] += children_start[i - 1];
    }
    let mut children = vec![0; n.saturating_sub(1)];
    let mut child_pos = children_start.clone();
    for &v in order.iter().skip(1) {
        let pos = child_pos[parent[v]];
        children[pos] = v;
        child_pos[parent[v]] += 1;
    }
    RootedInfo {
        order,
        children_start,
        children,
        edge_child,
        parent_edge,
    }
}

fn pop_bucket(buckets: &mut [Vec<Node>; 64], has: &mut u64) -> Node {
    let depth = has.trailing_zeros() as usize;
    let node = buckets[depth].pop().unwrap();
    if buckets[depth].is_empty() {
        *has &= !(1u64 << depth);
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{Associative, Magma, Unital},
        graph::UndirectedSparseGraph,
        num::{One, Zero, mint_basic::MInt998244353},
        tools::Xorshift,
        tree::{PathTree, PruferSequence, StarTree},
    };

    type MInt = MInt998244353;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Point {
        sum: MInt,
        cnt: MInt,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Path {
        a: MInt,
        b: MInt,
        sum: MInt,
        cnt: MInt,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct PathPair {
        forward: Path,
        reverse: Path,
    }

    struct PointMonoid;
    impl Magma for PointMonoid {
        type T = Point;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            Point {
                sum: x.sum + y.sum,
                cnt: x.cnt + y.cnt,
            }
        }
    }
    impl Unital for PointMonoid {
        fn unit() -> Self::T {
            Point {
                sum: MInt::zero(),
                cnt: MInt::zero(),
            }
        }
    }
    impl Associative for PointMonoid {}

    struct PathMonoid;
    impl Magma for PathMonoid {
        type T = Path;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            Path {
                a: x.a * y.a,
                b: x.b + x.a * y.b,
                sum: x.sum + x.a * y.sum + x.b * y.cnt,
                cnt: x.cnt + y.cnt,
            }
        }
    }
    impl Unital for PathMonoid {
        fn unit() -> Self::T {
            Path {
                a: MInt::one(),
                b: MInt::zero(),
                sum: MInt::zero(),
                cnt: MInt::zero(),
            }
        }
    }
    impl Associative for PathMonoid {}

    struct PathPairMonoid;
    impl Magma for PathPairMonoid {
        type T = PathPair;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            PathPair {
                forward: PathMonoid::operate(&x.forward, &y.forward),
                reverse: PathMonoid::operate(&y.reverse, &x.reverse),
            }
        }
    }
    impl Unital for PathPairMonoid {
        fn unit() -> Self::T {
            PathPair {
                forward: PathMonoid::unit(),
                reverse: PathMonoid::unit(),
            }
        }
    }
    impl Associative for PathPairMonoid {}

    struct FixedCluster;
    impl MonoidCluster for FixedCluster {
        type Vertex = MInt;
        type Edge = (MInt, MInt);
        type PointMonoid = PointMonoid;
        type PathMonoid = PathMonoid;

        fn add_vertex(point: &Point, vertex: &MInt, parent_edge: Option<&(MInt, MInt)>) -> Path {
            let cnt = point.cnt + MInt::one();
            let subtotal = point.sum + *vertex;
            let (a, b) = parent_edge.copied().unwrap_or((MInt::one(), MInt::zero()));
            Path {
                a,
                b,
                sum: a * subtotal + b * cnt,
                cnt,
            }
        }

        fn add_edge(path: &Path) -> Point {
            Point {
                sum: path.sum,
                cnt: path.cnt,
            }
        }
    }

    struct RerootCluster;
    impl MonoidCluster for RerootCluster {
        type Vertex = MInt;
        type Edge = (MInt, MInt);
        type PointMonoid = PointMonoid;
        type PathMonoid = PathPairMonoid;

        fn add_vertex(
            point: &Point,
            vertex: &MInt,
            parent_edge: Option<&(MInt, MInt)>,
        ) -> PathPair {
            let cnt = point.cnt + MInt::one();
            let subtotal = point.sum + *vertex;
            let (a, b) = parent_edge.copied().unwrap_or((MInt::one(), MInt::zero()));
            PathPair {
                forward: Path {
                    a,
                    b,
                    sum: a * subtotal + b * cnt,
                    cnt,
                },
                reverse: Path {
                    a,
                    b,
                    sum: subtotal,
                    cnt,
                },
            }
        }

        fn add_edge(path: &PathPair) -> Point {
            Point {
                sum: path.forward.sum,
                cnt: path.forward.cnt,
            }
        }
    }

    fn naive_rooted(
        graph: &UndirectedSparseGraph,
        vertices: &[MInt],
        edges: &[(MInt, MInt)],
        root: usize,
    ) -> Point {
        fn dfs(
            graph: &UndirectedSparseGraph,
            vertices: &[MInt],
            edges: &[(MInt, MInt)],
            u: usize,
            p: usize,
            in_edge: Option<usize>,
        ) -> Point {
            let mut point = PointMonoid::unit();
            for a in graph.adjacencies(u) {
                if a.to != p {
                    point = PointMonoid::operate(
                        &point,
                        &dfs(graph, vertices, edges, a.to, u, Some(a.id)),
                    );
                }
            }
            let cnt = point.cnt + MInt::one();
            let subtotal = point.sum + vertices[u];
            let (a, b) = in_edge
                .map(|eid| edges[eid])
                .unwrap_or((MInt::one(), MInt::zero()));
            Point {
                sum: a * subtotal + b * cnt,
                cnt,
            }
        }
        dfs(graph, vertices, edges, root, usize::MAX, None)
    }

    fn balanced_tree(n: usize) -> UndirectedSparseGraph {
        let edges = (1..n).map(|v| ((v - 1) / 2, v)).collect::<Vec<_>>();
        UndirectedSparseGraph::from_edges(n, edges)
    }

    fn gen_weights(rng: &mut Xorshift, n: usize, m: usize) -> (Vec<MInt>, Vec<(MInt, MInt)>) {
        let vertices = (0..n)
            .map(|_| MInt::from(rng.random(0u32..10)))
            .collect::<Vec<_>>();
        let edges = (0..m)
            .map(|_| {
                (
                    MInt::from(rng.random(0u32..10)),
                    MInt::from(rng.random(0u32..10)),
                )
            })
            .collect::<Vec<_>>();
        (vertices, edges)
    }

    fn run_fixed_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let m = graph.edges_size();
        let (mut vertices, mut edges) = gen_weights(rng, n, m);
        let tree = graph.static_top_tree(0);
        let mut dp = tree.dp::<FixedCluster>(vertices.clone(), edges.clone());
        assert_eq!(*dp.fold_all(), naive_rooted(graph, &vertices, &edges, 0));

        for _ in 0..rounds {
            if rng.random(0u32..2) == 0 {
                let v = rng.random(0..n);
                let x = MInt::from(rng.random(0u32..20));
                vertices[v] = x;
                dp.set_vertex(v, x);
            } else if m > 0 {
                let eid = rng.random(0..m);
                let edge = (
                    MInt::from(rng.random(0u32..20)),
                    MInt::from(rng.random(0u32..20)),
                );
                edges[eid] = edge;
                dp.set_edge(eid, edge);
            }
            assert_eq!(*dp.fold_all(), naive_rooted(graph, &vertices, &edges, 0));
        }
    }

    fn run_reroot_case(graph: &UndirectedSparseGraph, rounds: usize, rng: &mut Xorshift) {
        let n = graph.vertices_size();
        let m = graph.edges_size();
        let (mut vertices, mut edges) = gen_weights(rng, n, m);
        let tree = graph.static_top_tree(0);
        let mut dp = tree.dp::<RerootCluster>(vertices.clone(), edges.clone());
        assert_eq!(
            dp.fold_all().sum,
            naive_rooted(graph, &vertices, &edges, 0).sum
        );

        for _ in 0..rounds {
            if rng.random(0u32..2) == 0 {
                let v = rng.random(0..n);
                let x = MInt::from(rng.random(0u32..20));
                vertices[v] = x;
                dp.set_vertex(v, x);
            } else if m > 0 {
                let eid = rng.random(0..m);
                let edge = (
                    MInt::from(rng.random(0u32..20)),
                    MInt::from(rng.random(0u32..20)),
                );
                edges[eid] = edge;
                dp.set_edge(eid, edge);
            } else {
                let v = rng.random(0..n);
                let x = MInt::from(rng.random(0u32..20));
                vertices[v] = x;
                dp.set_vertex(v, x);
            }
            for root in 0..n {
                let got = dp.fold_path(root).reverse.sum;
                let want = naive_rooted(graph, &vertices, &edges, root).sum;
                assert_eq!(got, want, "root={root}");
            }
        }
    }

    #[test]
    fn static_top_tree_fixed_random() {
        let mut rng = Xorshift::default();
        for _ in 0..30 {
            let graph = rng.random(PruferSequence(2..=14usize));
            run_fixed_case(&graph, 40, &mut rng);
        }
    }

    #[test]
    fn static_top_tree_reroot_random() {
        let mut rng = Xorshift::default();
        for _ in 0..20 {
            let graph = rng.random(PruferSequence(2..=12usize));
            run_reroot_case(&graph, 30, &mut rng);
        }
    }

    #[test]
    fn static_top_tree_shapes() {
        let mut rng = Xorshift::default();
        for graph in [
            UndirectedSparseGraph::from_edges(1, vec![]),
            rng.random(PathTree(2..=16usize)),
            rng.random(StarTree(2..=16usize)),
            balanced_tree(15),
        ] {
            run_fixed_case(&graph, 30, &mut rng);
            run_reroot_case(&graph, 20, &mut rng);
        }
    }
}
