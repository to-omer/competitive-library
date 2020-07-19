pub use crate::graph::{Graph, GraphScanner};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/2/GRL_2_A")]
pub fn grl_2_a(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, w) = scanner.mscan(GraphScanner::<usize, u64>::new(vs, es, false));
    let span = graph.minimum_spanning_tree(|&eid| w[eid]);
    let ans = (0..es).map(|eid| w[eid] * span[eid] as u64).sum::<u64>();
    writeln!(writer, "{}", ans)
}
