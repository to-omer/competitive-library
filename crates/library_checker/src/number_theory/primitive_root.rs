#[doc(no_inline)]
pub use competitive::math::primitive_root as primitive_root_library;
use competitive::prelude::*;

#[verify::library_checker("primitive_root")]
pub fn primitive_root(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, q, p: [u64]);
    for p in p.take(q) {
        writeln!(writer, "{}", primitive_root_library(p)).ok();
    }
}
