use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeAdd,
    tree::{PathLinkCutTree, TopTree, TopTreeSpec},
};

struct SumTopTree;

impl TopTreeSpec for SumTopTree {
    type Info = i64;
    type Point = i64;
    type Path = (i64, i64);

    fn vertex(info: &Self::Info) -> Self::Path {
        (*info, *info)
    }

    fn add_vertex(point: &Self::Point, info: &Self::Info) -> Self::Path {
        (*point + *info, *info)
    }

    fn add_edge(path: &Self::Path) -> Self::Point {
        path.0
    }

    fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point {
        *left + *right
    }

    fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path {
        (left.0 + right.0, left.1 + right.1)
    }

    fn reverse(_path: &mut Self::Path) {}
}

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Relink { u: usize, v: usize, w: usize, x: usize }
        1 => Add { p: usize, x: i64 }
        2 => Sum { u: usize, v: usize }
    }
}

#[verify::library_checker("dynamic_tree_vertex_add_path_sum")]
pub fn dynamic_tree_vertex_add_path_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n], edges: [(usize, usize); n - 1]);
    let mut tree = PathLinkCutTree::<RangeSumRangeAdd<i64>>::from_iter(a);
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
            Query::Add { p, x } => {
                let value = *tree.get(p) + x;
                tree.set(p, value);
            }
            Query::Sum { u, v } => {
                writeln!(writer, "{}", tree.fold_path(u, v).0).ok();
            }
        }
    }
}

#[verify::library_checker("dynamic_tree_vertex_add_path_sum")]
pub fn dynamic_tree_vertex_add_path_sum_top_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n], edges: [(usize, usize); n - 1]);
    let mut tree = TopTree::<SumTopTree>::from_iter(a);
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
            Query::Add { p, x } => {
                let value = *tree.get(p) + x;
                tree.set(p, value);
            }
            Query::Sum { u, v } => {
                writeln!(writer, "{}", tree.fold_path(u, v).1).ok();
            }
        }
    }
}
