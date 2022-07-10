use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::UndirectedSparseGraph,
    tools::SizedCollect, tree::EulerTourForEdge,
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
    let et = EulerTourForEdge::new(0, &graph);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(et.length());

    scan!(scanner, q);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, v, w: i64);
                let (l, r) = et.eidx[et.par[v]];
                bit.update(l, w);
                bit.update(r, -w);
            }
            1 => {
                scan!(scanner, u);
                let ans = if u > 0 {
                    bit.accumulate(et.eidx[et.par[u]].0)
                } else {
                    0
                };
                writeln!(writer, "{}", ans).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
