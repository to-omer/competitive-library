use competitive::graph::{DirectedGraphScanner, ShortestPathExt};
use competitive::prelude::*;

#[verify::library_checker("shortest_path")]
pub fn shortest_path(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, s, t, (g, c): @DirectedGraphScanner::<usize, u64>::new(n, m));
    let sp = g
        .standard_sp_additive()
        .with_parent()
        .dijkstra_to([s], t, |eid| c[eid]);
    if let Some(path) = sp.path_to(&g, t) {
        pp!(sp.dist[t], path.len() - 1; @it2d path.windows(2));
    } else {
        pp!(-1);
    }
}
