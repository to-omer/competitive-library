use competitive::prelude::*;
use competitive::{
    data_structure::UnionFind,
    graph::{LowLink, UndirectedSparseGraph},
};

#[verify::library_checker("two_edge_connected_components")]
pub fn two_edge_connected_components(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, edges: [(usize, usize); m]);
    let graph = UndirectedSparseGraph::from_edges(n, edges);
    let low_link = LowLink::new(&graph);
    let mut uf = UnionFind::new(n);
    for &(mut u, mut v) in &graph.edges {
        if low_link.ord[u] > low_link.ord[v] {
            std::mem::swap(&mut u, &mut v);
        }
        if low_link.ord[u] >= low_link.low[v] {
            uf.unite(u, v);
        }
    }
    let groups = uf.all_group_members();
    pp!(groups.len());
    for group in groups.into_values() {
        pp!(group.len(), @it group);
    }
}
