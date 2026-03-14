use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{Associative, Magma, Unital},
    graph::TreeGraphScanner,
    num::{One, Zero, mint_basic::MInt998244353},
    tree::MonoidCluster,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PathPair {
    forward: Path,
    reverse: Path,
}

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

struct Dp;

impl MonoidCluster for Dp {
    type Vertex = MInt;
    type Edge = (MInt, MInt);
    type PointMonoid = PointMonoid;
    type PathMonoid = PathPairMonoid;

    fn add_vertex(point: &Point, vertex: &MInt, parent_edge: Option<&(MInt, MInt)>) -> PathPair {
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

competitive::define_enum_scan! {
    enum Query: usize {
        0 => SetVertex { v: usize, x: MInt, r: usize }
        1 => SetEdge { e: usize, a: MInt, b: MInt, r: usize }
    }
}

#[verify::library_checker("point_set_tree_path_composite_sum")]
pub fn point_set_tree_path_composite_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(
        scanner,
        n,
        q,
        value: [MInt; n],
        (graph, edges): @TreeGraphScanner::<usize, (MInt, MInt)>::new(n)
    );

    let top_tree = graph.static_top_tree(0);
    let mut dp = top_tree.dp::<Dp>(value, edges);

    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::SetVertex { v, x, r } => {
                dp.set_vertex(v, x);
                writeln!(writer, "{}", dp.fold_path(r).reverse.sum).ok();
            }
            Query::SetEdge { e, a, b, r } => {
                dp.set_edge(e, (a, b));
                writeln!(writer, "{}", dp.fold_path(r).reverse.sum).ok();
            }
        }
    }
}
