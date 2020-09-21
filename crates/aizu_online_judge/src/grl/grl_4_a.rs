pub use competitive::graph::DirectedGraphScanner;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/4/GRL_4_A")]
pub fn grl_4_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): { DirectedGraphScanner::<usize, ()>::new(vs, es) });
    writeln!(writer, "{}", (graph.topological_sort().len() != vs) as u32).ok();
}
