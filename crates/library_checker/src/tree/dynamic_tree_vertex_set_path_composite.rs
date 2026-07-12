use competitive::prelude::*;
use competitive::{
    algebra::{Associative, EmptyAct, LazyMapMonoid, LinearOperation, Magma, Unital},
    num::mint_basic::MInt998244353,
    tree::{PathLinkCutTree, TopTree, TopTreeSpec},
};

type MInt = MInt998244353;
type Affine = (MInt, MInt);

struct BidirectionalAffine;

impl Magma for BidirectionalAffine {
    type T = (Affine, Affine);

    fn operate(left: &Self::T, right: &Self::T) -> Self::T {
        (
            LinearOperation::operate(&left.0, &right.0),
            LinearOperation::operate(&right.1, &left.1),
        )
    }
}

impl Unital for BidirectionalAffine {
    fn unit() -> Self::T {
        let unit = LinearOperation::unit();
        (unit, unit)
    }
}

impl Associative for BidirectionalAffine {}

struct PathComposite;

impl LazyMapMonoid for PathComposite {
    type Key = Affine;
    type Agg = (Affine, Affine);
    type Act = ();
    type AggMonoid = BidirectionalAffine;
    type ActMonoid = ();
    type KeyAct = EmptyAct<Affine>;

    fn single_agg(key: &Self::Key) -> Self::Agg {
        (*key, *key)
    }

    fn toggle(value: &mut Self::Agg) {
        std::mem::swap(&mut value.0, &mut value.1);
    }

    fn act_agg(value: &Self::Agg, _action: &Self::Act) -> Option<Self::Agg> {
        Some(*value)
    }
}

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

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Relink { u: usize, v: usize, w: usize, x: usize }
        1 => Set { p: usize, cd: Affine }
        2 => Apply { u: usize, v: usize, x: MInt }
    }
}

#[verify::library_checker("dynamic_tree_vertex_set_path_composite")]
pub fn dynamic_tree_vertex_set_path_composite(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [Affine; n], edges: [(usize, usize); n - 1]);
    let mut tree = PathLinkCutTree::<PathComposite>::from_iter(ab);
    for (u, v) in edges {
        tree.link(u, v);
    }
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Relink { u, v, w, x } => {
                tree.cut(u, v);
                tree.link(w, x);
            }
            Query::Set { p, cd } => tree.set(p, cd),
            Query::Apply { u, v, x } => {
                writeln!(
                    writer,
                    "{}",
                    LinearOperation::apply(&tree.fold_path(u, v).0, &x)
                )
                .ok();
            }
        }
    }
}

#[verify::library_checker("dynamic_tree_vertex_set_path_composite")]
pub fn dynamic_tree_vertex_set_path_composite_top_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [Affine; n], edges: [(usize, usize); n - 1]);
    let mut tree = TopTree::<PathComposite>::from_iter(ab);
    for (u, v) in edges {
        tree.link(u, v);
    }
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Relink { u, v, w, x } => {
                tree.cut(u, v);
                tree.link(w, x);
            }
            Query::Set { p, cd } => tree.set(p, cd),
            Query::Apply { u, v, x } => {
                writeln!(
                    writer,
                    "{}",
                    LinearOperation::apply(&tree.fold_path(u, v).0, &x)
                )
                .ok();
            }
        }
    }
}
