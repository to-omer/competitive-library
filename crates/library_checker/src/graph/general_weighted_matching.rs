use competitive::graph::GeneralWeightedMatching;
use competitive::prelude::*;

#[verify::library_checker("general_weighted_matching")]
pub fn general_weighted_matching(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, uvw: [(usize, usize, i64); m]);
    let mut gm = GeneralWeightedMatching::from_edges(n, &uvw);
    let (w, matching) = gm.maximum_weight_matching();
    pp!(matching.len(), w; @ittup matching);
}
