use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    graph::UndirectedSparseGraph,
    tools::SizedCollect,
    tree::{EulerTourForRichVertex, LcaMonoidDefaultId},
};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_C")]
pub fn grl_5_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, c: [SizedCollect<usize>]);
    let edges = c
        .take(n)
        .enumerate()
        .flat_map(|(u, it)| it.into_iter().map(move |v| (u, v)))
        .collect();
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let et = EulerTourForRichVertex::new(0, &graph);
    let lca = et.gen_lca::<LcaMonoidDefaultId>();
    scan!(scanner, q, uv: [(usize, usize)]);
    for (u, v) in uv.take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}
