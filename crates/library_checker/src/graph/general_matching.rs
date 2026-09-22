use competitive::graph::GeneralMatching;
use competitive::prelude::*;

#[verify::library_checker("general_matching")]
pub fn general_matching(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, uv: [(usize, usize); iter m]);
    let mut gm = GeneralMatching::new(n);
    for (u, v) in uv {
        gm.add_edge(u, v);
    }
    let matching = gm.maximum_matching();
    pp!(matching.len(); @ittup matching);
}
