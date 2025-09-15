use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algorithm::binary_search, math::floor_sum_range_freq};

#[verify::library_checker("min_of_mod_of_linear")]
pub fn min_of_mod_of_linear(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, t, query: [(i64, u64, i64, i64)]);
    for (n, m, a, b) in query.take(t) {
        let x = binary_search(
            |&x| floor_sum_range_freq(0, n, a, b, m, 0..x + 1) > 0,
            m as i64 - 1,
            -1,
        );
        writeln!(writer, "{}", x).ok();
    }
}
