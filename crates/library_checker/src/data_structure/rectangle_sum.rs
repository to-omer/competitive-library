use competitive::prelude::*;
use competitive::{
    algebra::AdditiveOperation,
    algorithm::SliceSortExt,
    data_structure::{StaticSearch, WaveletMatrix},
};

#[verify::library_checker("rectangle_sum")]
pub fn rectangle_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, mut xyw: [(u32, u32, i64); n], queries: [(u32, u32, u32, u32); q]);
    xyw.radix_sort_by_key(|&(x, ..)| x);
    let xs: Vec<_> = xyw.iter().map(|&(x, ..)| x).collect();
    let search = StaticSearch::from_sorted(&xs);
    let endpoints: Vec<_> = queries.iter().flat_map(|&(l, _, r, _)| [l, r]).collect();
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let ys = xyw.iter().map(|&(_, y, _)| y).collect();
    let weights: Vec<_> = xyw.iter().map(|&(_, _, w)| w).collect();
    let wm = WaveletMatrix::new(ys);
    let fold = wm.build_fold::<AdditiveOperation<i64>>(&weights);
    let result = fold.fold_lessthan_batch(
        queries
            .into_iter()
            .zip(positions.as_chunks::<2>().0)
            .flat_map(|((_, d, _, u), &[l, r])| [(d, l..r), (u, l..r)]),
    );
    for &[lower, upper] in result.as_chunks::<2>().0 {
        pp!(upper - lower);
    }
}
