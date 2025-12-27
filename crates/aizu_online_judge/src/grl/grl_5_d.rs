use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::UndirectedSparseGraph,
    tools::SizedCollect,
};

#[verify::aizu_online_judge("GRL_5_D")]
pub fn grl_5_d(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, c: [SizedCollect<usize>]);
    let edges = c
        .take(n)
        .enumerate()
        .flat_map(|(u, it)| it.into_iter().map(move |v| (u, v)))
        .collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let et = graph.path_euler_tour_builder(0).build();
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(et.size);

    scan!(scanner, q);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, v, w: i64);
                et.update(v, w, -w, |k, x| bit.update(k, x));
            }
            1 => {
                scan!(scanner, u);
                let ans = et.fold(u, |k| bit.accumulate(k));
                writeln!(writer, "{}", ans).ok();
            }
            _ => unreachable!("unknown query"),
        }
    }
}
