#[doc(no_inline)]
pub use competitive::graph::GeneralMatching;
use competitive::prelude::*;

#[verify::library_checker("general_matching")]
pub fn general_matching(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, uv: [(usize, usize); m]);
    let mut gm = GeneralMatching::from_edges(n, &uv);
    let matching = gm.maximum_matching();
    writeln!(writer, "{}", matching.len()).ok();
    for (u, v) in matching {
        writeln!(writer, "{} {}", u, v).ok();
    }
}
