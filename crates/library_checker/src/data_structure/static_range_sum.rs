use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::Accumulate};

#[verify::library_checker("static_range_sum")]
pub fn static_range_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n], lr: [(usize, usize)]);
    let acc: Accumulate<AdditiveOperation<i64>> = a.into_iter().collect();
    for (l, r) in lr.take(q) {
        writeln!(writer, "{}", acc.fold(l..r)).ok();
    }
}
