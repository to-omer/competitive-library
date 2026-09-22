use competitive::prelude::*;
use competitive::{
    algebra::{Associative, Magma, Unital},
    graph::TreeGraphScanner,
    num::{One, Zero, mint_basic::MInt998244353 as M},
    tree::MonoidCluster,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    sum: M,
    cnt: M,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Path {
    a: M,
    b: M,
    sum: M,
    cnt: M,
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
            sum: M::zero(),
            cnt: M::zero(),
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
            a: M::one(),
            b: M::zero(),
            sum: M::zero(),
            cnt: M::zero(),
        }
    }
}
impl Associative for PathMonoid {}

struct Dp;

impl MonoidCluster for Dp {
    type Vertex = M;
    type Edge = (M, M);
    type PointMonoid = PointMonoid;
    type PathMonoid = PathMonoid;

    fn add_vertex(point: &Point, vertex: &M, parent_edge: Option<&(M, M)>) -> Path {
        let cnt = point.cnt + M::one();
        let subtotal = point.sum + *vertex;
        let (a, b) = parent_edge.copied().unwrap_or((M::one(), M::zero()));
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

competitive::define_enum_scan! {
    enum Query: usize {
        0 => SetVertex { v: usize, x: M }
        1 => SetEdge { e: usize, a: M, b: M }
    }
}

#[verify::library_checker("point_set_tree_path_composite_sum_fixed_root")]
pub fn point_set_tree_path_composite_sum_fixed_root(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n,
        q,
        value: [M; n],
        (graph, edges): @TreeGraphScanner::<usize, (M, M)>::new(n));

    let top_tree = graph.static_top_tree(0);
    let mut dp = top_tree.dp::<Dp>(value, edges);

    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::SetVertex { v, x } => {
                dp.set_vertex(v, x);
                pp!(dp.fold_all().sum);
            }
            Query::SetEdge { e, a, b } => {
                dp.set_edge(e, (a, b));
                pp!(dp.fold_all().sum);
            }
        }
    }
}
