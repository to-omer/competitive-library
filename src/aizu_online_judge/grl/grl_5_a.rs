pub use crate::algebra::AdditiveOperation;
pub use crate::graph::GraphScanner;
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_A")]
pub fn grl_5_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let (graph, w) = scanner.mscan(GraphScanner::<usize, u64>::new(n, n - 1, false));
    let d = graph.weighted_tree_depth(0, |eid| w[eid], AdditiveOperation::new());
    let r = (0..n).max_by_key(|&u| d[u]).unwrap();
    let ans = graph
        .weighted_tree_depth(r, |eid| w[eid], AdditiveOperation::new())
        .into_iter()
        .max()
        .unwrap();
    writeln!(writer, "{}", ans).ok();
}
