pub use crate::graph::{GraphScanner, StronglyConnectedComponent};
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/3/GRL_3_C")]
pub fn grl_3_c(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, _) = scanner.mscan(GraphScanner::<usize, ()>::new(vs, es, true));
    let scc = StronglyConnectedComponent::new(&graph);
    scan!(scanner, q);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", (scc[u] == scc[v]) as u32).ok();
    }
}
