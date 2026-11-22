/// Returns a mode and its frequency for each nonempty half-open range.
/// Any of the tied modes may be returned. Empty ranges panic.
pub fn static_range_mode<'a, T>(a: &'a [T], queries: &[(usize, usize)]) -> Vec<(&'a T, usize)>
where
    T: Ord,
{
    if queries.is_empty() {
        return Vec::new();
    }
    let n = a.len();
    let mut ans: Vec<_> = queries
        .iter()
        .map(|&(l, r)| {
            assert!(l < r && r <= n);
            (&a[l], 1)
        })
        .collect();
    let mut order: Vec<_> = (0..n).collect();
    order.sort_unstable_by(|&i, &j| a[i].cmp(&a[j]).then(i.cmp(&j)));
    let mut next = vec![0..0; n];
    let mut prefix = Vec::new();
    let mut offset = 0;
    let mut light_max = 0;
    for run in order.chunk_by(|&i, &j| a[i] == a[j]) {
        let f = run.len();
        // The pair scan reads contiguous positions; prefix queries read two scattered counts.
        if f as u128 * (f - 1) as u128 / 2 > 4 * (n as u128 + queries.len() as u128) {
            prefix.resize(n + 1, 0);
            let mut start = 0;
            for (k, &pos) in run.iter().enumerate() {
                prefix[start..=pos].fill(k);
                start = pos + 1;
            }
            prefix[start..].fill(f);
            for (&(l, r), best) in queries.iter().zip(&mut ans) {
                let count = prefix[r] - prefix[l];
                if count > best.1 {
                    *best = (&a[run[0]], count);
                }
            }
        } else if f > 1 {
            light_max = light_max.max(f);
            for k in 0..f - 1 {
                next[run[k]] = offset + k + 1..offset + f;
            }
        }
        offset += f;
    }
    if light_max > 1 {
        let mut head = vec![usize::MAX; n];
        let mut link = Vec::with_capacity(queries.len());
        for (i, &(l, _)) in queries.iter().enumerate() {
            link.push(std::mem::replace(&mut head[l], i));
        }
        // ends[k] is the earliest last position of k+2 equal light values in this suffix.
        let mut ends = vec![n; light_max - 1];
        for l in (0..n).rev() {
            for (end, &pos) in ends.iter_mut().zip(&order[next[l].clone()]) {
                *end = (*end).min(pos);
            }
            let mut i = head[l];
            while i != usize::MAX {
                let r = queries[i].1;
                let count = ends.partition_point(|&p| p < r) + 1;
                if count > ans[i].1 {
                    ans[i] = (&a[ends[count - 2]], count);
                }
                i = link[i];
            }
        }
    }
    ans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{NotEmptySegment, Xorshift, testutil::exhaustive_sequences};

    #[test]
    fn test_static_range_mode() {
        let mut rng = Xorshift::default();
        let small = exhaustive_sequences(0..3, 0..=10).map(|a| {
            let queries: Vec<_> = (0..a.len())
                .flat_map(|l| (l + 1..=a.len()).map(move |r| (l, r)))
                .collect();
            (a, queries)
        });
        let large = (9usize..=255).map(|n| {
            let sigma = [1, 2, 5, n.isqrt(), n / 2, n][n % 6];
            let mut a: Vec<_> = (0..n).map(|i| i % sigma).collect();
            rng.shuffle(&mut a);
            if n % 7 == 0 {
                a[..n / 2].fill(0);
            }
            let queries = rng.random_iter(NotEmptySegment(n)).take(n).collect();
            (a, queries)
        });
        let boundary = (39..=41).map(|frequency| {
            let a = (0usize..63)
                .map(|i| i.saturating_sub(frequency - 1))
                .collect();
            let queries = (0..132).map(|i| (i % 63, 63)).collect();
            (a, queries)
        });
        for (a, queries) in small.chain(large).chain(boundary) {
            let modes = static_range_mode(&a, &queries);
            assert_eq!(modes.len(), queries.len());
            for ((value, count), (l, r)) in modes.into_iter().zip(queries) {
                let mut freq = vec![0; a.len().max(3)];
                for &x in &a[l..r] {
                    freq[x] += 1;
                }
                assert_eq!(freq[*value], count);
                assert_eq!(count, *freq.iter().max().unwrap());
            }
        }
    }
}
