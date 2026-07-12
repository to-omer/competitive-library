use super::{Signed, assert_finite, output_len};

fn run_entries<T>(values: &[T]) -> Vec<(usize, T)>
where
    T: Signed,
{
    let mut result = Vec::new();
    for (start, &value) in values.iter().enumerate() {
        if result.last().is_none_or(|&(_, previous)| previous != value) {
            result.push((start, value));
        }
    }
    result
}

/// Computes convolution of same-direction monotone inputs from equal-value runs.
///
/// If the inputs contain `r_a` and `r_b` runs, the running time is
/// `O(r_a * r_b + n + m)`.
///
/// # Panics
///
/// Panics unless both inputs are finite and monotone in the same direction.
pub fn min_plus_convolution_monotone_runs<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    assert_finite(a);
    assert_finite(b);
    let increasing = if a.windows(2).all(|window| window[0] >= window[1])
        && b.windows(2).all(|window| window[0] >= window[1])
    {
        false
    } else if a.windows(2).all(|window| window[0] <= window[1])
        && b.windows(2).all(|window| window[0] <= window[1])
    {
        true
    } else {
        panic!("both inputs must be monotone in the same direction")
    };
    monotone_runs(a, b, increasing)
}

pub(super) fn monotone_runs<T>(a: &[T], b: &[T], increasing: bool) -> Vec<T>
where
    T: Signed,
{
    monotone_runs_from_entries(
        &run_entries(a),
        &run_entries(b),
        a.len(),
        b.len(),
        increasing,
    )
}

pub(super) fn monotone_runs_from_entries<T>(
    a_runs: &[(usize, T)],
    b_runs: &[(usize, T)],
    a_len: usize,
    b_len: usize,
    increasing: bool,
) -> Vec<T>
where
    T: Signed,
{
    let normalize = |runs: &[(usize, T)], len: usize| {
        if increasing {
            runs.iter()
                .enumerate()
                .rev()
                .map(|(index, &(_, value))| {
                    (
                        len - runs.get(index + 1).map_or(len, |&(start, _)| start),
                        value,
                    )
                })
                .collect()
        } else {
            runs.to_vec()
        }
    };
    let a_runs = normalize(a_runs, a_len);
    let b_runs = normalize(b_runs, b_len);
    let len = output_len(a_len, b_len);
    let mut result = vec![T::maximum(); len];
    for &(left_start, left_value) in &a_runs {
        for &(right_start, right_value) in &b_runs {
            let output = left_start + right_start;
            result[output] = result[output].min(left_value + right_value);
        }
    }
    for output in 1..len {
        result[output] = result[output].min(result[output - 1]);
    }
    if increasing {
        result.reverse();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::min_plus_convolution::min_plus_convolution_naive, tools::Xorshift};

    #[test]
    fn test_monotone_runs() {
        let mut rng = Xorshift::default();
        for _ in 0..64 {
            let a_len = rng.random(0..=11);
            let b_len = rng.random(0..=11);
            let mut a = Vec::with_capacity(a_len);
            let mut b = Vec::with_capacity(b_len);
            let increasing = rng.random(0_u64..2) == 0;
            let mut value = rng.random(-20_i64..=20);
            for _ in 0..a_len {
                a.push(value);
                let difference = rng.random(0_i64..3);
                value += if increasing { difference } else { -difference };
            }
            value = rng.random(-20_i64..=20);
            for _ in 0..b_len {
                b.push(value);
                let difference = rng.random(0_i64..3);
                value += if increasing { difference } else { -difference };
            }
            assert_eq!(
                min_plus_convolution_monotone_runs(&a, &b),
                min_plus_convolution_naive(&a, &b)
            );
        }
    }
}
