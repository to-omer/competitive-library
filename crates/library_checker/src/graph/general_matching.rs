use competitive::graph::GeneralMatching;
use competitive::prelude::*;

#[verify::library_checker("general_matching")]
pub fn general_matching(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, uv: [(usize, usize); m]);
    let mut gm = GeneralMatching::from_edges(n, &uv);
    let matching = gm.maximum_matching();
    pp!(matching.len());
    for (u, v) in matching {
        pp!(u, v);
    }
}
