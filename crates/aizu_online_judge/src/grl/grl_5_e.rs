use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::{AdditiveOperation, CartesianOperation},
    data_structure::LazySegmentTree,
    graph::UndirectedSparseGraph,
    tree::HeavyLightDecomposition,
};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_E")]
pub fn grl_5_e(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let mut edges = Vec::with_capacity(n - 1);
    for u in 0..n {
        scan!(scanner, k, c: [usize]);
        edges.extend(c.take(k).map(|v| (u, v)));
    }
    let mut graph = UndirectedSparseGraph::from_edges(n, edges);
    let hld = HeavyLightDecomposition::new(0, &mut graph);
    let monoid = CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new());
    let mut seg = LazySegmentTree::from_vec(
        vec![(0u64, 1u64); n],
        monoid.clone(),
        AdditiveOperation::new(),
        |x, &y| (x.0 + y * x.1, x.1),
    );

    scan!(scanner, q);
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, v, w: u64);
            hld.update(0, v, true, |l, r| seg.update(l, r, w));
        } else {
            scan!(scanner, u);
            let ans = hld.query(0, u, true, |l, r| seg.fold(l, r), &monoid).0;
            writeln!(writer, "{}", ans).ok();
        }
    }
}
