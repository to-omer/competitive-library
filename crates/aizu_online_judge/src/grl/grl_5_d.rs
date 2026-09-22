use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::UndirectedSparseGraph,
    tools::SizedCollect,
};

competitive::define_enum_scan! {
    enum Query: usize {
        0 => Add { v: usize, w: i64 }
        1 => Get { u: usize }
    }
}

#[verify::aizu_online_judge("GRL_5_D")]
pub fn grl_5_d(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, c: [SizedCollect<usize>; iter n]);
    let edges = c
        .enumerate()
        .flat_map(|(u, it)| it.into_iter().map(move |v| (u, v)))
        .collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let et = graph.path_euler_tour_builder(0).build();
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(et.size);

    sc!(q);
    for _ in 0..q {
        sc!(query: Query);
        match query {
            Query::Add { v, w } => {
                et.update(v, w, -w, |k, x| bit.update(k, x));
            }
            Query::Get { u } => {
                let ans = et.fold(u, |k| bit.accumulate(k));
                pp!(ans);
            }
        }
    }
}
