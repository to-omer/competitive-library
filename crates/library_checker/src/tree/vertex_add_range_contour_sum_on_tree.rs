use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { v: usize, l: usize, r: usize }
    }
}

#[verify::library_checker("vertex_add_range_contour_sum_on_tree")]
pub fn vertex_add_range_contour_sum_on_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, mut a: [i64; n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let cq = graph.contour_query_range();
    let mut raw: Vec<_> = cq.component_sizes().map(|n| vec![0; n]).collect();
    for (v, &x) in a.iter().enumerate() {
        cq.for_each_index(v, |c, i| raw[c][i] += x);
    }
    let mut bits: Vec<BinaryIndexedTree<AdditiveOperation<_>>> = raw
        .into_iter()
        .map(|values| BinaryIndexedTree::from_slice(&values))
        .collect();
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { p, x } => {
                a[p] += x;
                cq.for_each_index(p, |c, i| bits[c].update(i, x));
            }
            Query::Sum { v, l, r } => {
                let mut ans = if l == 0 && 0 < r { a[v] } else { 0 };
                cq.for_each_contour_range(v, l, r, |c, start, end| {
                    ans += bits[c].fold_abelian(start, end);
                });
                pp!(ans);
            }
        }
    }
}
