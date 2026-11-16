use super::BitSet;

/// Counts pairs `l <= i < j < r` with `a[i] > a[j]` for each half-open query `(l, r)`.
pub fn static_range_inversions<T>(a: &[T], queries: &[(usize, usize)]) -> Vec<u64>
where
    T: Ord,
{
    let n = a.len();
    let log = |x: usize| x.max(4).ilog2() as usize;
    // Bounding position blocks keeps column counts in u16 and block sums below 2^32.
    let position_shift = (log(n * n / queries.len().max(1) + n) / 2 - 1).min(31 - log(n));
    inversions_by_blocks(a, queries, (log(n) - 1) / 2, position_shift)
}

fn inversions_by_blocks<T>(
    a: &[T],
    queries: &[(usize, usize)],
    value_shift: usize,
    position_shift: usize,
) -> Vec<u64>
where
    T: Ord,
{
    let n = a.len();
    let mut order: Vec<_> = (0..n as u32).collect();
    // Equal values are ranked by position, so they never form inversions.
    order.sort_unstable_by_key(|&i| (&a[i as usize], i));
    let mut rank = vec![0; n];
    for (r, &i) in order.iter().enumerate() {
        rank[i as usize] = r as u32;
    }
    let mut offsets = vec![0; n + 2];
    for &(l, r) in queries {
        assert!(l <= r && r <= n);
        offsets[r] += 1;
    }
    for r in 0..=n {
        offsets[r + 1] += offsets[r];
    }
    let mut by_right = vec![(0, 0); queries.len()];
    for (i, &(l, r)) in queries.iter().enumerate() {
        offsets[r] -= 1;
        by_right[offsets[r]] = (l as u32, i as u32);
    }
    let value_blocks = (n >> value_shift) + 1;
    let position_blocks = (n >> position_shift) + 1;
    let position_block = 1 << position_shift;
    let mut columns = vec![0u16; (value_blocks + 1) * position_blocks];
    for (i, &x) in rank.iter().enumerate() {
        columns[(x as usize >> value_shift) * position_blocks + (i >> position_shift)] += 1;
    }
    for b in (0..value_blocks).rev() {
        let (current, next) = columns.split_at_mut((b + 1) * position_blocks);
        for (count, &larger) in current[b * position_blocks..].iter_mut().zip(&*next) {
            *count += larger;
        }
    }
    let mut value = vec![0u32; n];
    let mut tag = vec![0u32; value_blocks];
    let mut block_sums = vec![0u32; position_blocks];
    let mut unswept = BitSet::ones(n);
    let mut smaller_after = vec![0; n];
    let mut answers = vec![0; queries.len()];
    let w = |value: &[u32], tag: &[u32], i: usize| {
        value[i].wrapping_add(tag[rank[i] as usize >> value_shift])
    };
    for j in (0..n).rev() {
        for &(l, i) in &by_right[offsets[j + 1]..offsets[j + 2]] {
            let (l, i) = (l as usize, i as usize);
            let offset = l & (position_block - 1);
            answers[i] = if offset * 2 <= position_block {
                let full: u64 = block_sums[..l >> position_shift]
                    .iter()
                    .map(|&s| s as u64)
                    .sum();
                let part: u64 = (l - offset..l).map(|i| w(&value, &tag, i) as u64).sum();
                full + part
            } else {
                let block = (l >> position_shift) + 1;
                let full: u64 = block_sums[..block].iter().map(|&s| s as u64).sum();
                let part: u64 = (l..(block << position_shift).min(n))
                    .map(|i| w(&value, &tag, i) as u64)
                    .sum();
                full - part
            };
        }
        smaller_after[j] = w(&value, &tag, j);
        let x = rank[j] as usize;
        unswept.set(x, false);
        let start = x >> value_shift << value_shift;
        let end = (start + (1 << value_shift)).min(n);
        let (tag_start, ranks, delta) = if x - start < end - x - 1 {
            (x >> value_shift, start..x, -1)
        } else {
            ((x >> value_shift) + 1, x + 1..end, 1)
        };
        for t in &mut tag[tag_start..] {
            *t += 1;
        }
        let row = &columns[tag_start * position_blocks..];
        for (sum, &count) in block_sums[..=j >> position_shift].iter_mut().zip(row) {
            *sum += count as u32;
        }
        for y in unswept.iter_ones_range(ranks) {
            let p = order[y] as usize;
            value[p] = value[p].wrapping_add_signed(delta);
            let sum = &mut block_sums[p >> position_shift];
            *sum = sum.wrapping_add_signed(delta);
        }
    }
    let mut inner = vec![0; n + 1];
    let mut starting = vec![0; n + 1];
    for j in 0..n {
        let after = smaller_after[j] as usize;
        inner[j + 1] = inner[j] + (j + after - rank[j] as usize) as u64;
        starting[j + 1] = starting[j] + after as u64;
    }
    for (answer, &(l, r)) in answers.iter_mut().zip(queries) {
        *answer = *answer + inner[r] - starting[l];
    }
    answers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{WithEmptySegment, Xorshift, testutil::sequence_cases};

    #[test]
    fn test_static_range_inversions() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        // Value blocks of 256 and 512 make the updated ranks span several bitmap words.
        let shifts = [
            (0, 0),
            (1, 3),
            (3, 1),
            (2, 5),
            (5, 2),
            (8, 4),
            (9, 6),
            (4, 9),
            (4, 11),
        ];
        for array in sequence_cases(&mut input_rng, 0u32..=1000, 0..=1000) {
            let n = array.len();
            let mut inversions = vec![vec![0; n + 1]; n + 1];
            for l in (0..n).rev() {
                let mut smaller = 0;
                for r in l + 1..=n {
                    smaller += u64::from(array[l] > array[r - 1]);
                    inversions[l][r] = inversions[l + 1][r] + smaller;
                }
            }
            let queries: Vec<_> = (0..=n)
                .map(|r| (0, r))
                .chain((0..=n).map(|l| (l, n)))
                .chain(rng.random_iter(WithEmptySegment(n)).take(4 * n))
                .collect();
            let expected: Vec<_> = queries.iter().map(|&(l, r)| inversions[l][r]).collect();
            assert_eq!(static_range_inversions(&array, &queries), expected);
            assert!(static_range_inversions(&array, &[]).is_empty());
            for (value_shift, position_shift) in shifts {
                assert_eq!(
                    inversions_by_blocks(&array, &queries, value_shift, position_shift),
                    expected
                );
            }
        }
    }
}
