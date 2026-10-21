//! Digit DP examples using [`Transducer`] and [`TransitionTable`].

use super::{
    ChainTransducer, FibHashMap, FunctionalTransducer, IdentityTransducer, IntersectionTransducer,
    LexicographicalTransducer, SequenceTransducer, Transducer, TransitionTable, VecMap,
};
use crate::{
    algebra::{AddMulOperation, AdditiveOperation},
    num::{Saturating, mint_basic::MInt998244353 as M},
    tools::ToDigitSequence,
    transducer,
};

/// Counts and sums integers in `[lower, upper]` whose base-`radix` digit sum is `k`.
/// Requires `2 <= radix <= 10` and `lower <= upper`. Zero has digit sum 0.
///
/// ```
/// use competitive::data_structure::transducer_examples::digit_sum_count_and_sum;
/// assert_eq!(digit_sum_count_and_sum(2, 2, 0, 31), (10, 124));
/// ```
pub fn digit_sum_count_and_sum(radix: usize, k: usize, lower: usize, upper: usize) -> (u128, u128) {
    let upper = upper.to_digit_sequence_radix(radix);
    let lower = lower.to_digit_sequence_radix_len(radix, upper.len());
    let machine = transducer!(=> || 0usize, |&sum, &digit| {
        (sum + digit <= k).then_some((sum + digit, digit))
    }, |&sum| sum == k)
    .intersection(transducer!((lower <=) & (<= upper)));
    let mut dp =
        machine.dp_with::<(AdditiveOperation<u128>, AdditiveOperation<u128>), _>((1, 0), || {
            VecMap::with_len(4 * (k + 1), |&(sum, ((_, lower), (_, upper)))| {
                4 * sum + 2 * usize::from(lower) + usize::from(upper)
            })
        });
    dp.run_effect(
        upper.len(),
        |_| 0..radix,
        |&(count, sum), (digit, _)| (count, radix as u128 * sum + count * digit as u128),
    )
}

/// Counts positive multiples of `divisor` up to `upper` that contain `required`
/// and use at most `max_distinct` decimal digits. Leading padding zeros are ignored.
/// Requires `1 <= divisor <= 100`, `max_distinct <= 10`, and `required < 10`.
///
/// ```
/// use competitive::data_structure::transducer_examples::count_multiples_with_digits;
/// assert_eq!(count_multiples_with_digits(100, 7, 2, 3), 2); // 35 and 63
/// ```
pub fn count_multiples_with_digits(
    upper: usize,
    divisor: usize,
    max_distinct: u32,
    required: usize,
) -> u64 {
    let upper = upper.to_digit_sequence();
    let machine = transducer!(=> || (false, 0usize, 0usize),
    |&(started, remainder, mask), &digit| {
        let started = started || digit != 0;
        let mask = if started { mask | 1usize << digit } else { mask };
        Some(((started, (10 * remainder + digit) % divisor, mask), ()))
    },
    |&(started, remainder, mask)| {
        started && remainder == 0 && mask.count_ones() <= max_distinct && mask & (1 << required) != 0
    })
    .intersection(transducer!(<= upper));
    machine
        .counting_with(FibHashMap::default)
        .run(upper.len(), |_| 0..10)
}

/// Counts words with digit sum `k` and selects a word for each `(lower, upper, rank)` query.
/// Bounds contain base-`radix` digits, including leading zeros, with equal lengths at most `len`.
/// Requires `2 <= radix <= 10`.
/// Counts saturate at `u128::MAX`; the zero-based `rank` must be smaller. Missing ranks return `None`.
/// Precomputation takes O(len * (k + 1) * radix) time and O(len * (k + 1)) space.
///
/// ```
/// use competitive::data_structure::transducer_examples::digit_sum_queries;
/// let lower = [0, 0, 1, 0, 0];
/// let upper = [0, 1, 1, 1, 1];
/// let answers = digit_sum_queries(2, 2, 5, &[(&lower, &upper, 2)]);
/// assert_eq!(answers, [(5, Some(vec![0, 1, 0, 0, 1]))]);
/// ```
pub fn digit_sum_queries(
    radix: usize,
    k: usize,
    len: usize,
    queries: &[(&[usize], &[usize], u128)],
) -> Vec<(u128, Option<Vec<usize>>)> {
    let machine = transducer!(=> || 0usize, |&sum, &digit| {
        (sum + digit <= k).then_some((sum + digit, ()))
    }, |&sum| sum == k);
    let table = TransitionTable::new(&machine, radix, k + 1);
    let counts = table.suffix_counts::<Saturating<u128>>(len);
    queries
        .iter()
        .map(|&(lower, upper, rank)| {
            (
                counts.count_between(lower, upper).0,
                counts.kth_between(lower, upper, rank),
            )
        })
        .collect()
}

/// Counts length-`len` base-`radix` words whose value is `remainder` modulo `divisor`.
/// Includes leading zeros and returns the count modulo 998244353.
/// Requires `2 <= radix <= 10`, `1 <= divisor <= 100`, and `remainder < divisor`.
/// Takes O(divisor^3 log len) time.
///
/// ```
/// use competitive::data_structure::transducer_examples::count_words_with_remainder;
/// assert_eq!(count_words_with_remainder(2, 3, 0, 3), 3.into()); // 000, 011, 110
/// ```
pub fn count_words_with_remainder(radix: usize, divisor: usize, remainder: usize, len: usize) -> M {
    let machine = transducer!(=> || 0usize, |&state, &digit| {
        Some(((radix * state + digit) % divisor, ()))
    }, |&state| state == remainder);
    let table = TransitionTable::new(&machine, radix, divisor);
    let matrix = table
        .to_matrix::<AddMulOperation<M>>(|_| M::from(1))
        .pow(len);
    (0..table.state_size())
        .filter(|&state| table.is_accepting(state))
        .map(|state| matrix[(0, state)])
        .sum()
}
