#[doc(no_inline)]
pub use competitive::algorithm::FloorQuotientIndex;
use competitive::prelude::*;

#[verify::library_checker("enumerate_quotients")]
pub fn enumerate_quotients(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n);
    let qi = FloorQuotientIndex::new(n);
    iter_print!(writer, qi.len(); @it qi.values());
}
