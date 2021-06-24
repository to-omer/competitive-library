use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, graph::TreeGraphScanner};

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_A")]
pub fn grl_5_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, (graph, w): @TreeGraphScanner::<usize, u64>::new(n));
    let d = graph.weighted_tree_depth::<AdditiveOperation<_>, _>(0, |eid| w[eid]);
    let r = (0..n).max_by_key(|&u| d[u]).unwrap();
    let ans = graph
        .weighted_tree_depth::<AdditiveOperation<_>, _>(r, |eid| w[eid])
        .into_iter()
        .max()
        .unwrap();
    writeln!(writer, "{}", ans).ok();
}
