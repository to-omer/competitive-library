use super::{Signed, assert_finite, output_len};
use std::{
    cmp::Ordering,
    ops::{Range, RangeInclusive},
};

pub(super) fn is_convex<T>(values: &[T]) -> bool
where
    T: Signed,
{
    values
        .windows(3)
        .all(|window| window[1] - window[0] <= window[2] - window[1])
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
    let mut a_slopes = a.windows(2).map(|window| window[1] - window[0]);
    let mut b_slopes = b.windows(2).map(|window| window[1] - window[0]);
    let mut next_a = a_slopes.next();
    let mut next_b = b_slopes.next();
    let mut current = a[0] + b[0];
    let mut result = Vec::with_capacity(len);
    result.push(current);
    while next_a.is_some() || next_b.is_some() {
        let slope = match (next_a, next_b) {
            (Some(left), Some(right)) if left <= right => {
                next_a = a_slopes.next();
                left
            }
            (Some(_), Some(right)) => {
                next_b = b_slopes.next();
                right
            }
            (Some(left), None) => {
                next_a = a_slopes.next();
                left
            }
            (None, Some(right)) => {
                next_b = b_slopes.next();
                right
            }
            (None, None) => break,
        };
        current += slope;
        result.push(current);
    }
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
    let mut result = vec![T::zero(); len];

    fn solve<T>(
        arbitrary: &[T],
        convex: &[T],
        result: &mut [T],
        rows: Range<usize>,
        options: RangeInclusive<usize>,
    ) where
        T: Signed,
    {
        if rows.is_empty() {
            return;
        }
        let row = (rows.start + rows.end) / 2;
        let first = (*options.start()).max(row.saturating_sub(convex.len() - 1));
        let last = (*options.end()).min(row).min(arbitrary.len() - 1);
        let mut best_col = first;
        let mut best_value = arbitrary[first] + convex[row - first];
        for col in first + 1..=last {
            let value = arbitrary[col] + convex[row - col];
            if value < best_value {
                best_value = value;
                best_col = col;
            }
        }
        result[row] = best_value;
        solve(
            arbitrary,
            convex,
            result,
            rows.start..row,
            *options.start()..=best_col,
        );
        solve(
            arbitrary,
            convex,
            result,
            row + 1..rows.end,
            best_col..=*options.end(),
        );
    }

    solve(
        arbitrary,
        convex,
        &mut result,
        0..len,
        0..=arbitrary.len() - 1,
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
    }
}
