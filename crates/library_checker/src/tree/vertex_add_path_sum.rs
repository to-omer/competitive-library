use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::TreeGraphScanner,
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
    sc!(n, q, a: [i64; n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let hld = graph.hld(0);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(n);
    for (v, &x) in a.iter().enumerate() {
        bit.update(hld.index(v), x);
    }
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { p, x } => {
                bit.update(hld.index(p), x);
            }
            Query::Sum { u, v } => {
                let mut sum = 0;
                hld.path_vertices(u, v, |l, r| sum += bit.fold(l, r));
                pp!(sum);
            }
        }
    }
}
