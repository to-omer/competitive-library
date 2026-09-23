use competitive::prelude::*;
use competitive::{
    algebra::RangeMinCountRangeAdd,
    algorithm::SliceSortExt,
    data_structure::{LazySegmentTree, StaticSearch},
};

#[verify::library_checker("area_of_union_of_rectangles")]
pub fn area_of_union_of_rectangles(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, rectangles: [(u32, u32, u32, u32); n]);
    let endpoints: Vec<_> = rectangles.iter().flat_map(|&(_, d, _, u)| [d, u]).collect();
    let mut ys = endpoints.clone();
    ys.radix_sort_by_key(|&y| y);
    ys.dedup();
    let search = StaticSearch::from_sorted(&ys);
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let mut events: Vec<_> = rectangles
        .into_iter()
        .zip(positions.as_chunks().0)
        .flat_map(|((l, _, r, _), &[d, u])| {
            let d = d as u32;
            let u = u as u32;
            [(l, d, u, 1), (r, d, u, -1)]
        })
        .collect();
    events.radix_sort_by_key(|&(x, ..)| x);
    let mut seg = LazySegmentTree::<RangeMinCountRangeAdd<i32>>::from_vec(
        ys.windows(2).map(|w| (0, (w[1] - w[0]) as usize)).collect(),
    );
    let height = (ys[ys.len() - 1] - ys[0]) as usize;
    let mut prev_x = 0;
    let mut area = 0u64;
    for (x, d, u, delta) in events {
        let (minimum, count) = seg.fold_all();
        let covered = height - if minimum == 0 { count } else { 0 };
        area += (x - prev_x) as u64 * covered as u64;
        seg.update(d as usize..u as usize, delta);
        prev_x = x;
    }
    pp!(area);
}
