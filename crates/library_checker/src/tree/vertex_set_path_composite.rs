use competitive::prelude::*;
use competitive::{
    algebra::LinearOperation, graph::TreeGraphScanner, num::mint_basic::MInt998244353 as M,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Set { p: usize, cd: (M, M) }
        1 => Apply { u: usize, v: usize, x: M }
    }
}

#[verify::library_checker("vertex_set_path_composite")]
pub fn vertex_set_path_composite(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, ab: [(M, M); n], (graph, _): @TreeGraphScanner::<usize, ()>::new(n));
    let hld = graph.hld(0);
    let mut fold = hld.build_fold::<LinearOperation<_>>(&ab);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Set { p, cd } => {
                fold.set(p, cd);
            }
            Query::Apply { u, v, x } => {
                let (a, b) = fold.fold_vertices(u, v);
                pp!(a * x + b);
            }
        }
    }
}
