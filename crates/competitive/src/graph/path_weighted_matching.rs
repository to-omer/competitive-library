use super::{Associative, Magma, RangeMinimumQuery, SegmentTree, Unital, WaveletMatrix, Zero};
use std::{
    cmp::Reverse,
    ops::{Add, Sub},
};

#[derive(Debug, Clone)]
pub struct PathWeightedMatching<T> {
    weights: Vec<T>,
}

impl<T> PathWeightedMatching<T> {
    pub fn from_edge_weights(weights: impl IntoIterator<Item = T>) -> Self
    where
        T: Copy + Zero + Sub<Output = T>,
    {
        let mut vertex = T::zero();
        let vertices = std::iter::once(vertex).chain(weights.into_iter().map(|weight| {
            vertex = weight - vertex;
            vertex
        }));
        Self::from_vertex_weights(vertices)
    }

    /// Edge weights are the sums of adjacent vertex weights.
    pub fn from_vertex_weights(weights: impl IntoIterator<Item = T>) -> Self {
        Self {
            weights: weights.into_iter().collect(),
        }
    }

    /// Returns the maximum weight of a matching with exactly `k` edges in O(n) time.
    pub fn maximum_weight(self, k: usize) -> T
    where
        T: Copy + Ord + Zero + Add<Output = T>,
    {
        assert!(k <= self.weights.len() / 2);
        let mut gains = path_matching_candidates(self.weights, |x| x, |a, b| a + b);
        if k != 0 && k < gains.len() {
            gains.select_nth_unstable_by_key(k, |x| Reverse(*x));
        }
        gains.into_iter().take(k).fold(T::zero(), |a, b| a + b)
    }

    /// Returns the maximum weights for exactly 0, 1, ... edges, including all
    /// feasible sizes, in O(n log n) time. The first entry is zero.
    pub fn maximum_weights(self) -> Vec<T>
    where
        T: Copy + Ord + Zero + Add<Output = T>,
    {
        let mut gains = path_matching_candidates(self.weights, |x| x, |a, b| a + b);
        gains.sort_unstable_by_key(|x| Reverse(*x));
        gains.insert(0, T::zero());
        for i in 1..gains.len() {
            gains[i] = gains[i - 1] + gains[i];
        }
        gains
    }

    /// Returns the maximum weight and sorted edge indices for exactly `k` edges in O(n) time.
    pub fn maximum_weight_matching(self, k: usize) -> (T, Vec<usize>)
    where
        T: Copy + Ord + Zero + Add<Output = T>,
    {
        let n = self.weights.len();
        assert!(k <= n / 2);
        let mut pairs = path_matching_candidates(
            (0..n).collect(),
            |i| self.weights[i],
            |u, v| (self.weights[u] + self.weights[v], u, v),
        );
        if k != 0 && k < pairs.len() {
            pairs.select_nth_unstable_by_key(k, |&(w, u, v)| (Reverse(w), u, v));
        }
        let mut matched = vec![false; n];
        let mut weight = T::zero();
        for (w, u, v) in pairs.into_iter().take(k) {
            weight = weight + w;
            matched[u] = true;
            matched[v] = true;
        }
        let mut edges = Vec::with_capacity(k);
        let mut i = 0;
        while i < n {
            if matched[i] {
                edges.push(i);
                i += 2;
            } else {
                i += 1;
            }
        }
        (weight, edges)
    }
}

/// Returns maximum matching weights for queries `(l, r, k)` in O((n + q) log n) time.
/// Each query selects exactly `k` edges in the vertex interval `[l, r)`.
/// Edge weights are sums of adjacent entries in `weights`; answers follow query order.
pub fn path_weighted_matching_queries<T>(weights: &[T], queries: &[(usize, usize, usize)]) -> Vec<T>
where
    T: Copy + Ord + Zero + Add<Output = T> + Sub<Output = T>,
{
    let n = weights.len();
    let mut prefix = Vec::with_capacity(n + 1);
    prefix.push(T::zero());
    for &weight in weights {
        prefix.push(*prefix.last().unwrap() + weight);
    }
    let mut answers = vec![T::zero(); queries.len()];
    let mut pending = false;
    for (i, &(l, r, k)) in queries.iter().enumerate() {
        assert!(l <= r && r <= n && k <= (r - l) / 2);
        if 2 * k == r - l {
            answers[i] = prefix[r] - prefix[l];
        } else if k != 0 {
            pending = true;
        }
    }
    if !pending {
        return answers;
    }
    let mut pairs = path_matching_candidates(
        (0..n).collect(),
        |i| weights[i],
        |u, v| (weights[u] + weights[v], u, v),
    );
    pairs.sort_unstable_by_key(|&(w, u, v)| (Reverse(w), u, v));
    let mut times = vec![pairs.len(); n];
    for (t, &(_, u, v)) in pairs.iter().enumerate() {
        times[u] = t;
        times[v] = t;
    }
    let times = WaveletMatrix::new(times);
    let mut heads = vec![!0; pairs.len()];
    let mut next = vec![!0; queries.len()];
    for (i, &(l, r, k)) in queries.iter().enumerate() {
        if k != 0 && 2 * k != r - l {
            let t = times.quantile(l..r, 2 * k - 1);
            next[i] = heads[t];
            heads[t] = i;
        }
    }
    let minima: [_; 2] = std::array::from_fn(|parity| {
        RangeMinimumQuery::new(
            (parity..n)
                .step_by(2)
                .map(|i| (weights[i], Reverse(i)))
                .collect(),
        )
    });
    let mut state = SegmentTree::<RangeState<T>>::from_vec(
        weights
            .iter()
            .enumerate()
            .map(|(i, &w)| RangeState::unmatched(w, i))
            .collect(),
    );
    let mut rollback = Vec::with_capacity(6);
    for (t, &(_, u, v)) in pairs.iter().enumerate() {
        let mut query = heads[t];
        while query != !0 {
            let (l, r, k) = queries[query];
            let current = state.fold(l..r);
            let used = r - l - current.free;
            let left = (l - state.fold(..l).free) % 2;
            let right = (left + used) % 2;
            let mut answer = current.sum;
            if left != 0 {
                let first = current.first.unwrap().1;
                let (_, Reverse(i)) = minima[l % 2].fold(l / 2, first.div_ceil(2));
                rollback.push((i, state.get(i)));
                state.set(i, RangeState::unmatched(weights[i], i));
                answer = answer - weights[i];
            }
            if right != 0 {
                let last = current.last.unwrap().1;
                let (_, Reverse(i)) = minima[(r - 1) % 2].fold(last.div_ceil(2), r.div_ceil(2));
                rollback.push((i, state.get(i)));
                state.set(i, RangeState::unmatched(weights[i], i));
                answer = answer - weights[i];
            }
            for _ in 0..(2 * k - used + left + right) / 2 {
                let (gain, Reverse(a), Reverse(b)) = state.fold(l..r).best.unwrap();
                answer = answer + gain;
                for i in [a, b] {
                    rollback.push((i, state.get(i)));
                    state.set(i, RangeState::matched(weights[i]));
                }
            }
            answers[query] = answer;
            while let Some((i, value)) = rollback.pop() {
                state.set(i, value);
            }
            query = next[query];
        }
        state.set(u, RangeState::matched(weights[u]));
        state.set(v, RangeState::matched(weights[v]));
    }
    answers
}

fn path_matching_candidates<V, T, R>(
    mut stack: Vec<V>,
    weight: impl Fn(V) -> T,
    pair: impl Fn(V, V) -> R,
) -> Vec<R>
where
    V: Copy,
    T: Ord,
{
    let mut pairs = Vec::with_capacity(stack.len() / 2 + 1);
    let mut n = 0;
    for i in 0..stack.len() {
        stack[n] = stack[i];
        n += 1;
        while n >= 3 {
            // Equal gains prefer the leftmost pair.
            let local_maximum = weight(stack[n - 3]) >= weight(stack[n - 1])
                && (n == 3 || weight(stack[n - 2]) > weight(stack[n - 4]));
            if !local_maximum {
                break;
            }
            pairs.push(pair(stack[n - 3], stack[n - 2]));
            stack[n - 3] = stack[n - 1];
            n -= 2;
        }
    }
    while n >= 2 {
        pairs.push(pair(stack[n - 2], stack[n - 1]));
        n -= 2;
    }
    pairs
}

#[derive(Clone)]
struct RangeState<T> {
    free: usize,
    sum: T,
    first: Option<(T, usize)>,
    last: Option<(T, usize)>,
    best: Option<(T, Reverse<usize>, Reverse<usize>)>,
}

impl<T> RangeState<T> {
    fn unmatched(weight: T, index: usize) -> Self
    where
        T: Copy + Zero,
    {
        Self {
            free: 1,
            sum: T::zero(),
            first: Some((weight, index)),
            last: Some((weight, index)),
            best: None,
        }
    }

    fn matched(weight: T) -> Self {
        Self {
            free: 0,
            sum: weight,
            first: None,
            last: None,
            best: None,
        }
    }
}

impl<T> Magma for RangeState<T>
where
    T: Copy + Ord + Add<Output = T>,
{
    type T = Self;
    fn operate(x: &Self, y: &Self) -> Self {
        let crossing = x
            .last
            .zip(y.first)
            .map(|((a, i), (b, j))| (a + b, Reverse(i), Reverse(j)));
        Self {
            free: x.free + y.free,
            sum: x.sum + y.sum,
            first: x.first.or(y.first),
            last: y.last.or(x.last),
            best: x.best.max(y.best).max(crossing),
        }
    }
}

impl<T> Unital for RangeState<T>
where
    T: Copy + Ord + Zero + Add<Output = T>,
{
    fn unit() -> Self {
        Self::matched(T::zero())
    }
}

impl<T> Associative for RangeState<T> where T: Copy + Ord + Add<Output = T> {}

#[cfg(test)]
mod tests {
    use super::{PathWeightedMatching, path_weighted_matching_queries};
    use crate::tools::{
        Xorshift,
        testutil::{exhaustive_sequences, sequence_cases},
    };

    fn maximum_weights_by_dp(weights: &[i128]) -> Vec<i128> {
        let maximum_size = weights.len().div_ceil(2);
        let mut previous = vec![None; maximum_size + 1];
        let mut current = vec![None; maximum_size + 1];
        previous[0] = Some(0);
        current[0] = Some(0);
        for (i, &weight) in weights.iter().enumerate() {
            for k in (1..=i / 2 + 1).rev() {
                let take = previous[k - 1].map(|total| total + weight);
                previous[k] = current[k].max(take);
            }
            std::mem::swap(&mut previous, &mut current);
        }
        current.into_iter().map(Option::unwrap).collect()
    }

    fn check_matching(weights: &[i128], matching: PathWeightedMatching<i128>) {
        let expected = maximum_weights_by_dp(weights);
        assert_eq!(matching.clone().maximum_weights(), expected, "{weights:?}");
        for (k, &weight) in expected.iter().enumerate() {
            assert_eq!(
                matching.clone().maximum_weight(k),
                weight,
                "{weights:?}, k={k}"
            );
        }
    }

    #[test]
    fn test_from_edge_weights() {
        let low = i128::from(i64::MIN);
        let high = i128::from(i64::MAX);
        let mut rng = Xorshift::default();
        let cases = exhaustive_sequences(-2i128..=2, 0..=8)
            .chain(exhaustive_sequences([low, -1, 0, 1, high], 0..=6))
            .chain(sequence_cases(&mut rng, low..=high, 0..=256));
        for weights in cases {
            let matching = PathWeightedMatching::from_edge_weights(weights.iter().copied());
            check_matching(&weights, matching);
        }
    }

    #[test]
    fn test_from_vertex_weights() {
        let low = i128::from(i64::MIN);
        let high = i128::from(i64::MAX);
        let mut rng = Xorshift::default();
        let cases = exhaustive_sequences(-1i128..=1, 0..=11)
            .chain(exhaustive_sequences([low, -1, 0, 1, high], 0..=6))
            .chain(sequence_cases(&mut rng, low..=high, 0..=256))
            .chain([
                (0..256).chain([-1]).collect(),
                (0..256).map(|i| if i % 2 == 0 { i } else { -i }).collect(),
            ]);
        for vertices in cases {
            let weights: Vec<_> = vertices.windows(2).map(|pair| pair[0] + pair[1]).collect();
            let mut vertices = vertices.into_iter();
            let matching =
                PathWeightedMatching::from_vertex_weights(std::iter::from_fn(|| vertices.next()));
            check_matching(&weights, matching);
        }
    }

    #[test]
    fn test_maximum_weight_matching() {
        let low = i128::from(i64::MIN);
        let high = i128::from(i64::MAX);
        let mut rng = Xorshift::default();
        let cases = exhaustive_sequences(-1i128..=1, 0..=10)
            .chain(exhaustive_sequences([low, -1, 0, 1, high], 0..=6))
            .chain(sequence_cases(&mut rng, low..=high, 0..=256));
        for vertices in cases {
            let weights: Vec<_> = vertices.windows(2).map(|pair| pair[0] + pair[1]).collect();
            let expected = maximum_weights_by_dp(&weights);
            for matching in [
                PathWeightedMatching::from_vertex_weights(vertices),
                PathWeightedMatching::from_edge_weights(weights.iter().copied()),
            ] {
                for (k, &expected) in expected.iter().enumerate() {
                    let (weight, edges) = matching.clone().maximum_weight_matching(k);
                    assert_eq!(weight, expected, "{weights:?}, k={k}");
                    assert_eq!(edges.len(), k);
                    assert!(edges.iter().all(|&i| i < weights.len()));
                    assert!(edges.windows(2).all(|pair| pair[0] + 1 < pair[1]));
                    let total: i128 = edges.iter().map(|&i| weights[i]).sum();
                    assert_eq!(total, expected, "{weights:?}, k={k}, edges={edges:?}");
                }
            }
        }
    }

    #[test]
    fn test_path_weighted_matching_queries() {
        let low = i128::from(i64::MIN);
        let high = i128::from(i64::MAX);
        let mut rng = Xorshift::default();
        let cases = exhaustive_sequences(-1i128..=1, 0..=8)
            .chain(exhaustive_sequences([low, 0, high], 0..=5))
            .chain(sequence_cases(&mut rng, low..=high, 0..=128))
            .chain([vec![0; 513], vec![low; 513], (0..512).chain([-1]).collect()]);
        for vertices in cases {
            let n = vertices.len();
            let ranges: Vec<_> = if n <= 8 {
                (0..=n).flat_map(|l| (l..=n).map(move |r| (l, r))).collect()
            } else {
                vec![
                    (0, 0),
                    (0, n),
                    (1, n - 1),
                    (0, n / 2),
                    (n / 2, n),
                    (n / 4, 3 * n / 4),
                    (n, n),
                ]
            };
            let mut queries = Vec::new();
            let mut expected = Vec::new();
            for (l, r) in ranges {
                let weights: Vec<_> = vertices[l..r]
                    .windows(2)
                    .map(|pair| pair[0] + pair[1])
                    .collect();
                for (k, weight) in maximum_weights_by_dp(&weights).into_iter().enumerate() {
                    queries.push((l, r, k));
                    expected.push(weight);
                }
            }
            queries.extend_from_within(..);
            expected.extend_from_within(..);
            queries.reverse();
            expected.reverse();
            assert_eq!(
                path_weighted_matching_queries(&vertices, &queries),
                expected,
                "{vertices:?}"
            );
            assert!(path_weighted_matching_queries(&vertices, &[]).is_empty());
        }
    }
}
