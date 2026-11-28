use super::{
    CONCAVE_NAIVE_CUTOFF, IntBase, Signed, Unsigned, Zero, assert_finite,
    min_plus_convolution_naive, output_len,
};

pub(super) fn is_concave<T>(values: &[T]) -> bool
where
    T: Signed,
{
    let n = values.len().saturating_sub(2);
    (0..n).step_by(128).all(|start| {
        let end = n.min(start + 128);
        values[start..end + 2]
            .windows(3)
            .fold(true, |valid, w| valid & (w[1] - w[0] >= w[2] - w[1]))
    })
}

/// Computes convolution when one finite input is concave using offline envelopes.
///
/// The running time is `O((n + m) log(n + m))`. The arbitrary input may
/// contain `T::maximum()`.
///
/// # Panics
///
/// Panics unless at least one input is finite and concave.
pub fn min_plus_convolution_concave_envelope<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    let (arbitrary, concave) = if !b.iter().any(T::is_maximum) && is_concave(b) {
        (a, b)
    } else if !a.iter().any(T::is_maximum) && is_concave(a) {
        (b, a)
    } else {
        panic!("at least one min-plus convolution input must be finite and concave")
    };
    concave_envelope(arbitrary, concave)
}

pub(super) fn concave_envelope<T>(arbitrary: &[T], concave: &[T]) -> Vec<T>
where
    T: Signed,
{
    if concave.len() == 1 {
        return arbitrary
            .iter()
            .map(|&value| {
                if value.is_maximum() {
                    T::maximum()
                } else {
                    value + concave[0]
                }
            })
            .collect();
    }
    if arbitrary.len() == 1 {
        return if arbitrary[0].is_maximum() {
            vec![T::maximum(); concave.len()]
        } else {
            concave.iter().map(|&value| arbitrary[0] + value).collect()
        };
    }

    let n = concave.len();
    if n <= CONCAVE_NAIVE_CUTOFF || arbitrary.len() == 2 {
        return min_plus_convolution_naive(arbitrary, concave);
    }
    let mut result = vec![T::maximum(); output_len(arbitrary.len(), n)];
    let mut stack = Vec::new();
    let mut ranks = None;
    for (block, values) in arbitrary.chunks(n).enumerate() {
        let output = &mut result[block * n..][..n + values.len() - 1];
        sweep::<false, _>(concave, values, &mut output[..n], &mut stack, &mut ranks);
        sweep::<true, _>(concave, values, &mut output[n..], &mut stack, &mut ranks);
    }
    result
}

/// Computes convolution of two concave inputs from antidiagonal endpoints.
///
/// The running time is `O(n + m)`.
///
/// # Panics
///
/// Panics unless both inputs are finite and concave.
pub fn min_plus_convolution_concave_both<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    assert_finite(a);
    assert_finite(b);
    assert!(
        is_concave(a) && is_concave(b),
        "both inputs must be concave"
    );
    concave_both(a, b)
}

pub(super) fn concave_both<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    let mut result = Vec::with_capacity(len);
    for output in 0..len {
        let first = output.saturating_sub(b.len() - 1);
        let last = output.min(a.len() - 1);
        result.push((a[first] + b[output - first]).min(a[last] + b[output - last]));
    }
    result
}

/// At each row, the newest shift wins a prefix of the remaining rows.
/// Reversing both coordinates gives the same invariant past the end of a block.
fn sweep<const REVERSE: bool, T>(
    concave: &[T],
    values: &[T],
    output: &mut [T],
    stack: &mut Vec<(usize, T, usize)>,
    ranks: &mut Option<SlopeRanks<T>>,
) where
    T: Signed,
{
    let len = output.len();
    let value = |row, column, bias| {
        let offset: usize = row - column;
        let index = if REVERSE {
            concave.len() - 1 - offset
        } else {
            offset
        };
        concave[index] + bias
    };
    let count = len.min(values.len());
    let bias_at = |row: usize| values[if REVERSE { values.len() - 1 - row } else { row }];
    let Some(first) = (0..count).find(|&row| !bias_at(row).is_maximum()) else {
        return;
    };
    let dest = |row| if REVERSE { len - 1 - row } else { row };
    let joining = if REVERSE {
        concave[concave.len() - 1]
    } else {
        concave[0]
    };
    let bias = bias_at(first);
    output[dest(first)] = output[dest(first)].min(joining + bias);
    stack.clear();
    let mut top = (first, bias, len);
    let mut wide = 0;
    let mut crossing =
        |(row, bias): (usize, T), (col, old): (usize, T), mut lo: usize, mut hi: usize| {
            let d = row - col;
            if hi - lo > 32 && d <= 64 {
                if ranks.is_none() {
                    wide += 1;
                    if wide == 64 {
                        *ranks = Some(SlopeRanks::new(concave));
                    }
                }
                if let Some(ranks) = ranks {
                    let (low, high) = ranks.crossing::<REVERSE>(bias, old, d, concave.len() - 1);
                    lo = lo.max(row + low.saturating_sub(d));
                    hi = hi.min(row.saturating_add(high));
                }
            }
            if hi - lo >= 8 {
                let offset = |column| {
                    if REVERSE {
                        concave.len() - 8 - (lo - column)
                    } else {
                        lo - column
                    }
                };
                let a = &concave[offset(row)..][..8];
                let b = &concave[offset(col)..][..8];
                let wins: u8 = a
                    .iter()
                    .zip(b)
                    .map(|(&a, &b)| u8::from(a + bias <= b + old))
                    .sum();
                if wins < 8 {
                    return lo + usize::from(wins);
                }
                lo += 7;
            }
            let mut step = 1;
            while hi - lo > step {
                let probe = lo + step;
                if value(probe, row, bias) > value(probe, col, old) {
                    hi = probe;
                    break;
                }
                lo = probe;
                step = step.saturating_mul(2);
            }
            while hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                if value(mid, row, bias) <= value(mid, col, old) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            hi
        };
    for row in first + 1..count {
        if top.2 == row {
            top = stack.pop().unwrap();
        }
        let current = value(row, top.0, top.1);
        let bias = bias_at(row);
        let best = if !bias.is_maximum() && joining + bias <= current {
            let mut lo = row;
            let mut end = len;
            loop {
                let (col, old, upper) = top;
                if value(upper - 1, row, bias) > value(upper - 1, col, old) {
                    stack.push(top);
                    end = crossing((row, bias), (col, old), lo, upper - 1);
                    break;
                }
                lo = upper - 1;
                let Some(previous) = stack.pop() else { break };
                top = previous;
            }
            top = (row, bias, end);
            joining + bias
        } else {
            current
        };
        output[dest(row)] = output[dest(row)].min(best);
    }

    let mut row = values.len().min(len);
    for (col, bias, end) in std::iter::once(top).chain(stack.drain(..).rev()) {
        if row < end {
            // Only the forward sweep can have rows left after the last insertion.
            for (dest, &v) in output[row..end].iter_mut().zip(&concave[row - col..]) {
                *dest = (*dest).min(v + bias);
            }
            row = end;
        }
    }
}

struct SlopeRanks<T>
where
    T: Signed,
{
    minimum: T,
    maximum: T,
    width: T::Unsigned,
    counts: Vec<usize>,
    distances: [T::Unsigned; 64],
}

impl<T> SlopeRanks<T>
where
    T: Signed,
{
    fn new(a: &[T]) -> Self {
        let minimum = a[a.len() - 1] - a[a.len() - 2];
        let maximum = a[1] - a[0];
        let span = maximum.abs_diff(minimum);
        let shift = if span.is_zero() {
            0
        } else {
            span.ilog2().saturating_sub(12)
        };
        let mut distance = T::zero();
        let mut ranks = Self {
            minimum,
            maximum,
            width: (T::one() + T::one()).unsigned().pow(shift),
            counts: Vec::new(),
            distances: std::array::from_fn(|_| {
                distance += T::one();
                distance.unsigned()
            }),
        };
        ranks.counts.resize(ranks.bucket(maximum) + 2, 0);
        for w in a.windows(2) {
            let index = ranks.bucket(w[1] - w[0]);
            ranks.counts[index] += 1;
        }
        for i in (1..ranks.counts.len()).rev() {
            ranks.counts[i - 1] += ranks.counts[i];
        }
        ranks
    }

    fn bucket(&self, slope: T) -> usize {
        let delta = slope.abs_diff(self.minimum);
        let mut key = if self.width == T::one().unsigned() {
            delta
        } else {
            delta / self.width
        };
        let two = (T::one() + T::one()).unsigned();
        // The key fits 13 bits, including for integer types wider than usize.
        (0..13).fold(0, |index, bit| {
            let digit = usize::from(!key.is_multiple_of(two));
            key /= two;
            index | digit << bit
        })
    }

    fn crossing<const REVERSE: bool>(
        &self,
        new: T,
        old: T,
        d: usize,
        length: usize,
    ) -> (usize, usize) {
        let gap = new.abs_diff(old);
        let magnitude = if new < old {
            gap / self.distances[d - 1]
        } else {
            gap.div_ceil(self.distances[d - 1])
        };
        if magnitude > T::maximum().unsigned() {
            return (0, length);
        }
        let magnitude = magnitude.signed();
        let slope = if (new < old) != REVERSE {
            -magnitude
        } else {
            magnitude
        };
        if slope < self.minimum || slope > self.maximum {
            return (0, length);
        }
        let i = self.bucket(slope);
        // The sum of d consecutive slopes lies between d times the endpoint slopes.
        // A reversed sweep counts slopes <= -ceil(gap/d) in the original kernel.
        if self.width == T::one().unsigned() {
            let rank = self.counts[i + usize::from(REVERSE)];
            let rank = if REVERSE { length - rank } else { rank };
            (rank, rank)
        } else if REVERSE {
            (length - self.counts[i], length - self.counts[i + 1])
        } else {
            (self.counts[i + 1], self.counts[i])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::min_plus_convolution::min_plus_convolution_naive, tools::Xorshift};

    #[test]
    fn test_concave_algorithms_randomly() {
        let mut rng = Xorshift::default();
        for _ in 0..1_000 {
            let arbitrary_len = rng.random(0..=4 * CONCAVE_NAIVE_CUTOFF);
            let concave_len = rng.random(0..=2 * CONCAVE_NAIVE_CUTOFF);
            let arbitrary: Vec<_> = (0..arbitrary_len)
                .map(|_| {
                    if rng.random(0_u64..8) == 0 {
                        i64::MAX
                    } else {
                        rng.random(-50_i64..=50)
                    }
                })
                .collect();
            let mut slopes: Vec<_> = rng
                .random_iter(-20_i64..=20)
                .take(concave_len.saturating_sub(1))
                .collect();
            slopes.sort_unstable_by(|a, b| b.cmp(a));
            let mut concave = Vec::with_capacity(concave_len);
            if concave_len != 0 {
                concave.push(rng.random(-50_i64..=50));
            }
            for slope in slopes {
                concave.push(concave[concave.len() - 1] + slope);
            }
            let other_len: usize = rng.random(0..=32);
            let mut slopes: Vec<_> = rng
                .random_iter(-20_i64..=20)
                .take(other_len.saturating_sub(1))
                .collect();
            slopes.sort_unstable_by(|a, b| b.cmp(a));
            let mut other = Vec::with_capacity(other_len);
            if other_len != 0 {
                other.push(rng.random(-50_i64..=50));
            }
            for slope in slopes {
                other.push(other[other.len() - 1] + slope);
            }
            assert_eq!(
                min_plus_convolution_concave_envelope(&arbitrary, &concave),
                min_plus_convolution_naive(&arbitrary, &concave)
            );
            assert_eq!(
                min_plus_convolution_concave_both(&concave, &other),
                min_plus_convolution_naive(&concave, &other)
            );
        }
        let constant = [1_000_000_000i32; 33];
        assert_eq!(
            min_plus_convolution_concave_envelope(&constant, &constant),
            [2_000_000_000; 65]
        );
        for (n, m) in [
            (1, 4097),
            (2, 4097),
            (4097, 1),
            (4097, 2),
            (31, 1025),
            (32, 1025),
            (33, 1025),
            (63, 511),
            (64, 511),
            (65, 511),
            (1024, 2051),
        ] {
            for scale in [1i64, 65537] {
                let kernel: Vec<_> = (0..n).map(|i| i * (2 * n - i) * scale).collect();
                let mut value = 0;
                let mut arbitrary: Vec<_> = (0..m)
                    .map(|_| {
                        value += rng.random(0..=2 * n * scale);
                        value
                    })
                    .collect();
                arbitrary[m / 3] = i64::MAX;
                for _ in 0..2 {
                    assert_eq!(
                        min_plus_convolution_concave_envelope(&arbitrary, &kernel),
                        min_plus_convolution_naive(&arbitrary, &kernel)
                    );
                    arbitrary.reverse();
                }
                if n == 1024 {
                    // The gap isolates the final two shifts after the slope table has been built.
                    arbitrary.truncate(n as usize);
                    arbitrary.resize(2 * n as usize, i64::MAX);
                    arbitrary.extend([0, (n + 1) * scale + 1]);
                    assert_eq!(
                        min_plus_convolution_concave_envelope(&arbitrary, &kernel),
                        min_plus_convolution_naive(&arbitrary, &kernel)
                    );
                }
            }
        }
    }

    #[test]
    fn test_concave_envelope_exhaustively() {
        let values = [-2_i64, 0, 3, i64::MAX];
        let mut inputs = vec![Vec::new()];
        for _ in 0..3 {
            let prefixes = inputs.clone();
            for prefix in prefixes {
                for &value in &values {
                    let mut input = prefix.clone();
                    input.push(value);
                    inputs.push(input);
                }
            }
        }
        inputs.sort();
        inputs.dedup();
        let concave: Vec<_> = inputs
            .iter()
            .filter(|input| !input.contains(&i64::MAX) && is_concave(input))
            .collect();
        for arbitrary in &inputs {
            for &structured in &concave {
                assert_eq!(
                    min_plus_convolution_concave_envelope(arbitrary, structured),
                    min_plus_convolution_naive(arbitrary, structured)
                );
            }
        }
    }
}
