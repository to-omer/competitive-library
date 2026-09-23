use competitive::prelude::*;
use competitive::{
    algorithm::SliceSortExt,
    data_structure::{DaryPrefixSumTreeU32, StaticSearch},
};

#[verify::library_checker("ordered_set")]
pub fn ordered_set(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, a: [u32; n], queries: [(u8, u32); q]);
    let mut values: Vec<_> = a
        .iter()
        .copied()
        .chain(queries.iter().filter_map(|&(t, x)| (t <= 1).then_some(x)))
        .collect();
    values.radix_sort_by_key(|&x| x);
    values.dedup();
    let search = StaticSearch::from_sorted(&values);
    let endpoints: Vec<_> = a
        .into_iter()
        .chain(
            queries
                .iter()
                .map(|&(t, x)| if t == 3 || t == 4 { x + 1 } else { x }),
        )
        .collect();
    let mut positions = vec![0; endpoints.len()];
    search.lower_bound_batch(&endpoints, &mut positions);
    let mut counts = vec![0; values.len()];
    for &k in &positions[..n] {
        counts[k] = 1;
    }
    let mut seg = DaryPrefixSumTreeU32::from_slice(&counts);
    for ((t, x), &k) in queries.into_iter().zip(&positions[n..]) {
        match t {
            0 => seg.set(k, 1),
            1 => seg.set(k, 0),
            2 => {
                let k = seg.partition_point_acc(x - 1);
                pp!(values.get(k).map_or(-1, |&x| x as i64));
            }
            3 => {
                pp!(seg.accumulate0(k));
            }
            4 => {
                let count = seg.accumulate0(k);
                pp!(if count == 0 {
                    -1
                } else {
                    values[seg.partition_point_acc(count - 1)] as i64
                });
            }
            5 => {
                let count = seg.accumulate0(k);
                let k = seg.partition_point_acc(count);
                pp!(values.get(k).map_or(-1, |&x| x as i64));
            }
            _ => unreachable!(),
        }
    }
}
