use competitive::data_structure::WaveletMatrix;
use competitive::prelude::*;

#[verify::library_checker("range_kth_smallest")]
pub fn range_kth_smallest(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, a: [usize; n], queries: [(usize, usize, usize); iter q]);
    let wm = WaveletMatrix::new(a);
    let results = wm.quantile_batch(queries.map(|(l, r, k)| (l..r, k)));
    pp!(@lf @it results);
}
