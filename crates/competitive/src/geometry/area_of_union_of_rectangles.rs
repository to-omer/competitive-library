use super::{LazySegmentTree, RangeMinCountRangeAdd, SliceSortExt, StaticSearch};

/// Rectangles are `(left, bottom, right, top)`.
pub fn area_of_union_of_rectangles(rectangles: &[(u32, u32, u32, u32)]) -> u64 {
    let endpoints: Vec<_> = rectangles.iter().flat_map(|&(_, d, _, u)| [d, u]).collect();
    let mut ys = endpoints.clone();
    ys.radix_sort_by_key(|&y| y);
    ys.dedup();
    if ys.len() < 2 {
        return 0;
    }
    let search = StaticSearch::from_sorted(&ys);
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let mut events: Vec<_> = rectangles
        .iter()
        .zip(positions.as_chunks().0)
        .flat_map(|(&(l, _, r, _), &[d, u])| {
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
    let mut area = 0;
    for (x, d, u, delta) in events {
        let (minimum, count) = seg.fold_all();
        let covered = height - if minimum == 0 { count } else { 0 };
        area += (x - prev_x) as u64 * covered as u64;
        seg.update(d as usize..u as usize, delta);
        prev_x = x;
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::testutil::exhaustive_sequences;

    #[test]
    fn test_area_of_union_of_rectangles() {
        let intervals: Vec<_> = (0..=2).flat_map(|l| (l..=2).map(move |r| (l, r))).collect();
        let rectangles: Vec<_> = intervals
            .iter()
            .flat_map(|&(l, r)| intervals.iter().map(move |&(d, u)| (l, d, r, u)))
            .collect();
        for coordinates in [[0, 1, 2], [0, u32::MAX - 1, u32::MAX]] {
            for rectangles in exhaustive_sequences(rectangles.iter().copied(), 0..=2) {
                let input: Vec<_> = rectangles
                    .into_iter()
                    .map(|(l, d, r, u)| {
                        (
                            coordinates[l],
                            coordinates[d],
                            coordinates[r],
                            coordinates[u],
                        )
                    })
                    .collect();
                let expected: u64 = (0..2)
                    .flat_map(|x| (0..2).map(move |y| (x, y)))
                    .filter(|&(x, y)| {
                        input.iter().any(|&(l, d, r, u)| {
                            l <= coordinates[x]
                                && coordinates[x] < r
                                && d <= coordinates[y]
                                && coordinates[y] < u
                        })
                    })
                    .map(|(x, y)| {
                        (coordinates[x + 1] - coordinates[x]) as u64
                            * (coordinates[y + 1] - coordinates[y]) as u64
                    })
                    .sum();
                assert_eq!(area_of_union_of_rectangles(&input), expected);
            }
        }
    }
}
