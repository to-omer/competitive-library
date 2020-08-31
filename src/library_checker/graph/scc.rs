pub use crate::graph::{RevGraphScanner, StronglyConnectedComponent};
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/scc")]
pub fn scc(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): {RevGraphScanner::<usize, ()>::new(vs, es)});
    let scc = StronglyConnectedComponent::new(&graph);
    let comp = scc.components();
    writeln!(writer, "{}", comp.len()).ok();
    for vs in comp.into_iter() {
        write!(writer, "{}", vs.len()).ok();
        for v in vs.into_iter() {
            write!(writer, " {}", v).ok();
        }
        writeln!(writer, "").ok();
    }
}
