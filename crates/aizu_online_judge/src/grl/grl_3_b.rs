#[doc(no_inline)]
pub use competitive::graph::{LowLink, UndirectedGraphScanner};
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_3_B")]
pub fn grl_3_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): @UndirectedGraphScanner::<usize, ()>::new(vs, es));
    let mut bridge = LowLink::new(&graph).bridge;
    bridge.sort_unstable();
    for (u, v) in bridge.into_iter() {
        writeln!(writer, "{} {}", u, v).ok();
    }
}
