use competitive::prelude::*;
use competitive::{algebra::AdditiveOperation, graph::TreeGraphScanner};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { p: usize, x: i64 }
        1 => Sum { v: usize, l: usize, r: usize }
    }
}

#[verify::library_checker("vertex_add_range_contour_sum_on_tree")]
pub fn vertex_add_range_contour_sum_on_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [i64; n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let cq = graph.contour_query_range();
    let mut fold = cq.build_point_add::<AdditiveOperation<_>>(&a);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { p, x } => {
                fold.update(p, x);
            }
            Query::Sum { v, l, r } => {
                pp!(fold.fold(v, l, r));
            }
        }
    }
}
