pub use crate::graph::{LowLink, UndirectedGraphScanner};
use crate::prelude::*;

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/3/GRL_3_A")]
pub fn grl_3_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _, _): { UndirectedGraphScanner::<usize, ()>::new(vs, es) });
    let mut articulation = LowLink::new(&graph).articulation;
    articulation.sort();
    for u in articulation.into_iter() {
        writeln!(writer, "{}", u).ok();
    }
}
