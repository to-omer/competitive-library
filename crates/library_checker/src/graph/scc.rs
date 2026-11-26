use competitive::graph::StronglyConnectedComponent;
use competitive::prelude::*;

#[verify::library_checker("scc", include("StronglyConnectedComponent"), huge_pages)]
pub fn scc(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, edges: [u32; 2 * m; capacity 2 * m]);
    let (edges, _) = edges.as_chunks();
    let scc = StronglyConnectedComponent::from_edges(
        n,
        edges.iter().map(|&[u, v]| (u as usize, v as usize)),
    );
    pp!(scc.size());
    for group in scc.components() {
        pp!(group.len(), @it group);
    }
}
