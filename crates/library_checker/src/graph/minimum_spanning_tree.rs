#[doc(no_inline)]
pub use competitive::graph::EdgeListGraphScanner;
use competitive::prelude::*;

#[verify::library_checker("minimum_spanning_tree")]
pub fn minimum_spanning_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, (graph, w): @EdgeListGraphScanner::<usize, u64>::new(n, m));
    let span = graph.minimum_spanning_tree(|&eid| w[eid]);
    let ans = (0..m).filter(|&eid| span[eid]);
    let total: u64 = ans.clone().map(|eid| w[eid]).sum();
    iter_print!(writer, total; @it ans);
}
