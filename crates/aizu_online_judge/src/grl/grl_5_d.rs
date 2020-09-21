use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation, data_structure::BinaryIndexedTree, graph::UndirectedSparseGraph,
    tree::EulerTourForEdge,
};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_D")]
pub fn grl_5_d(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let mut edges = Vec::with_capacity(n - 1);
    for u in 0..n {
        scan!(scanner, k, c: [usize]);
        edges.extend(c.take(k).map(|v| (u, v)));
    }
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let et = EulerTourForEdge::new(0, &graph);
    let mut bit = BinaryIndexedTree::new(et.length(), AdditiveOperation::new());

    scan!(scanner, q);
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, v, w: i64);
            let (l, r) = et.eidx[et.par[v]];
            bit.update(l, w);
            bit.update(r, -w);
        } else {
            scan!(scanner, u);
            let ans = if u > 0 {
                bit.accumulate(et.eidx[et.par[u]].0)
            } else {
                0
            };
            writeln!(writer, "{}", ans).ok();
        }
    }
}
