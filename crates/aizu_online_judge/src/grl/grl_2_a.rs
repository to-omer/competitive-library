#[doc(no_inline)]
pub use competitive::graph::EdgeListGraphScanner;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/2/GRL_2_A")]
pub fn grl_2_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, w): @EdgeListGraphScanner::<usize, u64>::new(vs, es));
    let span = graph.minimum_spanning_tree(|&eid| w[eid]);
    let ans = (0..es).map(|eid| w[eid] * span[eid] as u64).sum::<u64>();
    writeln!(writer, "{}", ans).ok();
}
