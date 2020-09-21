pub use competitive::algebra::AdditiveOperation;
pub use competitive::graph::EdgeListGraphScanner;
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/directedmst")]
pub fn directedmst(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, s, (graph, w): { EdgeListGraphScanner::<usize, i64>::new(n, m) });
    let res = graph
        .minimum_spanning_arborescence(s, AdditiveOperation::new(), |u| w[u])
        .unwrap();
    writeln!(writer, "{}", res.0).ok();
    echo(writer, res.1, ' ').ok();
}
