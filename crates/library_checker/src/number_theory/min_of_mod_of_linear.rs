use competitive::math::min_of_mod_of_linear as min_of_mod_of_linear_library;
use competitive::prelude::*;

#[verify::library_checker("min_of_mod_of_linear")]
pub fn min_of_mod_of_linear(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, query: [(u64, u64, u64, u64)]);
    for (n, m, a, b) in query.take(t) {
        writeln!(writer, "{}", min_of_mod_of_linear_library(n, a, b, m)).ok();
    }
}
