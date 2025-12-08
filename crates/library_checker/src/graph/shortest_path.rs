#[doc(no_inline)]
pub use competitive::graph::{DirectedGraphScanner, DirectedSparseGraph, ShortestPathExt};
use competitive::prelude::*;

#[verify::library_checker("shortest_path")]
pub fn shortest_path(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, s, t, (g, c): @DirectedGraphScanner::<usize, u64>::new(n, m));
    let sp = g.standard_sp_additive().with_parent().dijkstra_ss(s, &c);
    if let Some(path) = sp.path_to(&g, t) {
        iter_print!(writer, sp.dist[t], path.len() - 1; @it2d path.windows(2));
    } else {
        iter_print!(writer, -1);
    }
}
