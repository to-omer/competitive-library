#[doc(no_inline)]
pub use competitive::graph::DirectedGraphScanner;
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_4_A")]
pub fn grl_4_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): @DirectedGraphScanner::<usize, ()>::new(vs, es));
    writeln!(writer, "{}", (graph.topological_sort().len() != vs) as u32).ok();
}
