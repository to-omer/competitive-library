use super::{CONVEX_NAIVE_CUTOFF, Signed, assert_finite, min_plus_convolution_naive, output_len};
use std::cmp::Ordering;

pub(super) fn is_convex<T>(values: &[T]) -> bool
where
    T: Signed,
{
    let n = values.len().saturating_sub(2);
    (0..n).step_by(128).all(|start| {
        let end = n.min(start + 128);
        values[start..end + 2]
            .windows(3)
            .fold(true, |valid, w| valid & (w[1] - w[0] <= w[2] - w[1]))
    })
}

fn orient_one_convex<'a, T>(a: &'a [T], b: &'a [T]) -> (&'a [T], &'a [T])
where
    T: Signed,
{
    if is_convex(b) {
        (a, b)
    } else if is_convex(a) {
        (b, a)
    } else {
        panic!("at least one min-plus convolution input must be convex")
    }
}

/// Computes convolution of two convex inputs by merging their slope sequences.
///
/// The running time is `O(n + m)`.
///
/// # Panics
///
/// Panics unless both inputs are finite and convex.
pub fn min_plus_convolution_convex_merge<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    assert_finite(a);
    assert_finite(b);
    assert!(is_convex(a) && is_convex(b), "both inputs must be convex");
    convex_merge(a, b)
}

pub(super) fn convex_merge<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    let (mut i, mut j) = (0, 0);
    const BLOCK: usize = 16;
    let mut result = Vec::with_capacity(len);
    result.push(a[i] + b[j]);
    while a.len() - i > BLOCK && b.len() - j > BLOCK {
        let (x, y) = (&a[i..], &b[j..]);
        let take_a = if x[4] - x[3] <= y[1] - y[0] {
            true
        } else if y[4] - y[3] <= x[1] - x[0] {
            false
        } else {
            let (first, second) = if x[1] - x[0] <= y[1] - y[0] {
                (x, y)
            } else {
                (y, x)
            };
            let alternating = (1..=8).fold(true, |ok, k| {
                let slope = second[k] - second[k - 1];
                ok & (first[k] - first[k - 1] <= slope) & (slope <= first[k + 1] - first[k])
            });
            if alternating {
                result
                    .extend((1..=8).flat_map(|k| [first[k] + second[k - 1], first[k] + second[k]]));
                i += 8;
                j += 8;
                continue;
            }
            for _ in 0..BLOCK {
                if a[i + 1] - a[i] <= b[j + 1] - b[j] {
                    i += 1;
                } else {
                    j += 1;
                }
                result.push(a[i] + b[j]);
            }
            continue;
        };
        let (run, other) = if take_a { (x, y) } else { (y, x) };
        let slope = other[1] - other[0];
        let mut lo = 4;
        let mut hi = BLOCK;
        let last = run.len() - 1;
        while hi < last && run[hi] - run[hi - 1] <= slope {
            lo = hi;
            hi = hi.saturating_mul(2).min(last);
        }
        if run[hi] - run[hi - 1] <= slope {
            lo = hi;
        } else {
            while hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                if run[mid] - run[mid - 1] <= slope {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
        }
        result.extend(run[1..=lo].iter().map(|&v| v + other[0]));
        if take_a {
            i += lo;
        } else {
            j += lo;
        }
    }
    while i + 1 < a.len() && j + 1 < b.len() {
        if a[i + 1] - a[i] <= b[j + 1] - b[j] {
            i += 1;
        } else {
            j += 1;
        }
        result.push(a[i] + b[j]);
    }
    result.extend(a[i + 1..].iter().map(|&value| value + b[j]));
    result.extend(b[j + 1..].iter().map(|&value| a[i] + value));
    result
}

/// Computes convolution when one input is convex using monotone divide and conquer.
///
/// # Panics
///
/// Panics unless both inputs are finite and at least one is convex.
pub fn min_plus_convolution_convex_divide_and_conquer<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    assert_finite(a);
    assert_finite(b);
    let (arbitrary, convex) = orient_one_convex(a, b);
    convex_divide_and_conquer(arbitrary, convex)
}

pub(super) fn convex_divide_and_conquer<T>(arbitrary: &[T], convex: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(arbitrary.len(), convex.len());
    let n = convex.len();
    let ordered = if n == 1 || (convex[n - 2] >= convex[n - 1] && arbitrary.is_sorted()) {
        Some((convex, arbitrary))
    } else if convex[0] <= convex[1] && arbitrary.is_sorted_by(|x, y| x >= y) {
        Some((arbitrary, convex))
    } else {
        None
    };
    if let Some((decreasing, increasing)) = ordered {
        let mut result = Vec::with_capacity(len);
        result.extend(decreasing.iter().map(|&v| v + increasing[0]));
        result.extend(
            increasing[1..]
                .iter()
                .map(|&v| v + decreasing[decreasing.len() - 1]),
        );
        return result;
    }
    if n <= CONVEX_NAIVE_CUTOFF || arbitrary.len() <= 2 {
        return min_plus_convolution_naive(arbitrary, convex);
    }
    let mut result = vec![T::zero(); len];

    let mut problem = ConvexSearch {
        arbitrary,
        convex,
        lower: Vec::new(),
        blocks: Vec::new(),
    };
    if len > ConvexSearch::<T>::TILE && n.min(arbitrary.len()) > 128 {
        let mut previous = T::maximum();
        problem.lower.extend(convex.chunks(32).map(|block| {
            let minimum = *block.iter().min().unwrap();
            std::mem::replace(&mut previous, minimum).min(minimum)
        }));
        problem.blocks.extend(
            arbitrary
                .chunks(32)
                .map(|block| *block.iter().min().unwrap()),
        );
    }
    problem.solve(
        &mut result,
        0,
        0,
        arbitrary.len() - 1,
        &mut Vec::new(),
        &mut Vec::new(),
    );
    result
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MatrixValue<T> {
    Finite(T),
    Infinite,
}

impl<T> Ord for MatrixValue<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Finite(left), Self::Finite(right)) => left.cmp(right),
            (Self::Finite(_), Self::Infinite) => Ordering::Less,
            (Self::Infinite, Self::Finite(_)) => Ordering::Greater,
            (Self::Infinite, Self::Infinite) => Ordering::Equal,
        }
    }
}

impl<T> PartialOrd for MatrixValue<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn smawk<T, F>(rows: usize, cols: usize, cost: &F) -> Vec<usize>
where
    T: Ord,
    F: Fn(usize, usize) -> T,
{
    fn solve<T, F>(rows: &[usize], cols: &[usize], cost: &F, argmins: &mut [usize])
    where
        T: Ord,
        F: Fn(usize, usize) -> T,
    {
        if rows.is_empty() {
            return;
        }
        let mut reduced = Vec::with_capacity(rows.len().min(cols.len()));
        for &col in cols {
            while let Some(&previous) = reduced.last() {
                let row = rows[reduced.len() - 1];
                if cost(row, col) <= cost(row, previous) {
                    reduced.pop();
                } else {
                    break;
                }
            }
            if reduced.len() < rows.len() {
                reduced.push(col);
            }
        }
        let odd_rows: Vec<_> = rows.iter().copied().skip(1).step_by(2).collect();
        solve(&odd_rows, &reduced, cost, argmins);
        let mut lower = 0;
        for row_position in (0..rows.len()).step_by(2) {
            let upper = if row_position + 1 < rows.len() {
                let target = argmins[rows[row_position + 1]];
                lower
                    + reduced[lower..]
                        .iter()
                        .position(|&col| col == target)
                        .expect("SMAWK odd-row minimum must remain in the reduced columns")
            } else {
                reduced.len() - 1
            };
            let row = rows[row_position];
            let mut best = lower;
            for position in lower + 1..=upper {
                if cost(row, reduced[position]) <= cost(row, reduced[best]) {
                    best = position;
                }
            }
            argmins[row] = reduced[best];
            lower = upper;
        }
    }

    let row_indices: Vec<_> = (0..rows).collect();
    let col_indices: Vec<_> = (0..cols).collect();
    let mut argmins = vec![0; rows];
    solve(&row_indices, &col_indices, cost, &mut argmins);
    argmins
}

/// Computes convolution when one input is convex using SMAWK in `O(n + m)`.
///
/// # Panics
///
/// Panics unless both inputs are finite and at least one is convex.
pub fn min_plus_convolution_convex_smawk<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    assert_finite(a);
    assert_finite(b);
    let (arbitrary, convex) = orient_one_convex(a, b);
    convex_smawk(arbitrary, convex)
}

pub(super) fn convex_smawk<T>(arbitrary: &[T], convex: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(arbitrary.len(), convex.len());
    let cost = |row: usize, col: usize| {
        row.checked_sub(col)
            .filter(|&index| index < convex.len())
            .map_or(MatrixValue::Infinite, |index| {
                MatrixValue::Finite(arbitrary[col] + convex[index])
            })
    };
    let argmins = smawk(len, arbitrary.len(), &cost);
    argmins
        .into_iter()
        .enumerate()
        .map(|(row, col)| {
            row.checked_sub(col)
                .filter(|&index| index < convex.len())
                .map(|index| arbitrary[col] + convex[index])
                .expect("SMAWK minimum must be a valid convolution entry")
        })
        .collect()
}

struct ConvexSearch<'a, T> {
    arbitrary: &'a [T],
    convex: &'a [T],
    lower: Vec<T>,
    blocks: Vec<T>,
}

impl<T> ConvexSearch<'_, T>
where
    T: Signed,
{
    const TILE: usize = 8192;

    fn solve(
        &self,
        output: &mut [T],
        row: usize,
        left: usize,
        right: usize,
        columns: &mut Vec<usize>,
        envelope: &mut Vec<(usize, usize)>,
    ) {
        if output.is_empty() {
            return;
        }
        if left == right {
            let bias = self.arbitrary[left];
            for (out, &v) in output.iter_mut().zip(&self.convex[row - left..]) {
                *out = bias + v;
            }
            return;
        }
        if output.len() <= Self::TILE {
            self.fill_envelope(output, row, left, right, columns, envelope);
            return;
        }
        let half = output.len() / 2;
        let mid = row + half;
        let col = self.minimum_at(
            mid,
            left.max(mid.saturating_sub(self.convex.len() - 1)),
            right.min(mid),
        );
        let (before, after) = output.split_at_mut(half);
        self.solve(before, row, left, col, columns, envelope);
        self.solve(after, mid, col, right, columns, envelope);
    }

    fn fill_envelope(
        &self,
        output: &mut [T],
        row: usize,
        left: usize,
        right: usize,
        columns: &mut Vec<usize>,
        envelope: &mut Vec<(usize, usize)>,
    ) {
        let n = self.convex.len();
        let end = row + output.len();
        let last = end - 1;
        let value = |r: usize, col: usize| self.arbitrary[col] + self.convex[r - col];
        columns.clear();
        let mut low = None;
        // At the last row, an older, cheaper column also dominates at every earlier common row.
        for col in left.max(row.saturating_sub(n - 1))..=right.min(last) {
            if last - col >= n {
                columns.push(col);
            } else {
                let v = value(last, col);
                if low.is_none_or(|best| v < best) {
                    low = Some(v);
                    columns.push(col);
                }
            }
        }
        columns.reverse();
        low = None;
        columns.retain(|&col| {
            if col > row {
                return true;
            }
            let v = value(row, col);
            if low.is_some_and(|best| best <= v) {
                return false;
            }
            low = Some(v);
            true
        });

        envelope.clear();
        for &col in columns.iter().rev() {
            let first = row.max(col);
            let mut start = first;
            while let Some(&(previous, from)) = envelope.last() {
                let lo = first.max(from);
                let stop = end.min(previous + n);
                if lo >= stop || value(lo, col) <= value(lo, previous) {
                    if first <= from {
                        envelope.pop();
                        continue;
                    }
                } else if value(stop - 1, col) > value(stop - 1, previous) {
                    start = stop;
                } else {
                    let mut hi = stop - 1;
                    let mut lo = lo;
                    let mut step = 4;
                    while hi - lo > step {
                        let probe = lo + step;
                        if value(probe, col) <= value(probe, previous) {
                            hi = probe;
                            break;
                        }
                        lo = probe;
                        step *= 2;
                    }
                    while hi - lo > 1 {
                        let mid = lo + (hi - lo) / 2;
                        if value(mid, col) <= value(mid, previous) {
                            hi = mid;
                        } else {
                            lo = mid;
                        }
                    }
                    start = hi;
                }
                break;
            }
            if start < end {
                envelope.push((col, start));
            }
        }
        let mut stop = end;
        for &(col, start) in envelope.iter().rev() {
            let bias = self.arbitrary[col];
            for (v, &x) in output[start - row..stop - row]
                .iter_mut()
                .zip(&self.convex[start - col..])
            {
                *v = bias + x;
            }
            stop = start;
        }
    }

    fn minimum_at(&self, row: usize, first: usize, last: usize) -> usize {
        let scan = |start: usize, end: usize| {
            let choices = &self.arbitrary[start..end];
            let offsets = &self.convex[row + 1 - end..=row - start];
            let sums = || {
                choices
                    .iter()
                    .zip(offsets.iter().rev())
                    .map(|(&x, &y)| x + y)
            };
            let best = sums().min().unwrap();
            (start + sums().position(|v| v == best).unwrap(), best)
        };
        if last - first < 128 {
            return scan(first, last + 1).0;
        }
        let head = first / 32 + 1;
        let tail = last / 32;
        let blocks = &self.blocks[head..tail];
        // The reversed kernel interval for a column block spans at most two kernel blocks.
        let lower = &self.lower[row / 32 + 1 - tail..=row / 32 - head];
        let bounds = || blocks.iter().zip(lower.iter().rev()).map(|(&x, &y)| x + y);
        let bound = bounds().min().unwrap();
        let seed = head + bounds().position(|x| x == bound).unwrap();
        let mut result = scan(seed * 32, (seed + 1) * 32);
        for (i, bound) in bounds().enumerate() {
            if bound < result.1 {
                let start = (head + i) * 32;
                let local = scan(start, start + 32);
                if local.1 < result.1 {
                    result = local;
                }
            }
        }
        for local in [scan(first, head * 32), scan(tail * 32, last + 1)] {
            if local.1 < result.1 {
                result = local;
            }
        }
        result.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::min_plus_convolution::min_plus_convolution_naive,
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, sample_usize},
        },
    };

    #[test]
    fn test_convex_algorithms() {
        let mut rng = Xorshift::default();
        let inputs: Vec<_> = exhaustive_sequences([-2i64, 0, 3], 0..=6).collect();
        let convex: Vec<_> = inputs.iter().filter(|input| is_convex(input)).collect();
        let exhaustive = inputs
            .iter()
            .flat_map(|a| convex.iter().map(move |&b| (a.clone(), b.clone())));
        let mut random = Vec::new();
        // Check every pair of small lengths, then cross allocation boundaries.
        let mut lengths: Vec<_> = (0..=32)
            .flat_map(|n| (0..=32).map(move |m| (n, m)))
            .collect();
        for n in sample_usize(&mut rng, 32, 0..=512, 1000) {
            lengths.push((n, rng.random(0usize..=512)));
        }
        for n in [0, 1, 2, 15, 16, 17, 31, 32, 33, 127, 128, 129] {
            lengths.extend([(n, 1025), (1025, n)]);
        }
        for len in [8191, 8192, 8193, 16385] {
            lengths.push((len - 128, 129));
            lengths.push((129, len - 128));
        }
        for (n, m) in lengths {
            let arbitrary: Vec<_> = rng.random_iter(-1000..=1000).take(n).collect();
            let mut slopes: Vec<_> = rng
                .random_iter(-100i64..=100)
                .take(m.saturating_sub(1))
                .collect();
            slopes.sort_unstable();
            let mut structured = Vec::new();
            if m != 0 {
                structured.push(rng.random(-1000..=1000));
            }
            for slope in slopes {
                structured.push(structured.last().unwrap() + slope);
            }
            random.push((arbitrary, structured.clone()));
            let mut other: Vec<_> = rng
                .random_iter(-100i64..=100)
                .take(n.saturating_sub(1))
                .collect();
            other.sort_unstable();
            let mut convex = Vec::new();
            if n != 0 {
                convex.push(rng.random(-1000..=1000));
            }
            for slope in other {
                convex.push(convex.last().unwrap() + slope);
            }
            random.push((convex, structured));
        }
        for (a, b) in exhaustive.chain(random) {
            let expected = min_plus_convolution_naive(&a, &b);
            assert_eq!(
                min_plus_convolution_convex_divide_and_conquer(&a, &b),
                expected,
                "a={a:?}, b={b:?}"
            );
            assert_eq!(
                min_plus_convolution_convex_smawk(&a, &b),
                expected,
                "a={a:?}, b={b:?}"
            );
            if is_convex(&a) {
                assert_eq!(
                    min_plus_convolution_convex_merge(&a, &b),
                    expected,
                    "a={a:?}, b={b:?}"
                );
            }
        }
        let a = [1_000_000_000i32; 33];
        let b = [1_000_000_000; 17];
        assert_eq!(
            min_plus_convolution_convex_merge(&a, &b),
            [2_000_000_000; 49]
        );
        assert_eq!(
            min_plus_convolution_convex_divide_and_conquer(&a, &b),
            [2_000_000_000; 49]
        );
        for n in [15, 16, 17, 31, 32, 33, 129] {
            let a: Vec<i64> = (0..n).map(|i| i * i).collect();
            let b: Vec<i64> = (0..n + 2).map(|i| i * (i + 1)).collect();
            let expected = min_plus_convolution_naive(&a, &b);
            assert_eq!(min_plus_convolution_convex_merge(&a, &b), expected);
            assert_eq!(min_plus_convolution_convex_merge(&b, &a), expected);
        }
    }
}
