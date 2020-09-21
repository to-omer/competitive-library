#[doc(no_inline)]
pub use competitive::graph::{LowLink, UndirectedGraphScanner};
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/3/GRL_3_A")]
pub fn grl_3_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): { UndirectedGraphScanner::<usize, ()>::new(vs, es) });
    let mut articulation = LowLink::new(&graph).articulation;
    articulation.sort_unstable();
    for u in articulation.into_iter() {
        writeln!(writer, "{}", u).ok();
    }
}
