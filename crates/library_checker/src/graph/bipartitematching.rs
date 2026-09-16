use competitive::graph::{BipartiteMatching, DinicBuilder};
use competitive::prelude::*;

#[verify::library_checker("bipartitematching")]
pub fn bipartitematching_dinic(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(l, r, m, ab: [(usize, usize); m]);
    let mut builder = DinicBuilder::new(l + r + 2, m + l + r);
    let s = l + r;
    let t = s + 1;
    for (a, b) in ab.iter().cloned() {
        builder.add_edge(a, b + l, 1);
    }
    for a in 0..l {
        builder.add_edge(s, a, 1);
    }
    for b in 0..r {
        builder.add_edge(b + l, t, 1);
    }
    let graph = builder.gen_graph();
    let mut dinic = builder.build(&graph);
    let f = dinic.maximum_flow(s, t);
    let ans = ab
        .into_iter()
        .enumerate()
        .filter_map(|(i, edge)| (dinic.get_flow(i) > 0).then_some(edge));
    pp!(f; @ittup ans);
}

#[verify::library_checker("bipartitematching")]
pub fn bipartitematching(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(l, r, m, ab: [(usize, usize); m]);
    let mut bm = BipartiteMatching::from_edges(l, r, &ab);
    let matching = bm.maximum_matching();
    pp!(matching.len(); @ittup matching);
}
