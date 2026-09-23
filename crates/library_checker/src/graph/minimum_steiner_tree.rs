use competitive::graph::{SteinerTreeExt, UndirectedGraphScanner};
use competitive::prelude::*;

#[verify::library_checker("minimum_steiner_tree")]
pub fn minimum_steiner_tree(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, (graph, weights): @UndirectedGraphScanner::<usize, u64>::new(n, m));
    sc!(k, terminals: [usize; k]);
    let tree = graph
        .steiner_tree()
        .with_standard_sp_additive()
        .with_parent()
        .solve(terminals[1..].iter().copied(), |eid| weights[eid]);
    let edges = tree.edges_from_source(terminals[0]).unwrap();
    pp!(tree.minimum_from_source(terminals[0]), edges.len(); @it edges);
}
