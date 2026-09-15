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
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(cq.len() + 1);

    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { v, l, r, x } => {
                cq.for_each_contour_range(v, l, r, |start, end| {
                    bit.update(start, x);
                    bit.update(end, -x);
                });
                if l == 0 && 0 < r {
                    a[v] += x;
                }
            }
            Query::Get { v } => {
                let mut ans = a[v];
                cq.for_each_index(v, |i| ans += bit.accumulate(i));
                pp!(ans);
            }
        }
    }
}
