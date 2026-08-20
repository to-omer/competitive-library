use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, data_structure::SegmentTree, graph::UndirectedSparseGraph,
    tree::XorLinkedRootedTree,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { u: usize, x: u64 }
        1 => Sum { u: usize }
    }
}

#[verify::library_checker("vertex_add_subtree_sum")]
pub fn vertex_add_subtree_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p));
    let tree = XorLinkedRootedTree::builder(n)
        .with_dfs_preorder()
        .build(0, edges);
    let b: Vec<_> = tree.dfs_order().iter().map(|&v| a[v]).collect();
    let mut seg = SegmentTree::<AdditiveOperation<_>>::from_vec(b);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { u, x } => seg.update(tree.dfs_index(u), x),
            Query::Sum { u } => {
                writeln!(writer, "{}", seg.fold(tree.subtree_range(u))).ok();
            }
        }
    }
}

#[verify::library_checker("vertex_add_subtree_sum")]
pub fn vertex_add_subtree_sum_hld(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], p: [usize]);
    let edges = p.take(n - 1).enumerate().map(|(i, p)| (i + 1, p)).collect();
    let tree = UndirectedSparseGraph::from_edges(n, edges);
    let hld = tree.hld(0);
    let mut b = vec![0; n];
    for (v, x) in a.into_iter().enumerate() {
        b[hld.index(v)] = x;
    }
    let mut seg = SegmentTree::<AdditiveOperation<_>>::from_vec(b);
    for _ in 0..q {
        scan!(scanner, query: Query);
        match query {
            Query::Add { u, x } => seg.update(hld.index(u), x),
            Query::Sum { u } => {
                writeln!(writer, "{}", seg.fold(hld.subtree_range(u))).ok();
            }
        }
    }
}
