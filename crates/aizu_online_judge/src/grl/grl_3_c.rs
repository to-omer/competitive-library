#[doc(no_inline)]
pub use competitive::graph::{DirectedGraphScanner, StronglyConnectedComponent};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_3_C")]
pub fn grl_3_c(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): @DirectedGraphScanner::<usize, ()>::new(vs, es));
    let scc = StronglyConnectedComponent::new(&graph);
    scan!(scanner, q);
    for (u, v) in scanner.iter::<(usize, usize)>().take(q) {
        writeln!(writer, "{}", (scc[u] == scc[v]) as u32).ok();
    }
}
