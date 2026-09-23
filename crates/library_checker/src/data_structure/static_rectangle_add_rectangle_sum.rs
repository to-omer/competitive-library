use competitive::prelude::*;
use competitive::{
    algebra::{AdditiveOperation, ArrayOperation},
    algorithm::SliceSortExt,
    data_structure::{BinaryIndexedTree, StaticSearch},
    num::{Zero, mint_basic::MInt998244353 as M},
};

#[verify::library_checker("static_rectangle_add_rectangle_sum")]
pub fn static_rectangle_add_rectangle_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, rectangles: [(u32, u32, u32, u32, M); n], queries: [(u32, u32, u32, u32); q]);
    let mut ys: Vec<_> = rectangles
        .iter()
        .flat_map(|&(_, d, _, u, _)| [d, u])
        .collect();
    ys.radix_sort_by_key(|&y| y);
    ys.dedup();
    let search = StaticSearch::from_sorted(&ys);
    let endpoints: Vec<_> = rectangles
        .iter()
        .flat_map(|&(_, d, _, u, _)| [d, u])
        .chain(queries.iter().flat_map(|&(_, d, _, u)| [d, u]))
        .collect();
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let mut points: Vec<_> = rectangles
        .into_iter()
        .zip(positions[..2 * n].as_chunks().0)
        .flat_map(|((l, _, r, _, w), &[d, u])| {
            let d = d as u32;
            let u = u as u32;
            [(l, d, w), (l, u, -w), (r, d, -w), (r, u, w)]
        })
        .collect();
    points.radix_sort_by_key(|&(x, ..)| x);
    let mut events: Vec<_> = queries
        .into_iter()
        .zip(positions[2 * n..].as_chunks().0)
        .enumerate()
        .flat_map(|(i, ((l, d, r, u), &[di, ui]))| {
            let di = di as u32;
            let ui = ui as u32;
            [
                (l, d, u, di, ui, i as u32, false),
                (r, d, u, di, ui, i as u32, true),
            ]
        })
        .collect();
    events.radix_sort_by_key(|&(x, ..)| x);
    let mut bit = BinaryIndexedTree::<ArrayOperation<AdditiveOperation<M>, 4>>::new(ys.len());
    let mut points = points.into_iter().peekable();
    let mut ans = vec![M::zero(); q];
    for (x, d, u, di, ui, i, add) in events {
        while points.peek().is_some_and(|&(px, ..)| px < x) {
            let (px, py, w) = points.next().unwrap();
            let wx = w * M::from(px);
            let wy = w * M::from(ys[py as usize]);
            bit.update(py as usize, [w, wx, wy, wx * M::from(ys[py as usize])]);
        }
        for (y, yi, add) in [(d, di, !add), (u, ui, add)] {
            let [w, wx, wy, wxy] = bit.accumulate0(yi as usize);
            let value = (w * M::from(x) - wx) * M::from(y) - wy * M::from(x) + wxy;
            if add {
                ans[i as usize] += value;
            } else {
                ans[i as usize] -= value;
            }
        }
    }
    pp!(@lf @it ans);
}
