use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation,
    tree::{
        LinkCutTree, LinkCutTreeSpec, LinkCutTreeSubtreeFold, LinkCutTreeSubtreeUpdate, TopTree,
        TopTreeAction, TopTreeSpec,
    },
};
use std::mem::replace;

struct SubtreeSum;

struct SubtreeSumData {
    value: u64,
    virtual_sum: u64,
    virtual_size: u64,
    sum: u64,
    size: u64,
    lazy: u64,
    virtual_lazy: u64,
    path_parent_lazy: u64,
}

impl SubtreeSum {
    fn apply(data: &mut SubtreeSumData, action: u64) {
        data.value += action;
        data.virtual_sum += data.virtual_size * action;
        data.sum += data.size * action;
        data.lazy += action;
        data.virtual_lazy += action;
    }
}

impl LinkCutTreeSpec for SubtreeSum {
    type Value = u64;
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
                .sum::<u64>();
        data.size = 1
            + data.virtual_size
            + children
                .into_iter()
                .flatten()
                .map(|child| child.size)
                .sum::<u64>();
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
    type Subtree = u64;

    fn fold_subtree(data: &Self::Data) -> Self::Subtree {
        data.sum
    }
}

impl LinkCutTreeSubtreeUpdate for SubtreeSum {
    type SubtreeAction = u64;

    fn update_subtree(data: &mut Self::Data, action: &Self::SubtreeAction) {
        Self::apply(data, *action);
    }
}

struct TopTreeSubtreeSum;

impl TopTreeSpec for TopTreeSubtreeSum {
    type Info = u64;
    type Point = (u64, u64);
    type Path = (u64, u64, u64);

    fn vertex(info: &Self::Info) -> Self::Path {
        (*info, 1, 1)
    }

    fn add_vertex(point: &Self::Point, info: &Self::Info) -> Self::Path {
        (point.0 + *info, point.1 + 1, 1)
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

struct AddAction;

impl TopTreeAction<TopTreeSubtreeSum> for AddAction {
    type Action = u64;
    type ActionMonoid = AdditiveOperation<u64>;

    fn act_info(info: &mut u64, action: &Self::Action) {
        *info += *action;
    }

    fn act_point(point: &mut (u64, u64), action: &Self::Action) {
        point.0 += point.1 * *action;
    }

    fn act_path(path: &mut (u64, u64, u64), action: &Self::Action) {
        path.0 += path.2 * *action;
    }

    fn act_path_light(path: &mut (u64, u64, u64), action: &Self::Action) {
        path.0 += (path.1 - path.2) * *action;
    }
}

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Relink { u: usize, v: usize, w: usize, x: usize }
        1 => Add { v: usize, p: usize, x: u64 }
        2 => Sum { v: usize, p: usize }
    }
}

#[verify::library_checker("dynamic_tree_subtree_add_subtree_sum")]
pub fn dynamic_tree_subtree_add_subtree_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], edges: [(usize, usize); n - 1]);
    let mut tree = LinkCutTree::<SubtreeSum>::from_edges(a, &edges);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Relink { u, v, w, x } => {
                tree.cut(u, v);
                tree.link(w, x);
            }
            Query::Add { v, p, x } => tree.update_subtree(v, p, &x),
            Query::Sum { v, p } => {
                writeln!(writer, "{}", tree.fold_subtree(v, p)).ok();
            }
        }
    }
}

#[verify::library_checker("dynamic_tree_subtree_add_subtree_sum")]
pub fn dynamic_tree_subtree_add_subtree_sum_top_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], edges: [(usize, usize); n - 1]);
    let mut tree = TopTree::<TopTreeSubtreeSum, AddAction>::from_edges(a, &edges);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Relink { u, v, w, x } => {
                tree.cut(u, v);
                tree.link(w, x);
            }
            Query::Add { v, p, x } => tree.update_subtree(v, p, &x),
            Query::Sum { v, p } => {
                writeln!(writer, "{}", tree.fold_subtree(v, p).0).ok();
            }
        }
    }
}
