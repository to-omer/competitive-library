use super::{Matrix, One, Saturating, SemiRing, StationaryTransducer, Transducer, Unsigned, Zero};

/// Inputs are digits in `0..radix`; state ID 0 is the initial state.
#[derive(Debug, Clone)]
pub struct TransitionTable<S, O> {
    radix: usize,
    states: Vec<S>,
    accepting: Vec<bool>,
    transitions: Vec<Option<(usize, O)>>,
}

impl<S, O> TransitionTable<S, O> {
    pub fn new<T>(machine: &T, radix: usize, max_states: usize) -> Self
    where
        T: StationaryTransducer<Input = usize, State = S, Output = O> + ?Sized,
        S: Eq + std::hash::Hash + Clone,
    {
        assert!(max_states > 0);
        let initial = machine.start();
        let mut states = vec![initial.clone()];
        let mut ids = std::collections::HashMap::from([(initial, 0)]);
        let mut accepting = Vec::new();
        let mut transitions = Vec::new();
        let mut state = 0;
        while state < states.len() {
            accepting.push(machine.accept(&states[state]));
            for input in 0..radix {
                let edge = machine
                    .relation(&states[state], &input)
                    .map(|(next, output)| {
                        let next = *ids.entry(next).or_insert_with_key(|next| {
                            assert!(states.len() < max_states, "state limit exceeded");
                            let id = states.len();
                            states.push(next.clone());
                            id
                        });
                        (next, output)
                    });
                transitions.push(edge);
            }
            state += 1;
        }
        Self {
            radix,
            states,
            accepting,
            transitions,
        }
    }

    pub fn radix(&self) -> usize {
        self.radix
    }

    pub fn state_size(&self) -> usize {
        self.states.len()
    }

    /// Original states in ID order.
    pub fn states(&self) -> &[S] {
        &self.states
    }

    /// Indexed by input digit; each edge is `(next state ID, output)`.
    pub fn transitions(&self, state: usize) -> &[Option<(usize, O)>] {
        &self.transitions[state * self.radix..(state + 1) * self.radix]
    }

    pub fn is_accepting(&self, state: usize) -> bool {
        self.accepting[state]
    }
}

impl<S, O> StationaryTransducer for TransitionTable<S, O> where O: Clone {}

impl<S, O> Transducer for TransitionTable<S, O>
where
    O: Clone,
{
    type Input = usize;
    type Output = O;
    type State = usize;

    fn start(&self) -> Self::State {
        0
    }

    fn relation(&self, state: &usize, input: &usize) -> Option<(usize, O)> {
        self.transitions(*state).get(*input)?.clone()
    }

    fn accept(&self, state: &usize) -> bool {
        self.is_accepting(*state)
    }
}

impl<S, O> TransitionTable<S, O> {
    /// Entry `(from, to)` sums `weight(output)`, called once per edge.
    pub fn to_matrix<R>(&self, mut weight: impl FnMut(&O) -> R::T) -> Matrix<R>
    where
        R: SemiRing,
    {
        let size = self.state_size();
        let mut matrix = Matrix::zeros((size, size));
        for state in 0..size {
            for (next, output) in self.transitions(state).iter().flatten() {
                R::add_assign(&mut matrix[(state, *next)], &weight(output));
            }
        }
        matrix
    }
}

/// Counts accepted words, including leading zeros. Bound queries use the
/// bound's length, digits in `0..radix`, and at most the precomputed length.
#[derive(Debug)]
pub struct SuffixCounts<'a, V, S, O> {
    table: &'a TransitionTable<S, O>,
    counts: Vec<Vec<V>>,
}

impl<V, S, O> Clone for SuffixCounts<'_, V, S, O>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            table: self.table,
            counts: self.counts.clone(),
        }
    }
}

impl<S, O> TransitionTable<S, O> {
    /// Precomputes lengths `0..=len` in O(len * states * radix) time.
    pub fn suffix_counts<V>(&self, len: usize) -> SuffixCounts<'_, V, S, O>
    where
        V: Clone + Zero + One + std::ops::Add<Output = V>,
    {
        let mut counts = Vec::with_capacity(len + 1);
        counts.push(
            (0..self.state_size())
                .map(|state| {
                    if self.is_accepting(state) {
                        V::one()
                    } else {
                        V::zero()
                    }
                })
                .collect::<Vec<_>>(),
        );
        for remaining in 0..len {
            let next = (0..self.state_size())
                .map(|state| {
                    self.transitions(state)
                        .iter()
                        .flatten()
                        .fold(V::zero(), |sum, &(next, _)| {
                            sum + counts[remaining][next].clone()
                        })
                })
                .collect();
            counts.push(next);
        }
        SuffixCounts {
            table: self,
            counts,
        }
    }
}

impl<V, S, O> SuffixCounts<'_, V, S, O> {
    /// Number of accepting continuations of `len` digits from `state`.
    pub fn count(&self, state: usize, len: usize) -> &V {
        &self.counts[len][state]
    }
}

impl<V, S, O> SuffixCounts<'_, V, S, O>
where
    V: Clone + Zero + std::ops::Add<Output = V>,
{
    pub fn count_lt(&self, bound: &[usize]) -> V {
        self.count_bound::<true>(0, bound, false)
    }

    pub fn count_le(&self, bound: &[usize]) -> V {
        self.count_bound::<true>(0, bound, true)
    }

    /// Counts the inclusive interval `[lower, upper]` with equal-length bounds.
    pub fn count_between(&self, lower: &[usize], upper: &[usize]) -> V {
        assert_eq!(lower.len(), upper.len());
        if lower > upper {
            return V::zero();
        }
        assert!(lower.len() < self.counts.len());
        let mut state = 0;
        for (i, (&lo, &hi)) in lower.iter().zip(upper).enumerate() {
            let edges = self.table.transitions(state);
            if lo == hi {
                match &edges[lo] {
                    Some((next, _)) => state = *next,
                    None => return V::zero(),
                }
                continue;
            }
            let mut sum = V::zero();
            for (digit, edge) in edges.iter().enumerate().take(hi + 1).skip(lo) {
                if let Some((next, _)) = edge {
                    let count = if digit == lo {
                        self.count_bound::<false>(*next, &lower[i + 1..], true)
                    } else if digit == hi {
                        self.count_bound::<true>(*next, &upper[i + 1..], true)
                    } else {
                        self.count(*next, lower.len() - i - 1).clone()
                    };
                    sum = sum + count;
                }
            }
            return sum;
        }
        self.count(state, 0).clone()
    }
}

impl<U, S, O> SuffixCounts<'_, Saturating<U>, S, O>
where
    U: Unsigned,
{
    /// The zero-based `k`th accepted word in lexicographic order, or `None` if absent.
    /// Requires `k < U::MAX` because counts may saturate.
    pub fn kth(&self, len: usize, mut k: U) -> Option<Vec<usize>> {
        assert!(k < U::maximum());
        if self.count(0, len).0 <= k {
            return None;
        }
        let mut word = Vec::with_capacity(len);
        let mut state = 0;
        for remaining in (0..len).rev() {
            for (digit, edge) in self.table.transitions(state).iter().enumerate() {
                if let Some((next, _)) = edge {
                    let count = self.count(*next, remaining).0;
                    if count <= k {
                        k -= count;
                    } else {
                        word.push(digit);
                        state = *next;
                        break;
                    }
                }
            }
        }
        Some(word)
    }

    /// Like `kth`, within `[lower, upper]` with equal-length bounds.
    pub fn kth_between(&self, lower: &[usize], upper: &[usize], mut k: U) -> Option<Vec<usize>>
    where
        Saturating<U>: std::ops::Add<Output = Saturating<U>>,
    {
        assert!(k < U::maximum());
        assert_eq!(lower.len(), upper.len());
        if lower > upper {
            return None;
        }
        let lower_counts = self.bound_counts::<false>(lower);
        let upper_counts = self.bound_counts::<true>(upper);
        let mut state = 0;
        let (mut tight_lower, mut tight_upper) = (true, true);
        let mut word = Vec::with_capacity(lower.len());
        for i in 0..lower.len() {
            let edges = self.table.transitions(state);
            let lo = if tight_lower { lower[i] } else { 0 };
            let hi = if tight_upper {
                upper[i]
            } else {
                self.table.radix() - 1
            };
            if tight_lower && tight_upper && lo == hi {
                state = edges[lo].as_ref()?.0;
                word.push(lo);
                continue;
            }
            let mut selected = None;
            for (digit, edge) in edges.iter().enumerate().take(hi + 1).skip(lo) {
                if let Some((next, _)) = edge {
                    let count = if tight_lower && digit == lo {
                        lower_counts[i + 1].0
                    } else if tight_upper && digit == hi {
                        upper_counts[i + 1].0
                    } else {
                        self.count(*next, lower.len() - i - 1).0
                    };
                    if count <= k {
                        k -= count;
                    } else {
                        selected = Some((digit, *next));
                        break;
                    }
                }
            }
            let (digit, next) = selected?;
            tight_lower &= digit == lower[i];
            tight_upper &= digit == upper[i];
            word.push(digit);
            state = next;
        }
        (k.is_zero() && self.table.is_accepting(state)).then_some(word)
    }
}

impl<V, S, O> SuffixCounts<'_, V, S, O>
where
    V: Clone + Zero + std::ops::Add<Output = V>,
{
    fn count_bound<const LESS: bool>(
        &self,
        mut state: usize,
        bound: &[usize],
        inclusive: bool,
    ) -> V {
        assert!(bound.len() < self.counts.len());
        let mut sum = V::zero();
        for (i, &digit) in bound.iter().enumerate() {
            let edges = self.table.transitions(state);
            let range = if LESS {
                0..digit
            } else {
                digit + 1..self.table.radix()
            };
            for (next, _) in edges[range].iter().flatten() {
                sum = sum + self.count(*next, bound.len() - i - 1).clone();
            }
            match &edges[digit] {
                Some((next, _)) => state = *next,
                None => return sum,
            }
        }
        if inclusive {
            sum = sum + self.count(state, 0).clone();
        }
        sum
    }

    fn bound_counts<const LESS: bool>(&self, bound: &[usize]) -> Vec<V> {
        assert!(bound.len() < self.counts.len());
        let mut counts = vec![V::zero(); bound.len() + 1];
        let mut state = 0;
        let mut matched = 0;
        for (i, &digit) in bound.iter().enumerate() {
            let edges = self.table.transitions(state);
            let range = if LESS {
                0..digit
            } else {
                digit + 1..self.table.radix()
            };
            counts[i] = edges[range]
                .iter()
                .flatten()
                .fold(V::zero(), |sum, (next, _)| {
                    sum + self.count(*next, bound.len() - i - 1).clone()
                });
            let Some((next, _)) = &edges[digit] else {
                break;
            };
            state = *next;
            matched += 1;
        }
        if matched == bound.len() {
            counts[matched] = self.count(state, 0).clone();
        }
        let mut sum = counts[matched].clone();
        for count in counts[..matched].iter_mut().rev() {
            sum = sum + count.clone();
            *count = sum.clone();
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AddMulOperation, AdditiveOperation},
        data_structure::{
            ChainTransducer, FunctionalTransducer, IdentityTransducer, IntersectionTransducer,
            LexicographicalTransducer, SequenceTransducer, VecMap,
        },
        num::mint_basic::MInt2,
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, integer_pairs},
        },
        transducer,
    };

    #[test]
    fn test_transition_table() {
        for radix in 0usize..=4 {
            for modulus in 1..=9 {
                let machine = transducer!(=> || 0usize, |&state, &digit| {
                    Some(((state * radix + digit) % modulus, digit))
                }, |&state| state == 0);
                let table = TransitionTable::new(&machine, radix, modulus);
                assert_eq!(table.radix(), radix);
                assert_eq!(table.state_size(), if radix < 2 { 1 } else { modulus });
                assert_eq!(table.states()[0], 0);
                for (id, &state) in table.states().iter().enumerate() {
                    assert_eq!(table.is_accepting(id), state == 0);
                    for (digit, edge) in table.transitions(id).iter().enumerate() {
                        let (next, output) = edge.as_ref().unwrap();
                        assert_eq!(table.states()[*next], (state * radix + digit) % modulus);
                        assert_eq!(*output, digit);
                    }
                }
                for len in 0..=4 {
                    let words = exhaustive_sequences(0..radix, len..=len);
                    let expected = words
                        .map(|word| word.into_iter().fold(0, |n, d| n * radix + d))
                        .filter(|n| n % modulus == 0)
                        .fold((0, 0), |(sum, count), n| (sum + n as u64, count + 1));
                    let mut dp = (&table)
                        .dp_with::<(AdditiveOperation<u64>, AdditiveOperation<u64>), _>(
                            (0, 1),
                            || VecMap::with_len(table.state_size(), |&id| id),
                        );
                    assert_eq!(
                        dp.run_effect(
                            len,
                            |_| 0..radix,
                            |&(sum, count), digit| {
                                (sum * radix as u64 + count * digit as u64, count)
                            }
                        ),
                        expected
                    );
                    assert_eq!(dp.iter_accept().map(|(_, v)| v.1).sum::<u64>(), expected.1);
                }
                for upper in exhaustive_sequences(0..radix, 0..=3) {
                    let value = upper.iter().fold(0, |n, &d| n * radix + d);
                    let expected = value / modulus + 1;
                    let mut dp = (&table)
                        .intersection(transducer!(<= upper))
                        .counting::<usize>();
                    assert_eq!(dp.run(upper.len(), |_| 0..radix), expected);
                    assert_eq!(dp.run(0, |_| 0..radix), expected);
                }
            }
        }
    }

    #[test]
    fn test_transition_table_owned_output() {
        struct Output;
        let machine = transducer!(=> || (), |_, _: &usize| Some(((), Output)), |_| true);
        let machine: &dyn StationaryTransducer<Input = usize, State = (), Output = Output> =
            &machine;
        let table = TransitionTable::new(machine, 3, 1);
        let counts = table.suffix_counts::<u64>(4);
        assert_eq!(*counts.clone().count(0, 4), 81);
        assert_eq!(table.transitions(0).len(), 3);
    }

    #[test]
    fn test_transition_matrix() {
        struct Weight(usize);
        for radix in 0..=2 {
            let width = 2 * radix;
            for edges in exhaustive_sequences([None, Some(0), Some(1)], width..=width) {
                for weights in exhaustive_sequences([0usize, 1, 3], radix..=radix) {
                    let machine = transducer!(=> || 0usize, |&state, &digit: &usize| {
                        edges[state * radix + digit].map(|next| (next, Weight(weights[digit])))
                    }, |_| true);
                    let table = TransitionTable::new(&machine, radix, 2);
                    let mut calls = 0;
                    let matrix = table.to_matrix::<AddMulOperation<usize>>(|weight| {
                        calls += 1;
                        weight.0
                    });
                    let edge_count = table
                        .states()
                        .iter()
                        .map(|&state| {
                            edges[state * radix..(state + 1) * radix]
                                .iter()
                                .flatten()
                                .count()
                        })
                        .sum::<usize>();
                    assert_eq!(calls, edge_count);
                    for len in 0..=6 {
                        let mut expected = vec![vec![0; table.state_size()]; table.state_size()];
                        for (from, &initial) in table.states().iter().enumerate() {
                            for word in exhaustive_sequences(0..radix, len..=len) {
                                let path = word.into_iter().try_fold(
                                    (initial, 1),
                                    |(state, value), digit| {
                                        edges[state * radix + digit]
                                            .map(|next| (next, value * weights[digit]))
                                    },
                                );
                                if let Some((state, value)) = path {
                                    let to =
                                        table.states().iter().position(|&s| s == state).unwrap();
                                    expected[from][to] += value;
                                }
                            }
                        }
                        assert_eq!(matrix.clone().pow(len).data, expected);
                    }
                }
            }
        }
    }

    #[test]
    fn test_suffix_counts() {
        for (edges, accepting) in suffix_count_inputs() {
            let radix = edges[0].len();
            let machine = transducer!(=> || 0usize, |&state, &digit: &usize| {
                edges[state][digit].map(|next| (next, ()))
            }, |&state| accepting[state]);
            let table = TransitionTable::new(&machine, radix, edges.len());
            let counts = table.suffix_counts::<Saturating<u8>>(6);
            let modular = table.suffix_counts::<MInt2>(6);
            for (len, words) in suffix_count_words(radix) {
                for (id, &initial) in table.states().iter().enumerate() {
                    let expected = words
                        .iter()
                        .filter(|word| {
                            word.iter()
                                .try_fold(initial, |state, &digit| edges[state][digit])
                                .is_some_and(|state| accepting[state])
                        })
                        .count();
                    assert_eq!(counts.count(id, len).0 as usize, expected.min(255));
                    assert_eq!(*modular.count(id, len), MInt2::from(expected));
                }
            }
        }
    }

    #[test]
    fn test_suffix_counts_bound() {
        for (edges, accepting) in suffix_count_inputs() {
            let radix = edges[0].len();
            let machine = transducer!(=> || 0usize, |&state, &digit: &usize| {
                edges[state][digit].map(|next| (next, ()))
            }, |&state| accepting[state]);
            let table = TransitionTable::new(&machine, radix, edges.len());
            let counts = table.suffix_counts::<Saturating<u8>>(6);
            let modular = table.suffix_counts::<MInt2>(6);
            for (_, words) in suffix_count_words(radix) {
                let mut expected = 0;
                for word in words {
                    assert_eq!(counts.count_lt(&word).0 as usize, expected.min(255));
                    assert_eq!(modular.count_lt(&word), MInt2::from(expected));
                    expected += usize::from(
                        word.iter()
                            .try_fold(0, |state, &digit| edges[state][digit])
                            .is_some_and(|state| accepting[state]),
                    );
                    assert_eq!(counts.count_le(&word).0 as usize, expected.min(255));
                    assert_eq!(modular.count_le(&word), MInt2::from(expected));
                }
            }
        }
    }

    #[test]
    fn test_suffix_counts_interval() {
        let mut rng = Xorshift::default();
        for (edges, accepting) in suffix_count_inputs() {
            let radix = edges[0].len();
            let machine = transducer!(=> || 0usize, |&state, &digit: &usize| {
                edges[state][digit].map(|next| (next, ()))
            }, |&state| accepting[state]);
            let table = TransitionTable::new(&machine, radix, edges.len());
            let counts = table.suffix_counts::<Saturating<u8>>(6);
            let modular = table.suffix_counts::<MInt2>(6);
            for (_, words) in suffix_count_words(radix).filter(|(_, words)| !words.is_empty()) {
                let accepted: Vec<_> = words
                    .iter()
                    .filter(|word| {
                        word.iter()
                            .try_fold(0, |state, &digit| edges[state][digit])
                            .is_some_and(|state| accepting[state])
                    })
                    .map(Vec::as_slice)
                    .collect();
                for (lower, upper) in suffix_count_intervals(&words, &accepted, &mut rng) {
                    let expected = accepted
                        .iter()
                        .filter(|&&word| lower <= word && word <= upper)
                        .count();
                    assert_eq!(
                        counts.count_between(lower, upper).0 as usize,
                        expected.min(255)
                    );
                    assert_eq!(modular.count_between(lower, upper), MInt2::from(expected));
                }
            }
        }
    }

    #[test]
    fn test_suffix_counts_kth() {
        for (edges, accepting) in suffix_count_inputs() {
            let radix = edges[0].len();
            let machine = transducer!(=> || 0usize, |&state, &digit: &usize| {
                edges[state][digit].map(|next| (next, ()))
            }, |&state| accepting[state]);
            let table = TransitionTable::new(&machine, radix, edges.len());
            let counts = table.suffix_counts::<Saturating<u8>>(6);
            for (len, words) in suffix_count_words(radix) {
                let accepted = words.iter().filter(|word| {
                    word.iter()
                        .try_fold(0, |state, &digit| edges[state][digit])
                        .is_some_and(|state| accepting[state])
                });
                for (k, expected) in accepted
                    .map(|word| Some(word.as_slice()))
                    .chain([None])
                    .take(255)
                    .enumerate()
                {
                    assert_eq!(counts.kth(len, k as u8).as_deref(), expected);
                }
            }
        }
    }

    #[test]
    fn test_suffix_counts_kth_between() {
        let mut rng = Xorshift::default();
        for (edges, accepting) in suffix_count_inputs() {
            let radix = edges[0].len();
            let machine = transducer!(=> || 0usize, |&state, &digit: &usize| {
                edges[state][digit].map(|next| (next, ()))
            }, |&state| accepting[state]);
            let table = TransitionTable::new(&machine, radix, edges.len());
            let counts = table.suffix_counts::<Saturating<u8>>(6);
            for (_, words) in suffix_count_words(radix).filter(|(_, words)| !words.is_empty()) {
                let accepted: Vec<_> = words
                    .iter()
                    .filter(|word| {
                        word.iter()
                            .try_fold(0, |state, &digit| edges[state][digit])
                            .is_some_and(|state| accepting[state])
                    })
                    .map(Vec::as_slice)
                    .collect();
                for (lower, upper) in suffix_count_intervals(&words, &accepted, &mut rng) {
                    let interval = accepted
                        .iter()
                        .copied()
                        .filter(|&word| lower <= word && word <= upper);
                    for (k, expected) in interval.map(Some).chain([None]).take(255).enumerate() {
                        assert_eq!(
                            counts.kth_between(lower, upper, k as u8).as_deref(),
                            expected
                        );
                    }
                }
            }
        }
    }

    fn suffix_count_inputs() -> impl Iterator<Item = (Vec<Vec<Option<usize>>>, Vec<bool>)> {
        // All partial binary automata with two states, including dead ends,
        // cycles, unreachable states, and parallel edges with different inputs.
        let binary = exhaustive_sequences([None, Some(0), Some(1)], 4..=4).flat_map(|edges| {
            exhaustive_sequences([false, true], 2..=2)
                .map(move |accepting| (vec![edges[..2].to_vec(), edges[2..].to_vec()], accepting))
        });
        let boundaries = (0..=1).flat_map(|radix| {
            [false, true].map(move |accepting| (vec![vec![Some(0); radix]], vec![accepting]))
        });
        let distinct = (0..=10)
            .map(|state| {
                (0..10)
                    .map(|digit| (state != digit + 1).then_some(digit + 1))
                    .collect()
            })
            .collect();
        // State 0 skips padding and rejects integer zero. States 1..=3
        // track remainders for positive integers whose digits are 1, 4, or 9.
        let positive = (0usize..4)
            .map(|state| {
                (0..10)
                    .map(|digit| match (state, digit) {
                        (0, 0) => Some(0),
                        (_, 1 | 4 | 9) => Some((state.saturating_sub(1) * 10 + digit) % 3 + 1),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        binary.chain(boundaries).chain([
            (distinct, vec![true; 11]),
            (positive, vec![false, true, false, false]),
        ])
    }

    fn suffix_count_words(radix: usize) -> impl Iterator<Item = (usize, Vec<Vec<usize>>)> {
        (0..=6)
            .take_while(move |&len| radix.pow(len as u32) <= 1000)
            .map(move |len| {
                let mut words: Vec<_> = exhaustive_sequences(0..radix, len..=len).collect();
                words.sort();
                (len, words)
            })
    }

    fn suffix_count_intervals<'a>(
        words: &'a [Vec<usize>],
        accepted: &'a [&[usize]],
        rng: &'a mut Xorshift,
    ) -> impl Iterator<Item = (&'a [usize], &'a [usize])> {
        let n = words.len();
        let all = (0..n * n)
            .take(if n <= 16 { n * n } else { 0 })
            .map(move |i| (i / n, i % n));
        let boundaries = integer_pairs(0..=n - 1).filter(move |_| n > 16);
        let sampled = (0..n)
            .filter(move |_| n > 16)
            .map(move |lo| (lo, rng.random(0..n)));
        let pairs = all
            .chain(boundaries)
            .chain(sampled)
            .map(move |(lo, hi)| (words[lo].as_slice(), words[hi].as_slice()));
        // Straddle the saturation cap, including intervals whose prefixes
        // have already saturated and short intervals after those prefixes.
        let saturated = [0, 256].into_iter().flat_map(move |start| {
            [1, 18, 254, 255, 256].into_iter().filter_map(move |len| {
                Some((*accepted.get(start)?, *accepted.get(start + len - 1)?))
            })
        });
        pairs.chain(saturated)
    }
}
