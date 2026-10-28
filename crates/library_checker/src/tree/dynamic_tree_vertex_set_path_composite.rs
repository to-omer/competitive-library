use competitive::prelude::*;
use competitive::{
    algebra::{
        Associative, EmptyAct, LazyMapAct, LazyMapAgg, LazyMapMonoid, LazyMapMonoidWithKeyAct,
        LinearOperation, Magma, MonoidAct, Unital,
    },
    num::mint_basic::MInt998244353 as M,
    tree::{PathLinkCutTree, TopTree, TopTreeSpec},
};

#[codesnip::entry("Affine", include("MInt"))]
type Affine = (M, M);

#[codesnip::entry("BidirectionalAffine", include("Affine", "LinearOperation"))]
struct BidirectionalAffine;

#[codesnip::entry("BidirectionalAffine")]
impl Magma for BidirectionalAffine {
    type T = (Affine, Affine);

    fn operate(left: &Self::T, right: &Self::T) -> Self::T {
        (
            LinearOperation::operate(&left.0, &right.0),
            LinearOperation::operate(&right.1, &left.1),
        )
    }
}

#[codesnip::entry("BidirectionalAffineMonoid", include("BidirectionalAffine"))]
impl Unital for BidirectionalAffine {
    fn unit() -> Self::T {
        let unit = LinearOperation::unit();
        (unit, unit)
    }
}

#[codesnip::entry("BidirectionalAffineMonoid")]
impl Associative for BidirectionalAffine {}

#[codesnip::entry("PathComposite")]
struct PathComposite;

#[codesnip::entry(
    "PathCompositeLinkCutTree",
    include("PathComposite", "BidirectionalAffineMonoid", "LinkCutTree")
)]
impl LazyMapMonoid for PathComposite {
    type Key = Affine;
    type AggMonoid = BidirectionalAffine;
    type ActMonoid = ();

    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        (*key, *key)
    }

    fn toggle(value: &mut LazyMapAgg<Self>) {
        std::mem::swap(&mut value.0, &mut value.1);
    }

    fn act_agg(value: &LazyMapAgg<Self>, _action: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(*value)
    }
}
#[codesnip::entry("PathCompositeLinkCutTree")]
impl LazyMapMonoidWithKeyAct for PathComposite {
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        EmptyAct::act(key, act)
    }
}

#[codesnip::entry(
    "PathCompositeTopTree",
    include("PathComposite", "BidirectionalAffine", "TopTree")
)]
impl TopTreeSpec for PathComposite {
    type Info = Affine;
    type Point = ();
    type Path = (Affine, Affine);

    fn vertex(info: &Self::Info) -> Self::Path {
        (*info, *info)
    }

    fn add_vertex(_point: &Self::Point, info: &Self::Info) -> Self::Path {
        (*info, *info)
    }

    fn add_edge(_path: &Self::Path) -> Self::Point {}

    fn rake(_left: &Self::Point, _right: &Self::Point) -> Self::Point {}

    fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path {
        BidirectionalAffine::operate(left, right)
    }

    fn reverse(path: &mut Self::Path) {
        std::mem::swap(&mut path.0, &mut path.1);
    }
}

#[codesnip::entry("Query", include("Affine"))]
competitive::define_enum_scan! {
    enum Query: usize {
        0 => Relink { u: usize, v: usize, w: usize, x: usize }
        1 => Set { p: usize, cd: Affine }
        2 => Apply { u: usize, v: usize, x: M }
    }
}

#[verify::library_checker(
    "dynamic_tree_vertex_set_path_composite",
    include("PathCompositeLinkCutTree", "Query")
)]
pub fn dynamic_tree_vertex_set_path_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, ab: [Affine; n], edges: [(usize, usize); n - 1]);
    let mut tree = PathLinkCutTree::<PathComposite>::from_edges(ab, &edges);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Relink { u, v, w, x } => {
                tree.cut(u, v);
                tree.link(w, x);
            }
            Query::Set { p, cd } => tree.set(p, cd),
            Query::Apply { u, v, x } => {
                pp!(LinearOperation::apply(&tree.fold_path(u, v).0, &x));
            }
        }
    }
}

#[verify::library_checker(
    "dynamic_tree_vertex_set_path_composite",
    include("PathCompositeTopTree", "Query")
)]
pub fn dynamic_tree_vertex_set_path_composite_top_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, ab: [Affine; n], edges: [(usize, usize); n - 1]);
    let mut tree = TopTree::<PathComposite>::from_edges(ab, &edges);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Relink { u, v, w, x } => {
                tree.cut(u, v);
                tree.link(w, x);
            }
            Query::Set { p, cd } => tree.set(p, cd),
            Query::Apply { u, v, x } => {
                pp!(LinearOperation::apply(&tree.fold_path(u, v).0, &x));
            }
        }
    }
}
