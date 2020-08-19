pub use crate::algebra::AdditiveOperation;
pub use crate::graph::GraphScanner;
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/directedmst")]
pub fn directedmst(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, s);
    let (graph, w) = scanner.mscan(GraphScanner::<usize, i64>::new(n, m, true));
    let res = graph
        .minimum_spanning_arborescence(s, AdditiveOperation::new(), |u| w[u])
        .unwrap();
    writeln!(writer, "{}", res.0).ok();
    echo!(writer, res.1, " ");
}
