use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { v: usize, l: usize, r: usize, x: i64 }
        1 => Get { v: usize }
    }
}

#[verify::library_checker("vertex_get_range_contour_add_on_tree")]
pub fn vertex_get_range_contour_add_on_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, mut a: [i64; n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let cq = graph.contour_query_range();
    let mut bits: Vec<BinaryIndexedTree<AdditiveOperation<_>>> = cq
        .component_sizes()
        .map(|n| BinaryIndexedTree::new(n + 1))
        .collect();

    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { v, l, r, x } => {
                cq.for_each_contour_range(v, l, r, |c, start, end| {
                    bits[c].update(start, x);
                    bits[c].update(end, -x);
                });
                if l == 0 && 0 < r {
                    a[v] += x;
                }
            }
            Query::Get { v } => {
                let mut ans = a[v];
                cq.for_each_index(v, |c, i| ans += bits[c].accumulate(i));
                pp!(ans);
            }
        }
    }
}
