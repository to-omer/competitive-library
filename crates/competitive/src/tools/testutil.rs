use super::{RandomSpec, Xorshift};
use std::{
    collections::HashSet,
    ops::{RangeBounds, RangeInclusive},
};

// Enumerates the complete Cartesian power for each requested length.
// The caller chooses the alphabet and exhaustive bound (zero includes the empty sequence).
pub(crate) fn exhaustive_sequences<I>(
    alphabet: I,
    lengths: RangeInclusive<usize>,
) -> impl Iterator<Item = Vec<I::Item>>
where
    I: IntoIterator,
    I::IntoIter: Clone,
{
    let alphabet = alphabet.into_iter();
    let base = alphabet.clone().count();
    lengths.flat_map(move |len| {
        let alphabet = alphabet.clone();
        let count = base.checked_pow(len.try_into().unwrap()).unwrap();
        (0..count).map(move |mut code| {
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(alphabet.clone().nth(code % base).unwrap());
                code /= base;
            }
            values
        })
    })
}

macro_rules! integer_boundary_values {
    ($ty:ty) => {{
        let mut values: Vec<$ty> = vec![<$ty>::MIN, <$ty>::MAX, 0];
        for base in [2, 10] {
            let mut power: $ty = 1;
            loop {
                for value in [power.checked_sub(1), Some(power), power.checked_add(1)]
                    .into_iter()
                    .flatten()
                {
                    values.push(value);
                    if let Some(negative) = (0 as $ty).checked_sub(value) {
                        values.push(negative);
                    }
                }
                if let Some(next) = power.checked_mul(base) {
                    power = next;
                } else {
                    break;
                }
            }
        }
        values.extend([<$ty>::MIN.saturating_add(1), <$ty>::MAX.saturating_sub(1)]);
        values.sort_unstable();
        values.dedup();
        values
    }};
}
pub(crate) use integer_boundary_values;

// Covers every small value, binary/decimal boundaries and the range endpoints before
// adding random values. Random repetitions never replace coverage.
pub(crate) fn sample_usize(
    rng: &mut Xorshift,
    exhaustive_max: usize,
    range: impl DoubleEndedIterator<Item = usize> + RangeBounds<usize> + Clone + RandomSpec<usize>,
    random_cases: usize,
) -> Vec<usize> {
    let min = range.clone().next().unwrap();
    let max = range.clone().next_back().unwrap();
    let mut values: Vec<_> = (min..=exhaustive_max.min(max)).collect();
    values.extend(
        integer_boundary_values!(usize)
            .into_iter()
            .filter(|n| (min..=max).contains(n)),
    );
    values.extend([
        min,
        min.saturating_add(1).min(max),
        max.saturating_sub(1).max(min),
        max,
    ]);
    values.sort_unstable();
    values.dedup();
    values.extend(rng.random_iter(range).take(random_cases));
    values
}

// Each prefix of the integer range supplies a period and a random value range.
// Constants, ordered inputs and sparse deviations are included for every length.
// Repeated lengths add random cases without repeating the deterministic patterns.
pub(crate) fn structured_sequences<T: Clone + Ord>(
    rng: &mut Xorshift,
    range: impl Iterator<Item = T> + RangeBounds<T> + Clone,
    lengths: impl IntoIterator<Item = usize>,
) -> impl Iterator<Item = Vec<T>>
where
    RangeInclusive<T>: RandomSpec<T>,
{
    let min = range.clone().next().unwrap();
    let max = range.clone().last().unwrap();
    let mut seen_lengths = HashSet::new();
    lengths.into_iter().flat_map(move |n| {
        let first = seen_lengths.insert(n);
        let mut cases = Vec::new();
        for (i, value) in range.clone().enumerate() {
            if first {
                cases.push(vec![value.clone(); n]);
                cases.push(range.clone().take(i + 1).cycle().take(n).collect());
            }
            cases.push(rng.random_iter(min.clone()..=value).take(n).collect());
        }
        if first {
            let mut ordered: Vec<_> = range.clone().cycle().take(n).collect();
            ordered.sort_unstable();
            cases.push(ordered.clone());
            ordered.reverse();
            cases.push(ordered);
            if n != 0 && min != max {
                for i in sample_usize(rng, 8, 0..n, 4) {
                    for (background, exception) in [(&min, &max), (&max, &min)] {
                        let mut sparse = vec![background.clone(); n];
                        sparse[i] = exception.clone();
                        cases.push(sparse);
                    }
                }
            }
        }
        cases.sort_unstable();
        cases.dedup();
        cases
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_exhaustive_sequences() {
        for base in 0usize..=4 {
            let alphabet = (0..base).map(|x| char::from(b'a' + x as u8));
            for min_len in 0..=6 {
                for max_len in min_len..=6 {
                    let values: Vec<_> =
                        exhaustive_sequences(alphabet.clone(), min_len..=max_len).collect();
                    let unique: BTreeSet<_> = values.iter().collect();
                    let expected: usize = (min_len..=max_len).map(|n| base.pow(n as u32)).sum();
                    assert_eq!(values.len(), expected);
                    assert_eq!(unique.len(), expected);
                    assert!(
                        values
                            .iter()
                            .all(|xs| (min_len..=max_len).contains(&xs.len())
                                && xs.iter().all(|x| alphabet.clone().any(|y| *x == y)))
                    );
                }
            }
        }
    }

    #[test]
    fn test_sample_usize() {
        let mut rng = Xorshift::default();
        for max in 0..=1024 {
            for min in (0..=max.min(4)).chain([max]) {
                for exhaustive_max in 0..=16 {
                    let required = sample_usize(&mut rng, exhaustive_max, min..=max, 0);
                    let sampled = sample_usize(&mut rng, exhaustive_max, min..=max, 32);
                    assert_eq!(&sampled[..required.len()], required);
                    assert_eq!(sampled.len(), required.len() + 32);
                    assert!(sampled.iter().all(|n| (min..=max).contains(n)));
                    assert!((min..=max.min(exhaustive_max)).all(|n| required.contains(&n)));
                    assert!(required.contains(&min));
                    assert!(required.contains(&max));
                    assert_eq!(
                        sample_usize(&mut rng, exhaustive_max, min..max + 1, 0),
                        required
                    );
                    for base in [2usize, 10] {
                        for power in 0..=max.max(1).ilog(base) {
                            let boundary = base.pow(power);
                            for n in boundary.saturating_sub(1)..=boundary + 1 {
                                if (min..=max).contains(&n) {
                                    assert!(required.contains(&n));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_structured_sequences() {
        for min in -4..=4 {
            for max in min..=4 {
                let mut rng = Xorshift::default();
                let inclusive: Vec<_> = structured_sequences(&mut rng, min..=max, 0..=32).collect();
                let mut rng = Xorshift::default();
                assert_eq!(
                    inclusive,
                    structured_sequences(&mut rng, min..max + 1, 0..=32).collect::<Vec<_>>()
                );
                let cases: BTreeSet<_> = inclusive.into_iter().collect();
                assert!(
                    cases
                        .iter()
                        .all(|xs| xs.len() <= 32 && xs.iter().all(|x| (min..=max).contains(x)))
                );
                for len in 0..=32 {
                    for value in min..=max {
                        assert!(cases.contains(&vec![value; len]));
                        let period: Vec<_> = (0..len)
                            .map(|i| min + i as i32 % (value - min + 1))
                            .collect();
                        assert!(cases.contains(&period));
                    }
                    let mut ordered: Vec<_> =
                        (0..len).map(|i| min + i as i32 % (max - min + 1)).collect();
                    ordered.sort_unstable();
                    assert!(cases.contains(&ordered));
                    ordered.reverse();
                    assert!(cases.contains(&ordered));
                    if len != 0 {
                        for i in sample_usize(&mut rng, 8, 0..len, 0) {
                            for (background, exception) in [(min, max), (max, min)] {
                                let mut sparse = vec![background; len];
                                sparse[i] = exception;
                                assert!(cases.contains(&sparse));
                            }
                        }
                    }
                }
            }
        }
    }
}
