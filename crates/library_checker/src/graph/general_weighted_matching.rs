#[doc(no_inline)]
pub use competitive::graph::GeneralWeightedMatching;
use competitive::prelude::*;

#[verify::library_checker("general_weighted_matching")]
pub fn general_weighted_matching(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, uvw: [(usize, usize, i64); m]);
    let mut gm = GeneralWeightedMatching::from_edges(n, &uvw);
    let (w, matching) = gm.maximum_weight_matching();
    writeln!(writer, "{} {}", matching.len(), w).ok();
    for (u, v) in matching {
        writeln!(writer, "{} {}", u, v).ok();
    }
}
