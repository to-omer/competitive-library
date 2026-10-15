use super::{RandomSpec, Xorshift};
use crate::{
    algebra::{
        ConcatenateOperation, LastOperation, LazyMapAct, LazyMapAgg, LazyMapMonoid,
        LazyMapMonoidWithKeyAct, MonoidAct, UpdateAct,
    },
    num::IntBase,
};
use std::ops::RangeInclusive;

// Enumerates the complete Cartesian power for each requested length.
// The caller chooses the alphabet and exhaustive bound (zero includes the empty sequence).
pub fn exhaustive_sequences<I>(
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

/// Binary and decimal boundaries, range endpoints, and their neighbors.
pub fn integer_cases<T>(range: RangeInclusive<T>) -> impl Iterator<Item = T>
where
    T: IntBase + TryFrom<i128> + TryFrom<u128>,
{
    assert!(!range.is_empty());
    let min = *range.start();
    let max = *range.end();
    let mut values: Vec<_> = integer_boundary_values!(i128)
        .into_iter()
        .filter_map(|value| T::try_from(value).ok())
        .chain(
            integer_boundary_values!(u128)
                .into_iter()
                .filter_map(|value| T::try_from(value).ok()),
        )
        .filter(|value| range.contains(value))
        .collect();
    values.extend([min, max]);
    if min != max {
        values.extend([min + T::one(), max - T::one()]);
    }
    values.sort_unstable();
    values.dedup();
    values.into_iter()
}

/// The Cartesian product of the range's boundary values.
pub fn integer_pairs<T>(range: RangeInclusive<T>) -> impl Iterator<Item = (T, T)>
where
    T: IntBase + TryFrom<i128> + TryFrom<u128>,
{
    let values: Vec<_> = integer_cases(range).collect();
    let n = values.len();
    (0..n * n).map(move |index| (values[index / n], values[index % n]))
}

/// Includes boundaries and one sample of each applicable structure per binary length layer.
/// Sequence contents are generated as the iterator advances.
pub fn sequence_cases<T>(
    rng: &mut Xorshift,
    values: RangeInclusive<T>,
    lengths: RangeInclusive<usize>,
) -> impl Iterator<Item = Vec<T>>
where
    T: IntBase + TryFrom<i128> + TryFrom<u128>,
    RangeInclusive<T>: RandomSpec<T>,
{
    assert!(!values.is_empty());
    assert!(!lengths.is_empty());
    let (min, max) = (*lengths.start(), *lengths.end());
    let mut boundaries = Vec::new();
    for boundary in [min, max]
        .into_iter()
        .chain((0..usize::BITS).map(|bit| 1usize << bit))
    {
        boundaries.extend(
            [
                boundary.checked_sub(1),
                Some(boundary),
                boundary.checked_add(1),
            ]
            .into_iter()
            .flatten()
            .filter(|n| lengths.contains(n)),
        );
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let single_value = values.start() == values.end();
    let mut required = Vec::new();
    for n in boundaries {
        if n == 0 {
            required.push((0..=0, SequenceShape::Constant(None)));
        } else if n != 1 {
            let shape = if single_value {
                SequenceShape::Constant(None)
            } else {
                SequenceShape::Dense
            };
            required.push((n..=n, shape));
        }
    }
    let value_length = min.max(1);
    if value_length <= max {
        for value in integer_cases(values.clone()) {
            required.push((
                value_length..=value_length,
                SequenceShape::Constant(Some(value)),
            ));
        }
        if max >= 2 && max != value_length {
            required.push((max..=max, SequenceShape::Constant(Some(*values.start()))));
            if !single_value {
                required.push((max..=max, SequenceShape::Constant(Some(*values.end()))));
            }
        }
    }
    let shapes = [
        SequenceShape::Dense,
        SequenceShape::Narrow,
        SequenceShape::Constant(None),
        SequenceShape::Periodic,
        SequenceShape::Ascending,
        SequenceShape::Descending,
        SequenceShape::SparseFirst(false),
        SequenceShape::SparseFirst(true),
        SequenceShape::SparseLast(false),
        SequenceShape::SparseLast(true),
        SequenceShape::Sparse,
        SequenceShape::SparseMany,
    ];
    let layers = (0..usize::BITS).filter_map(move |bit| {
        let low = 1usize << bit;
        let high = low.checked_mul(2).map_or(usize::MAX, |next| next - 1);
        let start = low.max(min);
        let end = high.min(max);
        (start <= end).then_some(start..=end)
    });
    let structured = layers.flat_map(move |layer| {
        shapes.into_iter().filter_map(move |shape| {
            if single_value && !matches!(shape, SequenceShape::Constant(_)) {
                return None;
            }
            let start = (*layer.start()).max(shape.minimum_len());
            let end = *layer.end();
            (start <= end).then_some((start..=end, shape))
        })
    });
    required
        .into_iter()
        .chain(structured)
        .map(move |(lengths, shape)| generate_sequence(rng, &values, lengths, shape))
}

pub struct ConcatUpdate;

impl LazyMapMonoid for ConcatUpdate {
    type Key = i64;
    type AggMonoid = ConcatenateOperation<i64>;
    type ActMonoid = LastOperation<i64>;

    fn single_agg(key: &Self::Key) -> LazyMapAgg<Self> {
        vec![*key]
    }

    fn toggle(agg: &mut LazyMapAgg<Self>) {
        agg.reverse();
    }

    fn act_agg(agg: &LazyMapAgg<Self>, act: &LazyMapAct<Self>) -> Option<LazyMapAgg<Self>> {
        Some(act.map_or_else(|| agg.clone(), |value| vec![value; agg.len()]))
    }
}
impl LazyMapMonoidWithKeyAct for ConcatUpdate {
    fn act_key(key: &Self::Key, act: &LazyMapAct<Self>) -> Self::Key {
        UpdateAct::act(key, act)
    }
}

// Covers every small value, binary/decimal boundaries and the range endpoints before
// adding random values. Random repetitions never replace coverage.
pub fn sample_usize(
    rng: &mut Xorshift,
    exhaustive_max: usize,
    range: impl DoubleEndedIterator<Item = usize> + Clone + RandomSpec<usize>,
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

fn generate_sequence<T>(
    rng: &mut Xorshift,
    range: &RangeInclusive<T>,
    lengths: RangeInclusive<usize>,
    shape: SequenceShape<T>,
) -> Vec<T>
where
    T: IntBase,
    RangeInclusive<T>: RandomSpec<T>,
{
    let n = if lengths.start() == lengths.end() {
        *lengths.start()
    } else {
        rng.random(lengths)
    };
    if n == 0 {
        return Vec::new();
    }
    let (min, max) = (*range.start(), *range.end());
    match shape {
        SequenceShape::Constant(value) => {
            vec![value.unwrap_or_else(|| rng.random(range)); n]
        }
        SequenceShape::Dense => rng.random_iter(range).take(n).collect(),
        SequenceShape::Narrow => {
            let low = rng.random(min..=max - T::one());
            let high = low + T::one();
            let mut values: Vec<_> = rng.random_iter(low..=high).take(n).collect();
            values[0] = low;
            values[1] = high;
            rng.shuffle(&mut values);
            values
        }
        SequenceShape::Periodic => {
            let period = rng.random(2..=n / 2);
            let phase = rng.random(0..period);
            let mut values = Vec::with_capacity(n);
            values.extend(rng.random_iter(range).take(period));
            let (a, b) = sequence_distinct_pair(range, rng);
            values[0] = a;
            values[1] = b;
            values.rotate_left(phase);
            for i in period..n {
                values.push(values[i - period]);
            }
            values
        }
        SequenceShape::Ascending | SequenceShape::Descending => {
            let mut values: Vec<_> = rng.random_iter(range).take(n).collect();
            let (a, b) = sequence_distinct_pair(range, rng);
            values[0] = a;
            values[1] = b;
            values.sort_unstable();
            if matches!(shape, SequenceShape::Descending) {
                values.reverse();
            }
            values
        }
        SequenceShape::SparseFirst(_)
        | SequenceShape::SparseLast(_)
        | SequenceShape::Sparse
        | SequenceShape::SparseMany => {
            let (a, b) = sequence_distinct_pair(range, rng);
            let (mut background, mut exception) = (a.min(b), a.max(b));
            let reverse = match shape {
                SequenceShape::SparseFirst(reverse) | SequenceShape::SparseLast(reverse) => reverse,
                _ => rng.random(0..2) != 0,
            };
            if reverse {
                std::mem::swap(&mut background, &mut exception);
            }
            let mut values = vec![background; n];
            match shape {
                SequenceShape::SparseFirst(_) => values[0] = exception,
                SequenceShape::SparseLast(_) => values[n - 1] = exception,
                SequenceShape::Sparse => values[rng.random(1..n - 1)] = exception,
                SequenceShape::SparseMany => {
                    let count = rng.random(2..=n / 2);
                    values[..count].fill(exception);
                    rng.shuffle(&mut values);
                }
                _ => unreachable!(),
            }
            values
        }
    }
}

#[derive(Clone, Copy)]
enum SequenceShape<T> {
    Dense,
    Narrow,
    Constant(Option<T>),
    Periodic,
    Ascending,
    Descending,
    SparseFirst(bool),
    SparseLast(bool),
    Sparse,
    SparseMany,
}

impl<T> SequenceShape<T> {
    fn minimum_len(&self) -> usize {
        match self {
            Self::Constant(_) => 1,
            Self::Dense | Self::Narrow | Self::Ascending | Self::Descending => 2,
            Self::SparseFirst(_) | Self::SparseLast(_) | Self::Sparse => 3,
            Self::Periodic | Self::SparseMany => 4,
        }
    }
}

fn sequence_distinct_pair<T>(values: &RangeInclusive<T>, rng: &mut Xorshift) -> (T, T)
where
    T: IntBase,
    RangeInclusive<T>: RandomSpec<T>,
{
    let a = rng.random(values.clone());
    let b = rng.random(*values.start()..=*values.end() - T::one());
    (a, if b >= a { b + T::one() } else { b })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn test_sequence_cases_boundaries() {
        let mut rng = Xorshift::default();
        for trial in 0..64 {
            let low = rng.random(i8::MIN..i8::MAX);
            let high = if trial % 2 == 0 {
                low
            } else {
                rng.random(low + 1..=i8::MAX)
            };
            let start = match trial % 3 {
                0 => 0,
                1 => 1,
                _ => rng.random(0..=128),
            };
            let end = match trial {
                0 => 0,
                1 => 1,
                _ => rng.random(start.max(2)..=256),
            };
            let values: Vec<_> = sequence_cases(&mut rng, low..=high, start..=end).collect();
            assert!(values.iter().all(|s| {
                (start..=end).contains(&s.len()) && s.iter().all(|x| (low..=high).contains(x))
            }));
            assert_eq!(
                values.iter().filter(|s| s.is_empty()).count(),
                usize::from(start == 0)
            );
            if start <= 1 && end >= 1 {
                let unique: BTreeSet<_> = values
                    .iter()
                    .filter(|s| s.len() == 1)
                    .map(|s| s[0])
                    .collect();
                assert!(unique.contains(&low));
                assert!(unique.contains(&high));
                if low <= 0 && 0 <= high {
                    assert!(unique.contains(&0));
                }
            }
            let lengths: BTreeSet<_> = values.iter().map(Vec::len).collect();
            for boundary in [start, end].into_iter().chain(
                (0..usize::BITS)
                    .map(|bit| 1usize << bit)
                    .take_while(|p| *p <= end + 1),
            ) {
                for n in boundary.saturating_sub(1)..=boundary + 1 {
                    if (start..=end).contains(&n) {
                        assert!(lengths.contains(&n), "length {n} in {start}..={end}");
                    }
                }
            }
            for value in [low, high] {
                assert!(
                    values
                        .iter()
                        .any(|s| s.len() == end && s.iter().all(|x| *x == value))
                );
            }
        }
    }

    #[test]
    fn test_sequence_cases_structures() {
        let mut rng = Xorshift::default();
        for trial in 0..64 {
            let low = rng.random(i8::MIN..i8::MAX);
            let high = rng.random(low + 1..=i8::MAX);
            let n = match trial {
                0 => 4,
                1 => 5,
                _ => rng.random(4..=128),
            };
            let cases: Vec<_> = sequence_cases(&mut rng, low..=high, n..=n).collect();
            assert!(cases.iter().all(|s| s.len() == n));
            assert!(cases.iter().any(|s| s[0] < s[n - 1] && s.is_sorted()));
            assert!(
                cases
                    .iter()
                    .any(|s| s[0] > s[n - 1] && s.windows(2).all(|p| p[0] >= p[1]))
            );
            assert!(cases.iter().any(|s| {
                s.iter().any(|x| *x != s[0]) && (2..=n / 2).any(|p| s[p..] == s[..s.len() - p])
            }));
            for reverse in [false, true] {
                assert!(cases.iter().any(|s| {
                    s[0] != s[1] && (s[0] < s[1]) == reverse && s[1..].iter().all(|x| *x == s[1])
                }));
                assert!(cases.iter().any(|s| {
                    s[n - 1] != s[0]
                        && (s[n - 1] < s[0]) == reverse
                        && s[..n - 1].iter().all(|x| *x == s[0])
                }));
            }
            assert!(
                cases
                    .iter()
                    .any(|s| s[0] == s[n - 1] && s.iter().filter(|&&x| x != s[0]).count() == 1)
            );
            assert!(cases.iter().any(|s| {
                let unique: BTreeSet<_> = s.iter().copied().collect();
                unique.len() == 2 && unique.first().unwrap() + 1 == *unique.last().unwrap()
            }));
            assert!(cases.iter().any(|s| {
                let unique: BTreeSet<_> = s.iter().copied().collect();
                let count = s.iter().filter(|x| **x == s[0]).count();
                unique.len() == 2 && (2..=n - 2).contains(&count)
            }));
        }
    }

    #[test]
    fn test_sequence_cases_sampling() {
        let mut rng = Xorshift::default();
        let low = rng.random(i8::MIN..i8::MAX);
        let high = rng.random(low + 1..=i8::MAX);
        let n = rng.random(16..=128);
        let mut untouched = rng.clone();
        drop(sequence_cases(&mut rng, low..=high, n..=n));
        assert_eq!(rng.rand64(), untouched.rand64());
        let mut replay = rng.clone();
        let values: Vec<_> = sequence_cases(&mut rng, low..=high, n..=n).collect();
        let repeated: Vec<_> = sequence_cases(&mut replay, low..=high, n..=n).collect();
        assert_eq!(values, repeated);
        assert_eq!(rng.rand64(), replay.rand64());
        let mut positions = BTreeSet::new();
        for _ in 0..n {
            for s in sequence_cases(&mut rng, low..=high, n..=n) {
                if let Some(index) = (1..s.len() - 1).find(|&i| {
                    s[i] != s[0] && s.iter().enumerate().all(|(j, x)| j == i || *x == s[0])
                }) {
                    positions.insert(index);
                }
            }
        }
        assert!(positions.len() > 1);
        assert!(sequence_cases(&mut rng, i128::MIN..=i128::MAX, n..=n).all(|s| s.len() == n));
        assert!(sequence_cases(&mut rng, 0..=u128::MAX, n..=n).all(|s| s.len() == n));
    }

    #[test]
    fn test_integer_cases() {
        let mut inputs = Xorshift::default();
        macro_rules! test_types {
            ($($ty:ty),* $(,)?) => {$(
                for case in 0..64 {
                    let a: $ty = inputs.random(..);
                    let b: $ty = inputs.random(..);
                    let range = match case {
                        0 => <$ty>::MIN..=<$ty>::MAX,
                        1 => 0..=0,
                        2 => <$ty>::MIN..=<$ty>::MIN,
                        3 => <$ty>::MAX..=<$ty>::MAX,
                        _ if case % 4 == 0 => a..=a,
                        _ => a.min(b)..=a.max(b),
                    };
                    let values: BTreeSet<_> = integer_cases(range.clone()).collect();
                    assert!(!values.is_empty());
                    assert!(values.iter().all(|value| range.contains(value)));
                    for value in [*range.start(), *range.end()] {
                        assert!(values.contains(&value));
                        for neighbor in [value.checked_sub(1), value.checked_add(1)]
                            .into_iter()
                            .flatten()
                            .filter(|value| range.contains(value))
                        {
                            assert!(values.contains(&neighbor));
                        }
                    }
                    for base in [2, 10] {
                        let mut power: $ty = 1;
                        loop {
                            for sign in [Some(power), (0 as $ty).checked_sub(power)]
                                .into_iter()
                                .flatten()
                            {
                                for value in [sign.checked_sub(1), Some(sign), sign.checked_add(1)]
                                    .into_iter()
                                    .flatten()
                                    .filter(|value| range.contains(value))
                                {
                                    assert!(values.contains(&value));
                                }
                            }
                            let Some(next) = power.checked_mul(base) else { break };
                            power = next;
                        }
                    }
                }
            )*};
        }
        test_types!(
            u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
        );
    }

    #[test]
    fn test_integer_pairs() {
        let mut rng = Xorshift::default();
        let ranges = (-8i32..=8)
            .flat_map(|low| (low..=8).map(move |high| low..=high))
            .chain(
                rng.random_iter((-1000..=1000, -1000..=1000))
                    .take(64)
                    .map(|(a, b): (i32, i32)| a.min(b)..=a.max(b)),
            );
        for range in ranges {
            let expected: BTreeSet<_> = integer_cases(range.clone()).collect();
            let mut rows = BTreeMap::<_, BTreeSet<_>>::new();
            for (a, b) in integer_pairs(range.clone()) {
                assert!(range.contains(&a) && range.contains(&b));
                rows.entry(a).or_default().insert(b);
            }
            assert_eq!(rows.keys().copied().collect::<BTreeSet<_>>(), expected);
            assert!(rows.values().all(|row| row == &expected));
        }
    }

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
        for trial in 0..3_000 {
            let min = if trial % 3 == 0 {
                0
            } else {
                rng.random(0..=4_096)
            };
            let max = if trial % 3 == 1 {
                min
            } else {
                rng.random(min..=4_096)
            };
            let exhaustive_max = rng.random(0..=64);
            let rounds = rng.random(0..=64);
            let required = sample_usize(&mut rng, exhaustive_max, min..=max, 0);
            let mut replay = rng.clone();
            let sampled = sample_usize(&mut rng, exhaustive_max, min..=max, rounds);
            assert_eq!(
                sampled,
                sample_usize(&mut replay, exhaustive_max, min..max + 1, rounds)
            );
            assert_eq!(&sampled[..required.len()], required);
            assert_eq!(sampled.len(), required.len() + rounds);
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
