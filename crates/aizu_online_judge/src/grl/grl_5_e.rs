use competitive::prelude::*;
use competitive::{
    algebra::RangeSumRangeAdd, data_structure::LazySegmentTree, graph::UndirectedSparseGraph,
    tools::SizedCollect,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { v: usize, w: u64 }
        1 => Get { u: usize }
    }
}

#[verify::aizu_online_judge("GRL_5_E")]
pub fn grl_5_e(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, c: [SizedCollect<usize>]);
    let edges = c
        .take(n)
        .enumerate()
        .flat_map(|(u, it)| it.into_iter().map(move |v| (u, v)))
        .collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let hld = graph.hld(0);
    let mut seg = LazySegmentTree::<RangeSumRangeAdd<_>>::from_vec(vec![(0u64, 1u64); n]);

    sc!(q);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { v, w } => {
                hld.path_edges(0, v, |l, r| seg.update(l..r, w));
            }
            Query::Get { u } => {
                let mut ans = 0;
                hld.path_edges(0, u, |l, r| ans += seg.fold(l..r).0);
                pp!(ans);
            }
        }
    }
}
