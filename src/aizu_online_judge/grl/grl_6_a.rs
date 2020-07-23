pub use crate::algebra::AdditiveOperation;
pub use crate::graph::Dinic;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/6/GRL_6_A")]
pub fn grl_6_a(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let mut dinic = Dinic::new(vs);
    for _ in 0..es {
        scan!(scanner, u, v, c: u64);
        dinic.add_edge(u, v, c);
    }
    writeln!(writer, "{}", dinic.maximum_flow(0, vs - 1))
}
