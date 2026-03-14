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

struct Dp;

impl MonoidCluster for Dp {
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

competitive::define_enum_scan! {
    enum Query: usize {
        0 => SetVertex { v: usize, x: MInt }
        1 => SetEdge { e: usize, a: MInt, b: MInt }
    }
}

#[verify::library_checker("point_set_tree_path_composite_sum_fixed_root")]
pub fn point_set_tree_path_composite_sum_fixed_root(reader: impl Read, mut writer: impl Write) {
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
            Query::SetVertex { v, x } => {
                dp.set_vertex(v, x);
                writeln!(writer, "{}", dp.fold_all().sum).ok();
            }
            Query::SetEdge { e, a, b } => {
                dp.set_edge(e, (a, b));
                writeln!(writer, "{}", dp.fold_all().sum).ok();
            }
        }
    }
}
