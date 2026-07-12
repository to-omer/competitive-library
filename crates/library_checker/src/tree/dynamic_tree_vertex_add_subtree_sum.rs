use competitive::prelude::*;
use competitive::tree::{
    LinkCutTree, LinkCutTreeSpec, LinkCutTreeSubtreeFold, TopTree, TopTreeSpec,
};

struct SubtreeSum;

struct SubtreeSumData {
    value: u64,
    virtual_sum: u64,
    sum: u64,
}

impl LinkCutTreeSpec for SubtreeSum {
    type Value = u64;
    type Data = SubtreeSumData;

    fn new(value: Self::Value) -> Self::Data {
        SubtreeSumData {
            value,
            virtual_sum: 0,
            sum: value,
        }
    }

    fn value(data: &Self::Data) -> &Self::Value {
        &data.value
    }

    fn value_mut(data: &mut Self::Data) -> &mut Self::Value {
        &mut data.value
    }

    fn bottom_up(data: &mut Self::Data, children: [Option<&Self::Data>; 2]) {
        data.sum = data.value
            + data.virtual_sum
            + children
                .into_iter()
                .flatten()
                .map(|child| child.sum)
                .sum::<u64>();
    }

    fn reverse(_data: &mut Self::Data) {}

    fn attach_virtual(parent: &mut Self::Data, child: &mut Self::Data) {
        parent.virtual_sum += child.sum;
    }

    fn detach_virtual(parent: &mut Self::Data, child: &mut Self::Data) {
        parent.virtual_sum -= child.sum;
    }
}

impl LinkCutTreeSubtreeFold for SubtreeSum {
    type Subtree = u64;

    fn fold_subtree(data: &Self::Data) -> Self::Subtree {
        data.sum
    }
}

impl TopTreeSpec for SubtreeSum {
    type Info = u64;
    type Point = (u64, u64);
    type Path = (u64, u64, u64);

    fn vertex(info: &Self::Info) -> Self::Path {
        (*info, 1, *info)
    }

    fn add_vertex(point: &Self::Point, info: &Self::Info) -> Self::Path {
        (point.0 + *info, point.1 + 1, *info)
    }

    fn add_edge(path: &Self::Path) -> Self::Point {
        (path.0, path.1)
    }

    fn rake(left: &Self::Point, right: &Self::Point) -> Self::Point {
        (left.0 + right.0, left.1 + right.1)
    }

    fn compress(left: &Self::Path, right: &Self::Path) -> Self::Path {
        (left.0 + right.0, left.1 + right.1, left.2 + right.2)
    }

    fn reverse(_path: &mut Self::Path) {}
}

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Relink { u: usize, v: usize, w: usize, x: usize }
        1 => Add { p: usize, x: u64 }
        2 => Sum { v: usize, p: usize }
    }
}

#[verify::library_checker("dynamic_tree_vertex_add_subtree_sum")]
pub fn dynamic_tree_vertex_add_subtree_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], edges: [(usize, usize); n - 1]);
    let mut tree = LinkCutTree::<SubtreeSum>::from_iter(a);
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
            Query::Sum { v, p } => {
                writeln!(writer, "{}", tree.fold_subtree(v, p)).ok();
            }
        }
    }
}

#[verify::library_checker("dynamic_tree_vertex_add_subtree_sum")]
pub fn dynamic_tree_vertex_add_subtree_sum_top_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], edges: [(usize, usize); n - 1]);
    let mut tree = TopTree::<SubtreeSum>::from_iter(a);
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
            Query::Sum { v, p } => {
                writeln!(writer, "{}", tree.fold_subtree(v, p).0).ok();
            }
        }
    }
}
