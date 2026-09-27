use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation,
    data_structure::BinaryIndexedTree,
    tree::{LowestCommonAncestor, XorLinkedRootedTreeScanner},
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { u: usize, v: usize }
    }
}

#[verify::library_checker("vertex_add_path_sum")]
pub fn vertex_add_path_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, mut a: [i64; n],
        (tree, _): @XorLinkedRootedTreeScanner::<usize, ()>::new(n, 0)
            .with_parent().with_dfs_preorder());
    let lca = LowestCommonAncestor::from_dfs_preorder(tree.parents(), tree.dfs_order());
    let mut values = vec![0; n + 1];
    for (u, &x) in a.iter().enumerate() {
        let range = tree.subtree_range(u);
        values[range.start] += x;
        values[range.end] -= x;
    }
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::from_slice(&values);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { p, x } => {
                a[p] += x;
                let range = tree.subtree_range(p);
                bit.update(range.start, x);
                bit.update(range.end, -x);
            }
            Query::Sum { u, v } => {
                let p = lca.lca(u, v);
                pp!(
                    a[p] + bit.accumulate(tree.dfs_index(u)) + bit.accumulate(tree.dfs_index(v))
                        - 2 * bit.accumulate(tree.dfs_index(p))
                );
            }
        }
    }
}
