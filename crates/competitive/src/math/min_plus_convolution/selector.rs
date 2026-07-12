use super::{
    Signed, bounded_requirements_from_extrema, concave, convex, min_plus_convolution_bounded_ntt,
    min_plus_convolution_naive, monotone, output_len, piecewise_linear, sparse,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Algorithm {
    Naive,
    Sparse,
    BoundedNtt,
    ConvexDivideAndConquerLeft,
    ConvexDivideAndConquerRight,
    ConvexMerge,
    ConcaveEnvelopeLeft,
    ConcaveEnvelopeRight,
    ConcaveBoth,
    MonotoneRunsIncreasing,
    MonotoneRunsDecreasing,
    LinearLeft,
    LinearRight,
    PiecewiseLinearLeft,
    PiecewiseLinearRight,
}

struct InputCharacteristics<T> {
    finite_count: usize,
    finite_prefix_len: usize,
    // Finite entries after the first infinity.
    finite_entries: Vec<(usize, T)>,
    // Present for finite inputs while the run count fits the selector cache.
    run_entries: Option<Vec<(usize, T)>>,
    extrema: Option<(T, T)>,
    is_convex: bool,
    is_concave: bool,
    is_nondecreasing: bool,
    is_nonincreasing: bool,
    run_count: usize,
    piece_count: usize,
}

fn analyze<T>(values: &[T]) -> InputCharacteristics<T>
where
    T: Signed,
{
    let Some(&first) = values.first() else {
        return InputCharacteristics {
            finite_count: 0,
            finite_prefix_len: 0,
            finite_entries: Vec::new(),
            run_entries: Some(Vec::new()),
            extrema: None,
            is_convex: true,
            is_concave: true,
            is_nondecreasing: true,
            is_nonincreasing: true,
            run_count: 0,
            piece_count: 0,
        };
    };
    if first.is_maximum() {
        return analyze_with_infinity(values, 0, None);
    }

    let mut minimum = first;
    let mut maximum = first;
    let mut is_convex = true;
    let mut is_concave = true;
    let mut is_nondecreasing = true;
    let mut is_nonincreasing = true;
    let mut run_count = 1;
    let mut run_entries = Some(vec![(0, first)]);
    let mut piece_count = 1;
    let mut previous_value = first;
    let mut previous_slope = None;

    for (index, &value) in values.iter().enumerate().skip(1) {
        if value.is_maximum() {
            return analyze_with_infinity(values, index, Some((minimum, maximum)));
        }
        minimum = minimum.min(value);
        maximum = maximum.max(value);
        is_nondecreasing &= previous_value <= value;
        is_nonincreasing &= previous_value >= value;
        if previous_value != value {
            run_count += 1;
            if let Some(entries) = &mut run_entries {
                if entries.len() == MAX_CACHED_RUNS {
                    run_entries = None;
                } else {
                    entries.push((index, value));
                }
            }
        }
        let slope = value - previous_value;
        if let Some(previous_slope) = previous_slope {
            is_convex &= previous_slope <= slope;
            is_concave &= previous_slope >= slope;
            piece_count += usize::from(previous_slope != slope);
        }
        previous_slope = Some(slope);
        previous_value = value;
    }
    InputCharacteristics {
        finite_count: values.len(),
        finite_prefix_len: values.len(),
        finite_entries: Vec::new(),
        run_entries,
        extrema: Some((minimum, maximum)),
        is_convex,
        is_concave,
        is_nondecreasing,
        is_nonincreasing,
        run_count,
        piece_count,
    }
}

fn analyze_with_infinity<T>(
    values: &[T],
    first_infinity: usize,
    mut extrema: Option<(T, T)>,
) -> InputCharacteristics<T>
where
    T: Signed,
{
    let mut finite_entries = Vec::new();
    for (offset, &value) in values[first_infinity + 1..].iter().enumerate() {
        if !value.is_maximum() {
            finite_entries.push((first_infinity + offset + 1, value));
            extrema = Some(extrema.map_or((value, value), |(minimum, maximum)| {
                (minimum.min(value), maximum.max(value))
            }));
        }
    }
    InputCharacteristics {
        finite_count: first_infinity + finite_entries.len(),
        finite_prefix_len: first_infinity,
        finite_entries,
        run_entries: None,
        extrema,
        is_convex: false,
        is_concave: false,
        is_nondecreasing: false,
        is_nonincreasing: false,
        run_count: 0,
        piece_count: 0,
    }
}

fn scaled_work(factor: u128, work: u128) -> u128 {
    factor.saturating_mul(work)
}

// min_plus/dense: input inspection is above the 15% budget at n=256 and
// below it at n=1024, so keep the conservative power-of-two boundary.
const SMALL_PAIR_COUNT: u128 = 1 << 18;

// min_plus_long/structured measures cached run counts through 4096.
const MAX_CACHED_RUNS: usize = 4096;

fn select_algorithm<T>(
    a_len: usize,
    b_len: usize,
    a_characteristics: &InputCharacteristics<T>,
    b_characteristics: &InputCharacteristics<T>,
) -> Algorithm
where
    T: Signed + TryFrom<usize>,
    T::Unsigned: TryInto<usize>,
{
    let output = a_len.saturating_add(b_len).saturating_sub(1) as u128;
    let mut selected = ((a_len as u128) * (b_len as u128), Algorithm::Naive);
    let mut consider = |work: u128, algorithm| {
        if work < selected.0 {
            selected = (work, algorithm);
        }
    };

    consider(
        // min_plus/sparse: 50% finite inputs are already over 10% faster than
        // the INF-skipping naive scan, while dense pair enumeration loses.
        scaled_work(
            3,
            (a_characteristics.finite_count as u128) * (b_characteristics.finite_count as u128),
        ),
        Algorithm::Sparse,
    );
    if let (Some(a_extrema), Some(b_extrema)) =
        (a_characteristics.extrema, b_characteristics.extrema)
        && let Some(requirements) =
            bounded_requirements_from_extrema(a_len, b_len, a_extrema, b_extrema)
    {
        // min_plus/bounded: small transforms need a wider margin for encoding
        // overhead; at 2^20 and above the NTT wins from a lower work ratio.
        consider(
            scaled_work(
                if requirements.transform_len < 1 << 20 {
                    8
                } else {
                    6
                },
                (requirements.transform_len as u128) * (requirements.transform_len.ilog2() as u128),
            ),
            Algorithm::BoundedNtt,
        );
    }

    if a_characteristics.is_convex && b_characteristics.is_convex {
        consider(output, Algorithm::ConvexMerge);
    } else if a_characteristics.is_convex {
        consider(
            scaled_work(2, output),
            Algorithm::ConvexDivideAndConquerLeft,
        );
    } else if b_characteristics.is_convex {
        consider(
            scaled_work(2, output),
            Algorithm::ConvexDivideAndConquerRight,
        );
    }
    if a_characteristics.is_concave && b_characteristics.is_concave {
        consider(output, Algorithm::ConcaveBoth);
    } else if a_characteristics.is_concave || b_characteristics.is_concave {
        consider(
            scaled_work(8, output.saturating_mul(output.max(1).ilog2() as u128 + 1)),
            if a_characteristics.is_concave {
                Algorithm::ConcaveEnvelopeLeft
            } else {
                Algorithm::ConcaveEnvelopeRight
            },
        );
    }
    let increasing = a_characteristics.is_nondecreasing && b_characteristics.is_nondecreasing;
    let decreasing = a_characteristics.is_nonincreasing && b_characteristics.is_nonincreasing;
    if increasing || decreasing {
        consider(
            scaled_work(
                4,
                a_characteristics
                    .run_count
                    .saturating_mul(b_characteristics.run_count) as u128,
            ) + output,
            if decreasing {
                Algorithm::MonotoneRunsDecreasing
            } else {
                Algorithm::MonotoneRunsIncreasing
            },
        );
    }
    let piecewise = match (
        a_characteristics.finite_count == a_len,
        b_characteristics.finite_count == b_len,
    ) {
        (true, true) if a_characteristics.piece_count < b_characteristics.piece_count => {
            Some((a_characteristics.piece_count, true))
        }
        (true, true) => Some((b_characteristics.piece_count, false)),
        (true, false) => Some((a_characteristics.piece_count, true)),
        (false, true) => Some((b_characteristics.piece_count, false)),
        (false, false) => None,
    };
    if let Some((pieces, structured_is_left)) = piecewise {
        if pieces == 1 {
            consider(
                output,
                if structured_is_left {
                    Algorithm::LinearLeft
                } else {
                    Algorithm::LinearRight
                },
            );
        } else {
            consider(
                scaled_work(4, (pieces as u128).saturating_mul(output)),
                if structured_is_left {
                    Algorithm::PiecewiseLinearLeft
                } else {
                    Algorithm::PiecewiseLinearRight
                },
            );
        }
    }
    selected.1
}

/// Computes min-plus convolution after selecting a deterministic exact method
/// from the observed input structure.
///
/// # Panics
///
/// Panics if the output length does not fit [`usize`]. Arithmetic overflow is
/// the caller's responsibility.
pub fn min_plus_convolution<T>(a: &[T], b: &[T]) -> Vec<T>
where
    T: Signed + TryFrom<usize>,
    T::Unsigned: TryInto<usize>,
{
    if (a.len() as u128) * (b.len() as u128) <= SMALL_PAIR_COUNT {
        return min_plus_convolution_naive(a, b);
    }
    let a_characteristics = analyze(a);
    let distinct_b_characteristics = (!std::ptr::eq(a, b)).then(|| analyze(b));
    let b_characteristics = distinct_b_characteristics
        .as_ref()
        .unwrap_or(&a_characteristics);
    match select_algorithm(a.len(), b.len(), &a_characteristics, b_characteristics) {
        Algorithm::Naive => min_plus_convolution_naive(a, b),
        Algorithm::Sparse => {
            let a_entries = a[..a_characteristics.finite_prefix_len]
                .iter()
                .copied()
                .enumerate()
                .chain(a_characteristics.finite_entries.iter().copied());
            let b_entries = b[..b_characteristics.finite_prefix_len]
                .iter()
                .copied()
                .enumerate()
                .chain(b_characteristics.finite_entries.iter().copied());
            sparse(a_entries, b_entries, output_len(a.len(), b.len()))
        }
        Algorithm::BoundedNtt => min_plus_convolution_bounded_ntt(a, b),
        Algorithm::ConvexDivideAndConquerLeft => convex::convex_divide_and_conquer(b, a),
        Algorithm::ConvexDivideAndConquerRight => convex::convex_divide_and_conquer(a, b),
        Algorithm::ConvexMerge => convex::convex_merge(a, b),
        Algorithm::ConcaveEnvelopeLeft => concave::concave_envelope(b, a),
        Algorithm::ConcaveEnvelopeRight => concave::concave_envelope(a, b),
        Algorithm::ConcaveBoth => concave::concave_both(a, b),
        algorithm @ (Algorithm::MonotoneRunsIncreasing | Algorithm::MonotoneRunsDecreasing) => {
            let increasing = algorithm == Algorithm::MonotoneRunsIncreasing;
            if let (Some(a_runs), Some(b_runs)) = (
                &a_characteristics.run_entries,
                &b_characteristics.run_entries,
            ) {
                monotone::monotone_runs_from_entries(a_runs, b_runs, a.len(), b.len(), increasing)
            } else {
                monotone::monotone_runs(a, b, increasing)
            }
        }
        Algorithm::LinearLeft => piecewise_linear::linear(b, a),
        Algorithm::LinearRight => piecewise_linear::linear(a, b),
        Algorithm::PiecewiseLinearLeft => piecewise_linear::piecewise_linear(b, a),
        Algorithm::PiecewiseLinearRight => piecewise_linear::piecewise_linear(a, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    #[test]
    fn test_selector() {
        const LEN: usize = 1024;

        let mut rng = Xorshift::default();
        let selected =
            |a: &[i64], b: &[i64]| select_algorithm(a.len(), b.len(), &analyze(a), &analyze(b));
        let mut sparse_a = vec![i64::MAX; LEN];
        let mut sparse_b = vec![i64::MAX; LEN];
        for _ in 0..8 {
            let i = rng.random(0..LEN);
            let j = rng.random(0..LEN);
            sparse_a[i] = rng.random(-1_000_i64..=1_000);
            sparse_b[j] = rng.random(-1_000_i64..=1_000);
        }
        assert_eq!(selected(&sparse_a, &sparse_b), Algorithm::Sparse);

        let mut slopes: Vec<_> = rng.random_iter(-100_i64..=100).take(LEN - 1).collect();
        slopes.sort_unstable();
        let mut convex = Vec::with_capacity(LEN);
        convex.push(rng.random(-1_000_i64..=1_000));
        for slope in slopes {
            convex.push(convex[convex.len() - 1] + slope);
        }
        assert_eq!(selected(&convex, &convex), Algorithm::ConvexMerge);
        let arbitrary: Vec<_> = rng
            .random_iter(-1_000_000_000_i64..=1_000_000_000)
            .take(LEN)
            .collect();
        assert_eq!(
            selected(&convex, &arbitrary),
            Algorithm::ConvexDivideAndConquerLeft
        );

        let concave: Vec<_> = convex.iter().map(|&value| -value).collect();
        assert_eq!(selected(&concave, &concave), Algorithm::ConcaveBoth);
        assert_eq!(
            selected(&concave, &arbitrary),
            Algorithm::ConcaveEnvelopeLeft
        );

        let bounded_a: Vec<_> = rng.random_iter(0_i64..=1).take(4096).collect();
        let bounded_b: Vec<_> = rng.random_iter(0_i64..=1).take(4096).collect();
        assert_eq!(selected(&bounded_a, &bounded_b), Algorithm::BoundedNtt);

        let mut run_values = Vec::with_capacity(8);
        let mut value = rng.random(-1_000_i64..=1_000);
        for _ in 0..8 {
            run_values.push(value);
            value += rng.random(1_i64..=10);
        }
        let monotone: Vec<_> = (0..LEN).map(|i| run_values[i * 8 / LEN]).collect();
        let low = rng.random(1_i64..=5);
        let high = rng.random(6_i64..=10);
        let mut irregular = Vec::with_capacity(LEN);
        value = rng.random(-1_000_i64..=1_000);
        for i in 0..LEN {
            irregular.push(value);
            value += if i % 2 == 0 { low } else { high };
        }
        assert_eq!(
            selected(&monotone, &irregular),
            Algorithm::MonotoneRunsIncreasing
        );

        let start = rng.random(-1_000_i64..=1_000);
        let slope = rng.random(-20_i64..=20);
        let linear: Vec<_> = (0..LEN).map(|i| start + slope * i as i64).collect();
        assert_eq!(selected(&arbitrary, &linear), Algorithm::LinearRight);

        let piece_slopes = [
            rng.random(-20_i64..=-11),
            rng.random(11_i64..=20),
            rng.random(-10_i64..=-1),
            rng.random(1_i64..=10),
        ];
        let mut piecewise = Vec::with_capacity(LEN);
        value = rng.random(-1_000_i64..=1_000);
        for i in 0..LEN {
            piecewise.push(value);
            value += piece_slopes[i * piece_slopes.len() / LEN];
        }
        assert_eq!(
            selected(&arbitrary, &piecewise),
            Algorithm::PiecewiseLinearRight
        );

        assert_eq!(selected(&irregular, &irregular), Algorithm::Naive);
    }
}
