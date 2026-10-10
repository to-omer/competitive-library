use super::{DaryPrefixSumTreeU32, FibHashMap};

pub fn static_range_count_distinct<T: Eq + std::hash::Hash>(
    a: &[T],
    queries: &[(usize, usize)],
) -> Vec<u32> {
    let end = queries.iter().map(|&(_, r)| r).max().unwrap_or(0);
    let mut offsets = vec![0; end + 2];
    for &(_, r) in queries {
        offsets[r] += 1;
    }
    for r in 0..=end {
        offsets[r + 1] += offsets[r];
    }
    let mut order = vec![(0, 0); queries.len()];
    for (i, &(l, r)) in queries.iter().enumerate() {
        offsets[r] -= 1;
        order[offsets[r]] = (l as u32, i);
    }
    let mut bit = DaryPrefixSumTreeU32::new(end + 1);
    let mut last = FibHashMap::default();
    let mut ans = vec![0; queries.len()];
    for r in 0..=end {
        if r != 0 {
            let previous = last.insert(&a[r - 1], r).unwrap_or(0);
            bit.update(previous, 1);
        }
        for &(l, i) in &order[offsets[r]..offsets[r + 1]] {
            ans[i] = bit.accumulate0(l as usize + 1) - l;
        }
    }
    ans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::testutil::exhaustive_sequences;

    #[derive(Eq, PartialEq, Hash)]
    struct Key(usize);

    #[test]
    fn test_static_range_count_distinct() {
        for values in exhaustive_sequences(0..3, 0..=7) {
            let array: Vec<_> = values.into_iter().map(Key).collect();
            for end in 0..=array.len() {
                let mut queries: Vec<_> = (0..=end)
                    .flat_map(|l| (l..=end).map(move |r| (l, r)))
                    .collect();
                queries.reverse();
                let expected: Vec<_> = queries
                    .iter()
                    .map(|&(l, r)| {
                        array[l..r]
                            .iter()
                            .collect::<std::collections::HashSet<_>>()
                            .len() as u32
                    })
                    .collect();
                assert_eq!(static_range_count_distinct(&array, &queries), expected);
            }
            assert!(static_range_count_distinct(&array, &[]).is_empty());
        }
    }
}
