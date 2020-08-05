pub use crate::graph::Graph;
use crate::scan;
use crate::tools::{read_all, Scanner};
pub use crate::tree::EulerTourForRichVertex;
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_C")]
pub fn grl_5_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let mut graph = Graph::new(n);
    for u in graph.vertices() {
        scan!(scanner, k);
        for v in scanner.iter::<usize>().take(k) {
            graph.add_undirected_edge(u, v);
        }
    }
    let mut et = EulerTourForRichVertex::new(n);
    et.vertex_tour(0, n, &graph);
    let lca = et.gen_lca(&graph);
    scan!(scanner, q);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", lca.lca(u, v)).ok();
    }
}
