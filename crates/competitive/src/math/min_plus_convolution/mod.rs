//! Exact min-plus convolution algorithms for structured integer sequences.
//!
//! `T::maximum()` represents positive infinity. Callers must choose a signed
//! integer type that can represent every finite result and every intermediate
//! arithmetic expression used by the selected algorithm.

use super::{Convolve998244353, ConvolveSteps, Signed, montgomery::MInt998244353};

pub use self::concave::{min_plus_convolution_concave_both, min_plus_convolution_concave_envelope};
pub use self::convex::{
    min_plus_convolution_convex_divide_and_conquer, min_plus_convolution_convex_merge,
    min_plus_convolution_convex_smawk,
};
pub use self::monotone::min_plus_convolution_monotone_runs;
pub use self::near_convex::min_plus_convolution_near_convex_scan;
pub use self::piecewise_linear::{
    min_plus_convolution_linear, min_plus_convolution_piecewise_linear,
};
pub use self::selector::min_plus_convolution;
pub use self::squared_distance::min_plus_convolution_with_squared_distance;

mod concave;
mod convex;
mod monotone;
mod near_convex;
mod piecewise_linear;
mod selector;
mod squared_distance;

pub(super) fn output_len(a_len: usize, b_len: usize) -> usize {
    if a_len == 0 || b_len == 0 {
        return 0;
    }
    a_len
        .checked_add(b_len)
        .and_then(|len| len.checked_sub(1))
        .expect("min-plus convolution output length must fit usize")
}

pub(super) fn assert_finite<T>(values: &[T])
where
    T: Signed,
{
    assert!(
        !values.iter().any(T::is_maximum),
        "min-plus convolution algorithm requires finite input values"
    );
}

/// Computes min-plus convolution by enumerating all input pairs.
///
/// # Panics
///
/// Panics if the output length does not fit [`usize`]. Arithmetic overflow is
/// the caller's responsibility.
pub fn min_plus_convolution_naive<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    let (outer, inner) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let inner_is_finite = !inner.iter().any(T::is_maximum);
    let mut result = vec![T::maximum(); len];
    for (index, &left) in outer.iter().enumerate() {
        if left.is_maximum() {
            continue;
        }
        let output = &mut result[index..index + inner.len()];
        if inner_is_finite {
            for (slot, &right) in output.iter_mut().zip(inner) {
                *slot = (*slot).min(left + right);
            }
        } else {
            for (slot, &right) in output.iter_mut().zip(inner) {
                if !right.is_maximum() {
                    *slot = (*slot).min(left + right);
                }
            }
        }
    }
    result
}

/// Computes min-plus convolution by enumerating finite input pairs only.
///
/// If the inputs have `s_a` and `s_b` finite values, the running time is
/// `O(s_a * s_b + n + m)`.
///
/// # Panics
///
/// Panics if the output length does not fit [`usize`]. Arithmetic overflow is
/// the caller's responsibility.
pub fn min_plus_convolution_sparse<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed,
{
    let len = output_len(a.len(), b.len());
    if len == 0 {
        return Vec::new();
    }
    let a: Vec<_> = a
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| !value.is_maximum())
        .collect();
    let b: Vec<_> = b
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| !value.is_maximum())
        .collect();
    sparse(a.iter().copied(), b.iter().copied(), len)
}

fn sparse<T>(
    a: impl IntoIterator<Item = (usize, T)>,
    b: impl IntoIterator<Item = (usize, T)> + Clone,
    len: usize,
) -> Vec<T>
where
    T: Signed,
{
    let mut result = vec![T::maximum(); len];
    for (i, left) in a {
        for (j, right) in b.clone() {
            result[i + j] = result[i + j].min(left + right);
        }
    }
    result
}

const MAX_NTT_SIZE: usize = 1 << 23;

#[derive(Clone, Copy, Debug)]
struct BoundedRequirements<T> {
    a_min: T,
    b_min: T,
    base: usize,
    transform_len: usize,
}

fn finite_extrema<T>(values: &[T]) -> Option<(T, T)>
where
    T: Signed,
{
    let mut finite = values.iter().copied().filter(|value| !value.is_maximum());
    let first = finite.next()?;
    Some(finite.fold((first, first), |(minimum, maximum), value| {
        (minimum.min(value), maximum.max(value))
    }))
}

fn bounded_transform_len(a_len: usize, b_len: usize, base: usize) -> Option<usize> {
    let left_len = a_len.checked_mul(base)?;
    let right_len = b_len.checked_mul(base)?;
    let coefficient_len = left_len
        .checked_add(right_len)
        .and_then(|len| len.checked_sub(1))?;
    let transform_len = coefficient_len.checked_next_power_of_two()?;
    (transform_len <= MAX_NTT_SIZE).then_some(transform_len)
}

fn bounded_requirements_from_extrema<T>(
    a_len: usize,
    b_len: usize,
    (a_min, a_max): (T, T),
    (b_min, b_max): (T, T),
) -> Option<BoundedRequirements<T>>
where
    T: Signed,
    T::Unsigned: TryInto<usize>,
{
    let a_span = a_max.abs_diff(a_min).try_into().ok()?;
    let b_span = b_max.abs_diff(b_min).try_into().ok()?;
    let base = a_span
        .checked_add(b_span)
        .and_then(|span| span.checked_add(1))?;
    let transform_len = bounded_transform_len(a_len, b_len, base)?;
    Some(BoundedRequirements {
        a_min,
        b_min,
        base,
        transform_len,
    })
}

/// Computes exact min-plus convolution for small integer value spans using NTT.
///
/// # Panics
///
/// Panics if the encoded range cannot be represented or requires a transform
/// longer than `2^23`. Arithmetic overflow is the caller's responsibility.
pub fn min_plus_convolution_bounded_ntt<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
    T::Unsigned: TryInto<usize>,
{
    let output_len = output_len(a.len(), b.len());
    if output_len == 0 {
        return Vec::new();
    }
    let (Some(a_extrema), Some(b_extrema)) = (finite_extrema(a), finite_extrema(b)) else {
        return vec![T::maximum(); output_len];
    };
    let requirements = bounded_requirements_from_extrema(a.len(), b.len(), a_extrema, b_extrema)
        .expect("bounded min-plus convolution encoding must fit the 2^23 NTT limit");
    let mut left = vec![MInt998244353::from(0_u32); a.len() * requirements.base];
    let mut right = vec![MInt998244353::from(0_u32); b.len() * requirements.base];
    for (index, &value) in a.iter().enumerate() {
        if !value.is_maximum() {
            let normalized: usize = value
                .abs_diff(requirements.a_min)
                .try_into()
                .ok()
                .expect("bounded min-plus convolution value span must fit usize");
            left[index * requirements.base + normalized] = MInt998244353::from(1_u32);
        }
    }
    for (index, &value) in b.iter().enumerate() {
        if !value.is_maximum() {
            let normalized: usize = value
                .abs_diff(requirements.b_min)
                .try_into()
                .ok()
                .expect("bounded min-plus convolution value span must fit usize");
            right[index * requirements.base + normalized] = MInt998244353::from(1_u32);
        }
    }
    let coefficients = Convolve998244353::convolve(left, right);
    let encoded_len = output_len
        .checked_mul(requirements.base)
        .expect("bounded min-plus convolution encoded length must fit usize");
    let mut result = Vec::with_capacity(output_len);
    for chunk in coefficients[..encoded_len].chunks_exact(requirements.base) {
        let value = if let Some(normalized) = chunk.iter().position(|&value| u32::from(value) != 0)
        {
            requirements.a_min
                + requirements.b_min
                + T::try_from(normalized)
                    .ok()
                    .expect("bounded min-plus convolution value must fit the output type")
        } else {
            T::maximum()
        };
        result.push(value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_min_plus_convolution() {
        let inf = i64::MAX;
        let mut rng = Xorshift::default();
        for a_len in 0..=8 {
            for b_len in 0..=8 {
                for case in 0..32 {
                    let mut values = |len, all_infinite| {
                        let mut values = Vec::with_capacity(len);
                        for _ in 0..len {
                            values.push(if all_infinite || rng.random(0_u64..5) == 0 {
                                inf
                            } else {
                                rng.random(-4_i64..=4)
                            });
                        }
                        values
                    };
                    let a = values(a_len, case <= 1);
                    let b = values(b_len, case == 0 || case == 2);
                    let mut expected = if a.is_empty() || b.is_empty() {
                        Vec::new()
                    } else {
                        vec![inf; a.len() + b.len() - 1]
                    };
                    for (i, &left) in a.iter().enumerate() {
                        if left == inf {
                            continue;
                        }
                        for (j, &right) in b.iter().enumerate() {
                            if right != inf {
                                expected[i + j] = expected[i + j].min(left + right);
                            }
                        }
                    }
                    assert_eq!(min_plus_convolution_naive(&a, &b), expected);
                    assert_eq!(min_plus_convolution_sparse(&a, &b), expected);
                    assert_eq!(min_plus_convolution_bounded_ntt(&a, &b), expected);
                    assert_eq!(min_plus_convolution(&a, &b), expected);
                }
            }
        }
    }

    #[test]
    fn test_automatic_selection() {
        let mut rng = Xorshift::default();
        for case in 0..30 {
            let a_len = rng.random(520..=640);
            let b_len = rng.random(520..=640);
            let (a, b): (Vec<i64>, Vec<i64>) = match case % 5 {
                0 => {
                    let mut a = vec![i64::MAX; a_len];
                    let mut b = vec![i64::MAX; b_len];
                    let a_prefix = rng.random(1..=8);
                    let b_prefix = rng.random(1..=8);
                    for value in &mut a[..a_prefix] {
                        *value = rng.random(-1_000_i64..=1_000);
                    }
                    for value in &mut b[..b_prefix] {
                        *value = rng.random(-1_000_i64..=1_000);
                    }
                    a[a_len - 1] = rng.random(-1_000_i64..=1_000);
                    b[b_len - 1] = rng.random(-1_000_i64..=1_000);
                    for _ in 0..8 {
                        let i = rng.random(0..a_len);
                        let j = rng.random(0..b_len);
                        a[i] = rng.random(-1_000_i64..=1_000);
                        b[j] = rng.random(-1_000_i64..=1_000);
                    }
                    (a, b)
                }
                1 => (
                    rng.random_iter(-2_i64..=2).take(a_len).collect(),
                    rng.random_iter(-2_i64..=2).take(b_len).collect(),
                ),
                2 | 3 => {
                    let len = if case % 5 == 2 { a_len } else { b_len };
                    let mut slope = rng.random(-20_i64..=20);
                    let mut value = rng.random(-1_000_i64..=1_000);
                    let mut structured = Vec::with_capacity(len);
                    for _ in 0..len {
                        structured.push(value);
                        value += slope;
                        slope += if case % 5 == 2 {
                            rng.random(0_i64..=3)
                        } else {
                            -rng.random(0_i64..=3)
                        };
                    }
                    if case % 5 == 2 {
                        (
                            structured,
                            rng.random_iter(-1_000_i64..=1_000).take(b_len).collect(),
                        )
                    } else {
                        (
                            rng.random_iter(-1_000_i64..=1_000).take(a_len).collect(),
                            structured,
                        )
                    }
                }
                _ => {
                    let mut a = Vec::with_capacity(a_len);
                    let mut b = Vec::with_capacity(b_len);
                    let mut left = rng.random(-1_000_i64..=1_000);
                    let mut right = rng.random(-1_000_i64..=1_000);
                    for _ in 0..a_len {
                        a.push(left);
                        left += rng.random(0_i64..=3);
                    }
                    for _ in 0..b_len {
                        b.push(right);
                        right += rng.random(0_i64..=3);
                    }
                    (a, b)
                }
            };
            let expected = min_plus_convolution_naive(&a, &b);
            assert_eq!(min_plus_convolution(&a, &b), expected);
            assert_eq!(min_plus_convolution(&b, &a), expected);
        }
    }
}
