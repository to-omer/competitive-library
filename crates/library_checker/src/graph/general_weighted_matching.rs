use competitive::graph::GeneralWeightedMatching;
use competitive::prelude::*;

#[verify::library_checker("general_weighted_matching")]
pub fn general_weighted_matching(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, uvw: [(usize, usize, i64); iter m]);
    let mut gm = GeneralWeightedMatching::new(n);
    for (u, v, w) in uvw {
        gm.add_edge(u, v, w);
    }
    let (w, matching) = gm.maximum_weight_matching();
    pp!(matching.len(), w; @ittup matching);
}
