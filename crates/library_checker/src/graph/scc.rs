#[doc(no_inline)]
pub use competitive::graph::{DirectedSparseGraph, StronglyConnectedComponent};
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/scc")]
pub fn scc(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, edges: [(usize, usize); es]);
    let graph = DirectedSparseGraph::from_edges(vs, edges);
    let scc = StronglyConnectedComponent::new(&graph);
    let comp = scc.components();
    writeln!(writer, "{}", comp.len()).ok();
    for vs in comp.into_iter() {
        write!(&mut writer, "{} ", vs.len()).ok();
        echo(&mut writer, vs, ' ').ok();
    }
}
