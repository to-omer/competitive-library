use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{AdditiveOperation, RangeSumRangeAdd},
    data_structure::LazySegmentTree,
    graph::UndirectedSparseGraph,
    tools::SizedCollect,
    tree::HeavyLightDecomposition,
};

#[cfg_attr(
    nightly,
    verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_E")
)]
pub fn grl_5_e(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, c: [SizedCollect<usize>]);
    let edges = c
        .take(n)
        .enumerate()
        .map(|(u, it)| it.into_iter().map(move |v| (u, v)))
        .flatten()
        .collect();
    let mut graph = UndirectedSparseGraph::from_edges(n, edges);
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    type M = (AdditiveOperation<u64>, AdditiveOperation<u64>);
    let mut seg = LazySegmentTree::<RangeSumRangeAdd<_>>::from_vec(vec![(0u64, 1u64); n]);

    scan!(scanner, q);
    for _ in 0..q {
        match scanner.scan::<usize>() {
            0 => {
                scan!(scanner, v, w: u64);
                hld.update(0, v, true, |l, r| seg.update(l, r, w));
            }
            1 => {
                scan!(scanner, u);
                let ans = hld.query::<M, _>(0, u, true, |l, r| seg.fold(l, r)).0;
                writeln!(writer, "{}", ans).ok();
            }
            _ => panic!("unknown query"),
        }
    }
}
