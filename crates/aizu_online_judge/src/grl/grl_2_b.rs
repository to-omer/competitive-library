use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation,
    graph::{EdgeListGraph, EdgeListGraphScanner},
};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/2/GRL_2_B")]
pub fn grl_2_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, root, (graph, w): @EdgeListGraphScanner::<usize, i64>::new(vs, es));
    let res = graph.minimum_spanning_arborescence::<AdditiveOperation<_>, _>(root, |u| w[u]);
    writeln!(writer, "{}", res.unwrap().0).ok();
}
