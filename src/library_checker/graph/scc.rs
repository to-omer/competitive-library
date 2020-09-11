pub use crate::graph::{
    AdjacencyGraphAbstraction, DirectedSparseGraph, StronglyConnectedComponent,
};
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/scc")]
pub fn scc(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, edges: [(usize, usize); es]);
    let graph = DirectedSparseGraph::from_edges(vs, edges.iter().cloned());
    let scc = StronglyConnectedComponent::new(&graph);
    let comp = scc.components();
    writeln!(writer, "{}", comp.len()).ok();
    for vs in comp.into_iter() {
        write!(writer, "{} ", vs.len()).ok();
        echo(writer, vs, ' ').ok();
    }
}
