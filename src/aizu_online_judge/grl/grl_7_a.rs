pub use crate::algebra::AdditiveOperation;
pub use crate::graph::Dinic;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/7/GRL_7_A")]
pub fn grl_7_a(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, xs, ys, es);
    let mut dinic = Dinic::new(xs + ys + 2);
    for x in 0..xs {
        dinic.add_edge(0, x + 2, 1);
    }
    for y in 0..ys {
        dinic.add_edge(y + xs + 2, 1, 1);
    }
    for _ in 0..es {
        scan!(scanner, x, y);
        dinic.add_edge(x + 2, y + xs + 2, 1);
    }
    writeln!(writer, "{}", dinic.maximum_flow(0, 1))
}
