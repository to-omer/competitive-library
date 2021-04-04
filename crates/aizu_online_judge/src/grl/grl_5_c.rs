use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    graph::UndirectedSparseGraph,
    tree::{EulerTourForRichVertex, LcaMonoidDefaultId},
};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_C")]
pub fn grl_5_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let mut edges = Vec::with_capacity(n - 1);
    for u in 0..n {
        scan!(scanner, k, c: [usize]);
        edges.extend(c.take(k).map(|v| (u, v)));
    }
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let et = EulerTourForRichVertex::new(0, &graph);
    let lca = et.gen_lca::<LcaMonoidDefaultId>();
    scan!(scanner, q);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}
