use competitive::prelude::*;
use competitive::{
    algebra::{LinearOperation, ReverseOperation},
    data_structure::SegmentTree,
    graph::TreeGraphScanner,
    num::mint_basic::MInt998244353 as M,
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
    let mut nab = vec![(M::default(), M::default()); n];
    for i in 0..n {
        nab[hld.index(i)] = ab[i];
    }
    let mut seg1 = SegmentTree::<LinearOperation<_>>::from_vec(nab.clone());
    let mut seg2 = SegmentTree::<ReverseOperation<LinearOperation<_>>>::from_vec(nab);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Set { p, cd } => {
                seg1.set(hld.index(p), cd);
                seg2.set(hld.index(p), cd);
            }
            Query::Apply { u, v, x } => {
                let (a, b) = hld.fold_vertices::<LinearOperation<_>, _, _>(
                    u,
                    v,
                    |l, r| seg1.fold(l..r),
                    |l, r| seg2.fold(l..r),
                );
                pp!(a * x + b);
            }
        }
    }
}
