use competitive::{
    graph::UndirectedSparseGraph,
    tools::Xorshift,
    tree::{MixedTree, XorLinkedRootedTree},
};
use criterion::{BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;

pub fn bench_xor_linked_tree(c: &mut Criterion) {
    const SIZES: [usize; 3] = [1_000, 10_000, 100_000];
    let mut rng = Xorshift::default();
    let mut group = c.benchmark_group("xor_linked_tree");

    for n in SIZES {
        let edges = rng.random(MixedTree(n)).edges;
        group.bench_function(BenchmarkId::new("xor_parent", n), |b| {
            b.iter(|| {
                let tree = XorLinkedRootedTree::builder(n)
                    .with_parent()
                    .build(0, edges.iter().copied());
                black_box(tree.parent(n - 1));
            })
        });
        group.bench_function(BenchmarkId::new("xor_bottom_up_order", n), |b| {
            b.iter(|| {
                let tree = XorLinkedRootedTree::builder(n)
                    .with_xor_bottom_up_order()
                    .build(0, edges.iter().copied());
                black_box(tree.xor_top_down_order().next());
            })
        });
        group.bench_function(BenchmarkId::new("xor_dfs_preorder", n), |b| {
            b.iter(|| {
                let tree = XorLinkedRootedTree::builder(n)
                    .with_dfs_preorder()
                    .build(0, edges.iter().copied());
                black_box((tree.dfs_order().len(), tree.subtree_size(0)));
            })
        });
        group.bench_function(BenchmarkId::new("xor_eindexed", n), |b| {
            b.iter(|| {
                let tree = XorLinkedRootedTree::builder(n)
                    .with_parent()
                    .with_eindexed()
                    .with_parent_edge()
                    .with_edge_child()
                    .build(0, edges.iter().copied());
                black_box((tree.parent(n - 1), tree.parent_edge(n - 1)));
            })
        });
        group.bench_function(BenchmarkId::new("sparse_graph_tree_order", n), |b| {
            b.iter_batched(
                || edges.clone(),
                |edges| {
                    let graph = UndirectedSparseGraph::from_edges(n, edges);
                    let (order, parent) = graph.tree_order(0);
                    black_box((order.len(), parent[n - 1]));
                },
                BatchSize::LargeInput,
            )
        });
    }

    group.finish();
}
