pub use crate::graph::{GraphScanner, StronglyConnectedComponent};
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/scc")]
pub fn scc(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): {GraphScanner::<usize, ()>::new(vs, es, true)});
    let scc = StronglyConnectedComponent::new(&graph);
    let comp = scc.components();
    writeln!(writer, "{}", comp.len()).ok();
    for vs in comp.into_iter() {
        write!(writer, "{} ", vs.len()).ok();
        echo(writer, vs, ' ').ok();
    }
}
