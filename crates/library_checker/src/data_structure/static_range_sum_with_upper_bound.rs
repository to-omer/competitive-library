use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, data_structure::WaveletMatrix};

#[verify::library_checker("static_range_sum_with_upper_bound")]
pub fn static_range_sum_with_upper_bound(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let weights = a.clone();
    let wm = WaveletMatrix::new(a);
    let fold = wm.build_fold::<AdditiveOperation<i64>>(&weights);
    for _ in 0..q {
        scan!(scanner, l, r, x: i64);
        let (count, sum) = fold.fold_lessthan_with_count(x + 1, l..r);
        writeln!(writer, "{} {}", count, sum).ok();
    }
}
