pub use crate::graph::{GraphScanner, LowLink};
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/3/GRL_3_A")]
pub fn grl_3_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es);
    let (graph, _) = scanner.mscan(GraphScanner::<usize, ()>::new(vs, es, false));
    let mut articulation = LowLink::new(&graph).articulation;
    articulation.sort();
    for u in articulation.into_iter() {
        writeln!(writer, "{}", u).ok();
    }
}
