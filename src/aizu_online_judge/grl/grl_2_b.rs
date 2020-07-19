pub use crate::algebra::AdditiveOperation;
pub use crate::graph::{RevGraph, RevGraphScanner};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/2/GRL_2_B")]
pub fn grl_2_b(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, root);
    let (graph, w) = scanner.mscan(RevGraphScanner::<usize, i64>::new(vs, es));
    let res = graph.minimum_spanning_arborescence(root, AdditiveOperation::new(), &w, 0i64);
    writeln!(writer, "{}", res.unwrap_or_default())
}
