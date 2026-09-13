use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation, algorithm::SliceSortExt, data_structure::WaveletMatrix,
};

#[verify::library_checker("rectangle_sum")]
pub fn rectangle_sum(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, mut xyw: [(u32, u32, i64); n]);
    xyw.radix_sort_by_key(|&(x, _, _)| x);
    let ys = xyw.iter().map(|&(_, y, _)| y).collect();
    let weights: Vec<_> = xyw.iter().map(|&(_, _, w)| w).collect();
    let wm = WaveletMatrix::new(ys);
    let fold = wm.build_fold::<AdditiveOperation<i64>>(&weights);
    for _ in 0..q {
        scan!(scanner, l: u32, d: u32, r: u32, u: u32);
        let l = xyw.partition_point(|&(x, _, _)| x < l);
        let r = xyw.partition_point(|&(x, _, _)| x < r);
        writeln!(writer, "{}", fold.fold_range(d..u, l..r)).ok();
    }
}
