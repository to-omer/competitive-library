use super::{Signed, assert_finite, convex::is_convex, output_len};

fn validate_witness<T>(values: &[T], witness: &[T], delta: T)
where
    T: Signed,
{
    assert_eq!(
        values.len(),
        witness.len(),
        "near-convex witness must have the same length as its input"
    );
    assert_finite(values);
    assert_finite(witness);
    assert!(
        !delta.is_negative() && is_convex(witness),
        "near-convex delta must be nonnegative and the witness convex"
    );
    assert!(
        values
            .iter()
            .zip(witness)
            .all(|(&value, &lower)| lower <= value && value - lower <= delta),
        "near-convex witness must satisfy witness[i] <= input[i] <= witness[i] + delta"
    );
}

fn convex_convolution_witnesses<T>(a: &[T], b: &[T]) -> (Vec<T>, Vec<usize>)
where
    T: Signed,
{
    let len = a.len() + b.len() - 1;
    let mut values = Vec::with_capacity(len);
    let mut witnesses = Vec::with_capacity(len);
    let (mut i, mut j) = (0, 0);
    loop {
        values.push(a[i] + b[j]);
        witnesses.push(i);
        if i + 1 == a.len() && j + 1 == b.len() {
            break;
        }
        let take_a = if i + 1 == a.len() {
            false
        } else if j + 1 == b.len() {
            true
        } else {
            a[i + 1] - a[i] <= b[j + 1] - b[j]
        };
        if take_a {
            i += 1;
        } else {
            j += 1;
        }
    }
    (values, witnesses)
}

/// Computes exact near-convex convolution by scanning witness-relevant pairs.
///
/// Each witness must be convex and satisfy
/// `witness[i] <= input[i] <= witness[i] + delta`.
///
/// # Panics
///
/// Panics if a witness is invalid or an input is non-finite.
pub fn min_plus_convolution_near_convex_scan<T>(
    a: &[T],
    b: &[T],
    convex_a: &[T],
    convex_b: &[T],
    delta: T,
) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    validate_witness(a, convex_a, delta);
    validate_witness(b, convex_b, delta);
    let (convex_output, witnesses) = convex_convolution_witnesses(convex_a, convex_b);
    let tolerance = delta + delta;
    let relevant = |output: usize, i: usize| {
        convex_a[i] + convex_b[output - i] <= convex_output[output] + tolerance
    };
    let mut result = Vec::with_capacity(len);
    for output in 0..len {
        let first = output.saturating_sub(b.len() - 1);
        let last = output.min(a.len() - 1);
        let witness = witnesses[output];
        let mut low = first;
        let mut high = witness;
        while low < high {
            let middle = low + (high - low) / 2;
            if relevant(output, middle) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        let first_relevant = low;
        low = witness;
        high = last;
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            if relevant(output, middle) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        let mut best = T::maximum();
        for i in first_relevant..=low {
            best = best.min(a[i] + b[output - i]);
        }
        result.push(best);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::min_plus_convolution::min_plus_convolution_naive, tools::Xorshift};

    #[test]
    fn test_near_convex() {
        let mut rng = Xorshift::default();
        for _ in 0..64 {
            let a_len: usize = rng.random(0..=11);
            let b_len: usize = rng.random(0..=11);
            let mut a_slopes: Vec<_> = rng
                .random_iter(-8_i64..=8)
                .take(a_len.saturating_sub(1))
                .collect();
            let mut b_slopes: Vec<_> = rng
                .random_iter(-8_i64..=8)
                .take(b_len.saturating_sub(1))
                .collect();
            a_slopes.sort_unstable();
            b_slopes.sort_unstable();
            let mut convex_a = Vec::with_capacity(a_len);
            let mut convex_b = Vec::with_capacity(b_len);
            if a_len != 0 {
                convex_a.push(rng.random(-8_i64..=8));
            }
            if b_len != 0 {
                convex_b.push(rng.random(-8_i64..=8));
            }
            for slope in a_slopes {
                convex_a.push(convex_a[convex_a.len() - 1] + slope);
            }
            for slope in b_slopes {
                convex_b.push(convex_b[convex_b.len() - 1] + slope);
            }
            let delta = rng.random(0_i64..=5);
            let a: Vec<_> = convex_a
                .iter()
                .map(|&lower| lower + rng.random(0_i64..=delta))
                .collect();
            let b: Vec<_> = convex_b
                .iter()
                .map(|&lower| lower + rng.random(0_i64..=delta))
                .collect();
            assert_eq!(
                min_plus_convolution_near_convex_scan(&a, &b, &convex_a, &convex_b, delta),
                min_plus_convolution_naive(&a, &b)
            );
        }
    }
}
