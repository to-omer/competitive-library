use competitive::graph::DynamicStarMinimumCut;
use competitive::prelude::*;

#[verify::library_checker(
    "global_minimum_cut_of_dynamic_star_augmented_graph",
    include("DynamicStarMinimumCut")
)]
pub fn global_minimum_cut_of_dynamic_star_augmented_graph(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, q, star: [u64; n], edges: [(usize, usize, u64); iter m]);
    let mut graph = DynamicStarMinimumCut::new(star, edges);
    for _ in 0..q {
        sc!(v, w: u64);
        graph.set_weight(v, w);
        pp!(graph.minimum_cut());
    }
}
