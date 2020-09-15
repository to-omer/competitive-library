pub use crate::graph::UndirectedSparseGraph;
use crate::prelude::*;
pub use crate::tree::EulerTourForRichVertex;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_C")]
pub fn grl_5_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let mut edges = Vec::with_capacity(n - 1);
    for u in 0..n {
        scan!(scanner, k);
        for v in scanner.iter::<usize>().take(k) {
            edges.push((u, v));
        }
    }
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let et = EulerTourForRichVertex::new(0, &graph);
    let lca = et.gen_lca();
    scan!(scanner, q);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}
