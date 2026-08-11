use competitive::data_structure::WaveletMatrix;
use competitive::prelude::*;

#[verify::library_checker("range_kth_smallest")]
pub fn range_kth_smallest(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [usize; n], queries: [(usize, usize, usize)]);
    let wm = WaveletMatrix::new(a);
    let results = wm.quantile_batch(queries.take(q).map(|(l, r, k)| (l..r, k)));
    iter_print!(writer, @lf @it results);
}
