use competitive::graph::EdgeListGraphScanner;
use competitive::prelude::*;

#[verify::library_checker("tree_diameter")]
pub fn tree_diameter(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, (graph, weights): @EdgeListGraphScanner::<usize, u32>::new(n, n - 1));
    let (diameter, path) = graph.tree_diameter(|&e| weights[e] as u64);
    pp!(diameter, path.len(); @it path);
}
