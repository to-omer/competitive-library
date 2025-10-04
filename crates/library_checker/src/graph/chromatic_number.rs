use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algorithm::IndependentSubSet, num::mint_basic::Modulo998244353};

#[verify::library_checker("chromatic_number")]
pub fn chromatic_number(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, uv: [(usize, usize); m]);
    let ind = IndependentSubSet::<Modulo998244353>::from_edges(n, &uv);
    iter_print!(writer, ind.chromatic_number());
}
