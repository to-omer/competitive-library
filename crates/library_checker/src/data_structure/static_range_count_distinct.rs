use competitive::data_structure::{DaryPrefixSumTreeU32, FibHashMap};
use competitive::prelude::*;

#[verify::library_checker("static_range_count_distinct")]
pub fn static_range_count_distinct(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, a: [u32; n], queries: [(usize, usize); q]);
    let end = queries.iter().map(|&(_, r)| r).max().unwrap_or(0);
    if end == 0 {
        pp!(@lf @it std::iter::repeat_n(0u32, q));
        return;
    }
    let mut offsets = vec![0; end + 2];
    for &(_, r) in &queries {
        offsets[r] += 1;
    }
    for r in 0..=end {
        offsets[r + 1] += offsets[r];
    }
    let mut order = vec![(0, 0); q];
    for (i, (l, r)) in queries.into_iter().enumerate() {
        offsets[r] -= 1;
        order[offsets[r]] = (l as u32, i as u32);
    }
    let mut bit = DaryPrefixSumTreeU32::new(end + 1);
    let mut last = FibHashMap::default();
    let mut ans = vec![0; q];
    for r in 0..=end {
        if r != 0 {
            let previous = last.insert(a[r - 1], r).unwrap_or(0);
            bit.update(previous, 1);
        }
        for &(l, i) in &order[offsets[r]..offsets[r + 1]] {
            ans[i as usize] = bit.accumulate0(l as usize + 1) - l;
        }
    }
    pp!(@lf @it ans);
}
