use super::{BitSet, Field, Invertible, Matrix, RandomSpec, SerdeByteStr, Xorshift};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
    iter::{from_fn, once_with},
    marker::PhantomData,
    time::Instant,
};

pub trait BlackBoxAutomaton {
    type Output;
    fn sigma(&self) -> usize; // Σ={0,1,...,sigma-1}
    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>;
}

#[derive(Debug, Clone)]
pub struct BlackBoxAutomatonImpl<T, F>
where
    F: Fn(Vec<usize>) -> T,
{
    sigma: usize,
    behavior_fn: F,
    memo: RefCell<HashMap<Vec<usize>, T>>,
}

impl<T, F> BlackBoxAutomatonImpl<T, F>
where
    F: Fn(Vec<usize>) -> T,
{
    pub fn new(sigma: usize, behavior_fn: F) -> Self {
        Self {
            sigma,
            behavior_fn,
            memo: RefCell::new(HashMap::new()),
        }
    }
}

impl<T, F> BlackBoxAutomaton for BlackBoxAutomatonImpl<T, F>
where
    F: Fn(Vec<usize>) -> T,
    T: Clone,
{
    type Output = T;

    fn sigma(&self) -> usize {
        self.sigma
    }

    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>,
    {
        let input: Vec<usize> = input.into_iter().collect();
        self.memo
            .borrow_mut()
            .entry(input.clone())
            .or_insert_with(|| (self.behavior_fn)(input))
            .clone()
    }
}

impl<A> BlackBoxAutomaton for &A
where
    A: BlackBoxAutomaton,
{
    type Output = A::Output;

    fn sigma(&self) -> usize {
        (*self).sigma()
    }

    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>,
    {
        (*self).behavior(input)
    }
}

#[derive(Debug, Clone)]
struct DfaState {
    delta: Vec<usize>,
    accept: bool,
}

#[derive(Debug, Clone)]
pub struct DeterministicFiniteAutomaton {
    states: Vec<DfaState>,
    initial_state: usize,
}

impl DeterministicFiniteAutomaton {
    pub fn size(&self) -> usize {
        self.states.len()
    }
    pub fn delta(&self, state: usize, input: usize) -> usize {
        assert!(state < self.states.len());
        assert!(input < self.states[0].delta.len());
        self.states[state].delta[input]
    }
    pub fn accept(&self, state: usize) -> bool {
        assert!(state < self.states.len());
        self.states[state].accept
    }
}

impl BlackBoxAutomaton for DeterministicFiniteAutomaton {
    type Output = bool;

    fn sigma(&self) -> usize {
        self.states[0].delta.len()
    }

    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>,
    {
        let mut state = self.initial_state;
        for x in input {
            state = self.states[state].delta[x];
        }
        self.states[state].accept
    }
}

impl SerdeByteStr for DfaState {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.delta.serialize(buf);
        self.accept.serialize(buf);
    }

    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        let delta = Vec::deserialize(iter);
        let accept = bool::deserialize(iter);
        Self { delta, accept }
    }
}

impl SerdeByteStr for DeterministicFiniteAutomaton {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.states.serialize(buf);
        self.initial_state.serialize(buf);
    }

    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        let states = Vec::deserialize(iter);
        let initial_state = usize::deserialize(iter);
        Self {
            states,
            initial_state,
        }
    }
}

pub struct WeightedFiniteAutomaton<F>
where
    F: Field<Additive: Invertible, Multiplicative: Invertible>,
{
    pub initial_weights: Matrix<F>,
    pub transitions: Vec<Matrix<F>>,
    pub final_weights: Matrix<F>,
}

impl<F> Debug for WeightedFiniteAutomaton<F>
where
    F: Field<T: Debug, Additive: Invertible, Multiplicative: Invertible>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeightedFiniteAutomaton")
            .field("initial_weights", &self.initial_weights)
            .field("transitions", &self.transitions)
            .field("final_weights", &self.final_weights)
            .finish()
    }
}

impl<F> Clone for WeightedFiniteAutomaton<F>
where
    F: Field<Additive: Invertible, Multiplicative: Invertible>,
{
    fn clone(&self) -> Self {
        Self {
            initial_weights: self.initial_weights.clone(),
            transitions: self.transitions.clone(),
            final_weights: self.final_weights.clone(),
        }
    }
}

impl<F> BlackBoxAutomaton for WeightedFiniteAutomaton<F>
where
    F: Field<Additive: Invertible, Multiplicative: Invertible>,
{
    type Output = F::T;

    fn sigma(&self) -> usize {
        self.transitions.len()
    }

    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>,
    {
        let mut weights = self.initial_weights.clone();
        for x in input {
            weights = &weights * &self.transitions[x];
        }
        let result = &weights * &self.final_weights;
        if result.shape != (0, 0) {
            result[0][0].clone()
        } else {
            F::zero()
        }
    }
}

impl<F> SerdeByteStr for WeightedFiniteAutomaton<F>
where
    F: Field<T: SerdeByteStr, Additive: Invertible, Multiplicative: Invertible>,
{
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.initial_weights.serialize(buf);
        self.transitions.serialize(buf);
        self.final_weights.serialize(buf);
    }

    fn deserialize<I>(iter: &mut I) -> Self
    where
        I: Iterator<Item = u8>,
    {
        let initial_weights = Matrix::deserialize(iter);
        let transitions = Vec::deserialize(iter);
        let final_weights = Matrix::deserialize(iter);
        Self {
            initial_weights,
            transitions,
            final_weights,
        }
    }
}

pub fn dense_sampling(sigma: usize, max_len: usize) -> impl Iterator<Item = Vec<usize>> {
    assert_ne!(sigma, 0, "Sigma must be greater than 0");
    let mut current = vec![];
    once_with(Vec::new).chain(from_fn(move || {
        let mut carry = true;
        for i in (0..current.len()).rev() {
            current[i] += 1;
            if current[i] == sigma {
                current[i] = 0;
            } else {
                carry = false;
                break;
            }
        }
        if carry {
            current.push(0);
        }
        if current.len() <= max_len {
            Some(current.to_vec())
        } else {
            None
        }
    }))
}

pub fn random_sampling(
    sigma: usize,
    len_spec: impl RandomSpec<usize>,
    seconds: f64,
) -> impl Iterator<Item = Vec<usize>> {
    assert_ne!(sigma, 0, "Sigma must be greater than 0");
    let now = Instant::now();
    let mut rng = Xorshift::new();
    from_fn(move || {
        if now.elapsed().as_secs_f64() > seconds {
            None
        } else {
            let n = rng.random(&len_spec);
            Some(rng.random_iter(0..sigma).take(n).collect())
        }
    })
}

#[derive(Debug, Clone)]
pub struct DfaLearning<A>
where
    A: BlackBoxAutomaton<Output = bool>,
{
    automaton: A,
    prefixes: Vec<Vec<usize>>,
    suffixes: Vec<Vec<usize>>,
    table: Vec<BitSet>,
    row_map: HashMap<BitSet, usize>,
}

impl<A> DfaLearning<A>
where
    A: BlackBoxAutomaton<Output = bool>,
{
    pub fn new(automaton: A) -> Self {
        let mut this = Self {
            automaton,
            prefixes: vec![],
            suffixes: vec![],
            table: vec![],
            row_map: HashMap::new(),
        };
        this.add_suffix(vec![]);
        this.add_prefix(vec![]);
        this
    }
    fn add_prefix(&mut self, prefix: Vec<usize>) -> usize {
        let row: BitSet = self
            .suffixes
            .iter()
            .map(|s| {
                self.automaton
                    .behavior(prefix.iter().cloned().chain(s.iter().cloned()))
            })
            .collect();
        *self.row_map.entry(row.clone()).or_insert_with(|| {
            let idx = self.table.len();
            self.table.push(row);
            self.prefixes.push(prefix);
            idx
        })
    }
    fn add_suffix(&mut self, suffix: Vec<usize>) {
        if self.suffixes.contains(&suffix) {
            return;
        }
        for (prefix, table) in self.prefixes.iter_mut().zip(&mut self.table) {
            table.push(
                self.automaton
                    .behavior(prefix.iter().cloned().chain(suffix.iter().cloned())),
            );
        }
        self.suffixes.push(suffix);
        self.row_map.clear();
        for (i_prefix, row) in self.table.iter().enumerate() {
            self.row_map.insert(row.clone(), i_prefix);
        }
    }
    pub fn construct_dfa(&mut self) -> DeterministicFiniteAutomaton {
        let sigma = self.automaton.sigma();
        let mut dfa = DeterministicFiniteAutomaton {
            states: vec![],
            initial_state: 0,
        };
        let mut i_prefix = 0;
        while i_prefix < self.prefixes.len() {
            let mut delta = vec![];
            for x in 0..sigma {
                let prefix: Vec<usize> =
                    self.prefixes[i_prefix].iter().cloned().chain([x]).collect();
                let index = self.add_prefix(prefix);
                delta.push(index);
            }
            dfa.states.push(DfaState {
                delta,
                accept: self.table[i_prefix].get(0),
            });
            i_prefix += 1;
        }
        dfa
    }
    pub fn train_sample(&mut self, dfa: &DeterministicFiniteAutomaton, sample: &[usize]) -> bool {
        let expected = self.automaton.behavior(sample.iter().cloned());
        if expected == dfa.behavior(sample.iter().cloned()) {
            return false;
        }
        let n = sample.len();
        let mut states: Vec<(usize, usize)> = Vec::with_capacity(n + 1);
        let mut s = 0usize;
        states.push((s, 0));
        for (k, &x) in sample.iter().enumerate() {
            s = dfa.states[s].delta[x];
            states.push((s, k + 1));
        }
        let split = states.partition_point(|&(state, k)| {
            self.automaton.behavior(
                self.prefixes[state]
                    .iter()
                    .cloned()
                    .chain(sample[k..].iter().cloned()),
            ) == expected
        });
        let new_prefix = sample[..split].to_vec();
        let new_suffix = sample[split..].to_vec();
        self.add_suffix(new_suffix);
        self.add_prefix(new_prefix);
        true
    }
    pub fn train(
        &mut self,
        samples: impl IntoIterator<Item = Vec<usize>>,
    ) -> DeterministicFiniteAutomaton {
        let mut dfa = self.construct_dfa();
        for sample in samples {
            if self.train_sample(&dfa, &sample) {
                dfa = self.construct_dfa();
            }
        }
        dfa
    }
}

pub struct WfaLearning<F, A>
where
    F: Field<Additive: Invertible, Multiplicative: Invertible>,
    A: BlackBoxAutomaton<Output = F::T>,
{
    automaton: A,
    prefixes: Vec<Vec<usize>>,
    suffixes: Vec<Vec<usize>>,
    inv_h: Matrix<F>,
    nh: Vec<Matrix<F>>,
    wfa: WeightedFiniteAutomaton<F>,
    _marker: PhantomData<fn() -> F>,
}

impl<F, A> Debug for WfaLearning<F, A>
where
    F: Field<T: Debug, Additive: Invertible, Multiplicative: Invertible>,
    A: BlackBoxAutomaton<Output = F::T> + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WfaLearning")
            .field("automaton", &self.automaton)
            .field("prefixes", &self.prefixes)
            .field("suffixes", &self.suffixes)
            .field("inv_h", &self.inv_h)
            .field("nh", &self.nh)
            .field("wfa", &self.wfa)
            .finish()
    }
}

impl<F, A> Clone for WfaLearning<F, A>
where
    F: Field<Additive: Invertible, Multiplicative: Invertible>,
    A: BlackBoxAutomaton<Output = F::T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            automaton: self.automaton.clone(),
            prefixes: self.prefixes.clone(),
            suffixes: self.suffixes.clone(),
            inv_h: self.inv_h.clone(),
            nh: self.nh.clone(),
            wfa: self.wfa.clone(),
            _marker: self._marker,
        }
    }
}

impl<F, A> WfaLearning<F, A>
where
    F: Field<T: PartialEq, Additive: Invertible, Multiplicative: Invertible>,
    A: BlackBoxAutomaton<Output = F::T>,
{
    pub fn new(automaton: A) -> Self {
        let sigma = automaton.sigma();
        Self {
            automaton,
            prefixes: vec![],
            suffixes: vec![],
            inv_h: Matrix::zeros((0, 0)),
            nh: vec![Matrix::zeros((0, 0)); sigma],
            wfa: WeightedFiniteAutomaton {
                initial_weights: Matrix::zeros((1, 0)),
                transitions: vec![Matrix::zeros((0, 0)); sigma],
                final_weights: Matrix::zeros((0, 1)),
            },
            _marker: PhantomData,
        }
    }
    pub fn wfa(&self) -> &WeightedFiniteAutomaton<F> {
        &self.wfa
    }
    fn split_sample(&mut self, sample: &[usize]) -> Option<(Vec<usize>, Vec<usize>)> {
        if self.prefixes.is_empty() && !F::is_zero(&self.automaton.behavior(sample.iter().cloned()))
        {
            return Some((vec![], sample.to_vec()));
        }
        let expected = self.automaton.behavior(sample.iter().cloned());
        if expected == self.wfa.behavior(sample.iter().cloned()) {
            return None;
        }
        let n = sample.len();
        let dim = self.wfa.final_weights.shape.0;
        let mut states: Vec<(Matrix<F>, usize)> = Vec::with_capacity(n + 1);
        let mut v = self.wfa.final_weights.clone();
        states.push((v.clone(), n));
        for k in (0..n).rev() {
            v = &self.wfa.transitions[sample[k]] * &v;
            states.push((v.clone(), k));
        }
        states.reverse();
        let split = states.partition_point(|(state, k)| {
            (0..dim).any(|j| {
                self.automaton.behavior(
                    self.prefixes[j]
                        .iter()
                        .cloned()
                        .chain(sample[*k..].iter().cloned()),
                ) != state[j][0]
            })
        });
        Some((sample[..split].to_vec(), sample[split..].to_vec()))
    }
    pub fn train_sample(&mut self, sample: &[usize]) -> bool {
        let Some((prefix, suffix)) = self.split_sample(sample) else {
            return false;
        };
        self.prefixes.push(prefix);
        self.suffixes.push(suffix);
        let n = self.inv_h.shape.0;
        let prefix = &self.prefixes[n];
        let suffix = &self.suffixes[n];
        let u = Matrix::<F>::new_with((n, 1), |i, _| {
            self.automaton.behavior(
                self.prefixes[i]
                    .iter()
                    .cloned()
                    .chain(suffix.iter().cloned()),
            )
        });
        let v = Matrix::<F>::new_with((1, n), |_, j| {
            self.automaton.behavior(
                prefix
                    .iter()
                    .cloned()
                    .chain(self.suffixes[j].iter().cloned()),
            )
        });
        let w = Matrix::<F>::new_with((1, 1), |_, _| {
            self.automaton
                .behavior(prefix.iter().cloned().chain(suffix.iter().cloned()))
        });
        let t = &self.inv_h * &u;
        let s = &v * &self.inv_h;
        let d = F::inv(&(&w - &(&v * &t))[0][0]);
        let dh = &t * &s;
        for i in 0..n {
            for j in 0..n {
                F::add_assign(&mut self.inv_h[i][j], &F::mul(&dh[i][j], &d));
            }
        }
        self.inv_h
            .add_col_with(|i, _| F::neg(&F::mul(&t[i][0], &d)));
        self.inv_h.add_row_with(|_, j| {
            if j != n {
                F::neg(&F::mul(&s[0][j], &d))
            } else {
                d.clone()
            }
        });

        for (x, transition) in self.wfa.transitions.iter_mut().enumerate() {
            let b = &(&self.nh[x] * &t) * &s;
            for i in 0..n {
                for j in 0..n {
                    F::add_assign(&mut transition[i][j], &F::mul(&b[i][j], &d));
                }
            }
        }
        for (x, nh) in self.nh.iter_mut().enumerate() {
            nh.add_col_with(|i, j| {
                self.automaton.behavior(
                    self.prefixes[i]
                        .iter()
                        .cloned()
                        .chain([x])
                        .chain(self.suffixes[j].iter().cloned()),
                )
            });
            nh.add_row_with(|i, j| {
                self.automaton.behavior(
                    self.prefixes[i]
                        .iter()
                        .cloned()
                        .chain([x])
                        .chain(self.suffixes[j].iter().cloned()),
                )
            });
        }
        self.wfa
            .initial_weights
            .add_col_with(|_, _| if n == 0 { F::one() } else { F::zero() });
        self.wfa
            .final_weights
            .add_row_with(|_, _| self.automaton.behavior(prefix.iter().cloned()));
        for (x, transition) in self.wfa.transitions.iter_mut().enumerate() {
            transition.add_col_with(|_, _| F::zero());
            transition.add_row_with(|_, _| F::zero());
            for i in 0..=n {
                for j in 0..=n {
                    if i == n || j == n {
                        for k in 0..=n {
                            if i != n && j != n && k != n {
                                continue;
                            }
                            F::add_assign(
                                &mut transition[i][k],
                                &F::mul(&self.nh[x][i][j], &self.inv_h[j][k]),
                            );
                        }
                    } else {
                        let k = n;
                        F::add_assign(
                            &mut transition[i][k],
                            &F::mul(&self.nh[x][i][j], &self.inv_h[j][k]),
                        );
                    }
                }
            }
        }
        true
    }
    pub fn train(&mut self, samples: impl IntoIterator<Item = Vec<usize>>) {
        for sample in samples {
            self.train_sample(&sample);
        }
    }
    pub fn batch_train(&mut self, samples: impl IntoIterator<Item = Vec<usize>>) {
        let mut prefix_set: HashSet<_> = self.prefixes.iter().cloned().collect();
        let mut suffix_set: HashSet<_> = self.suffixes.iter().cloned().collect();
        for sample in samples {
            if prefix_set.insert(sample.to_vec()) {
                self.prefixes.push(sample.to_vec());
            }
            if suffix_set.insert(sample.to_vec()) {
                self.suffixes.push(sample);
            }
        }
        let mut h = Matrix::<F>::new_with((self.prefixes.len(), self.suffixes.len()), |i, j| {
            self.automaton.behavior(
                self.prefixes[i]
                    .iter()
                    .cloned()
                    .chain(self.suffixes[j].iter().cloned()),
            )
        });
        if !self.prefixes.is_empty() && !self.suffixes.is_empty() && F::is_zero(&h[0][0]) {
            for j in 1..self.suffixes.len() {
                if !F::is_zero(&h[0][j]) {
                    self.suffixes.swap(0, j);
                    for row in &mut h.data {
                        row.swap(0, j);
                    }
                    break;
                }
            }
        }
        let mut row_id: Vec<usize> = (0..h.shape.0).collect();
        let mut pivots = vec![];
        h.row_reduction_with(false, |r, p, c| {
            row_id.swap(r, p);
            pivots.push((row_id[r], c));
        });
        let mut new_prefixes = vec![];
        let mut new_suffixes = vec![];
        for (i, j) in pivots {
            new_prefixes.push(self.prefixes[i].clone());
            new_suffixes.push(self.suffixes[j].clone());
        }
        self.prefixes = new_prefixes;
        self.suffixes = new_suffixes;
        assert_eq!(self.prefixes.len(), self.suffixes.len());
        let n = self.prefixes.len();
        let h = Matrix::<F>::new_with((n, n), |i, j| {
            self.automaton.behavior(
                self.prefixes[i]
                    .iter()
                    .cloned()
                    .chain(self.suffixes[j].iter().cloned()),
            )
        });
        self.inv_h = h.inverse().expect("Hankel matrix must be invertible");
        self.wfa = WeightedFiniteAutomaton::<F> {
            initial_weights: Matrix::new_with((1, n), |_, j| {
                if self.prefixes[j].is_empty() {
                    F::one()
                } else {
                    F::zero()
                }
            }),
            transitions: (0..self.automaton.sigma())
                .map(|x| {
                    &Matrix::new_with((n, n), |i, j| {
                        self.automaton.behavior(
                            self.prefixes[i]
                                .iter()
                                .cloned()
                                .chain([x])
                                .chain(self.suffixes[j].iter().cloned()),
                        )
                    }) * &self.inv_h
                })
                .collect(),
            final_weights: Matrix::new_with((n, 1), |i, _| {
                self.automaton.behavior(self.prefixes[i].iter().cloned())
            }),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation,
        num::mint_basic::MInt998244353 as M,
        tools::{
            Xorshift,
            testutil::{exhaustive_sequences, sample_usize, structured_sequences},
        },
    };
    use std::{collections::VecDeque, iter::repeat_n};

    #[test]
    fn test_dense_sampling() {
        for (base, max_len) in (1usize..=10).flat_map(|base| (0..=5).map(move |len| (base, len))) {
            let mut expected = Vec::new();
            for len in 0..=max_len {
                for mut code in 0..base.pow(len as u32) {
                    let mut word = vec![0; len];
                    for x in word.iter_mut().rev() {
                        *x = code % base;
                        code /= base;
                    }
                    expected.push(word);
                }
            }
            assert_eq!(dense_sampling(base, max_len).collect::<Vec<_>>(), expected);
        }
    }

    #[test]
    fn test_random_sampling() {
        for sigma in 1..=10 {
            for min_len in 0..=16 {
                for max_len in min_len..=16 {
                    let samples: Vec<_> = random_sampling(sigma, min_len..=max_len, f64::INFINITY)
                        .take(100)
                        .collect();
                    assert_eq!(samples.len(), 100);
                    assert!(
                        samples
                            .iter()
                            .all(|s| (min_len..=max_len).contains(&s.len())
                                && s.iter().all(|&x| x < sigma))
                    );
                }
            }
        }
    }

    #[test]
    fn test_lstar_transition_tables() {
        let mut rng = Xorshift::default();
        let mut cases = Vec::new();
        // All transition tables, initial states and accepting sets through two states.
        for n in 1usize..=2 {
            for sigma in 1..=3 {
                for table in exhaustive_sequences(0..n, n * sigma..=n * sigma) {
                    for accepting in exhaustive_sequences([false, true], n..=n) {
                        for initial in 0..n {
                            cases.push((n, sigma, table.clone(), accepting.clone(), initial));
                        }
                    }
                }
            }
        }
        for n in 3usize..=4 {
            for sigma in 1..=3 {
                for _ in 0..100 {
                    cases.push((
                        n,
                        sigma,
                        rng.random_iter(0..n).take(n * sigma).collect(),
                        (0..n).map(|_| rng.random(0..2) == 0).collect(),
                        rng.random(0..n),
                    ));
                }
            }
        }
        for (n, sigma, table, accepting, initial) in cases {
            let automaton = BlackBoxAutomatonImpl::new(sigma, |input| {
                let state = input
                    .iter()
                    .fold(initial, |state, &x| table[state * sigma + x]);
                accepting[state]
            });
            let dfa = DfaLearning::new(&automaton).train(dense_sampling(sigma, 2 * n));
            // Exhaust reachable pairs of states: agreement here proves equivalence
            // for every input word, with no bound on the word length.
            let mut seen = HashSet::new();
            let mut queue = VecDeque::from([(initial, dfa.initial_state)]);
            while let Some((expected, actual)) = queue.pop_front() {
                if !seen.insert((expected, actual)) {
                    continue;
                }
                assert_eq!(
                    accepting[expected],
                    dfa.accept(actual),
                    "table={table:?}, accepting={accepting:?}, initial={initial}"
                );
                for x in 0..sigma {
                    queue.push_back((table[expected * sigma + x], dfa.delta(actual, x)));
                }
            }
        }
    }

    #[test]
    fn test_wfa_transition_matrices() {
        let mut rng = Xorshift::default();
        let mut cases = Vec::new();
        for (n, sigma) in [(1, 1), (1, 2), (2, 1)] {
            let coefficients = 2 * n + sigma * n * n;
            for values in exhaustive_sequences(0u32..=1, coefficients..=coefficients) {
                cases.push((n, sigma, values));
            }
        }
        for n in 1usize..=4 {
            for sigma in 1..=3 {
                let coefficients = 2 * n + sigma * n * n;
                for nonzero in 0..=4 {
                    for _ in 0..2 {
                        let values = (0..coefficients)
                            .map(|_| {
                                if rng.random(0..4) < nonzero {
                                    rng.random(0..M::get_mod())
                                } else {
                                    0
                                }
                            })
                            .collect();
                        cases.push((n, sigma, values));
                    }
                }
            }
        }
        for (n, sigma, values) in cases {
            let initial: Vec<_> = values[..n].iter().copied().map(M::from).collect();
            let final_weights: Vec<_> = values[n..2 * n].iter().copied().map(M::from).collect();
            let transitions: Vec<_> = values[2 * n..].iter().copied().map(M::from).collect();
            let automaton = BlackBoxAutomatonImpl::new(sigma, |input| {
                let mut state = initial.clone();
                for x in input {
                    state = (0..n)
                        .map(|j| {
                            (0..n)
                                .map(|i| state[i] * transitions[x * n * n + i * n + j])
                                .sum()
                        })
                        .collect();
                }
                state
                    .iter()
                    .zip(&final_weights)
                    .map(|(&a, &b)| a * b)
                    .sum::<M>()
            });
            let lengths = sample_usize(&mut rng, 8, 0..=64, 100);
            check_wfa(
                &automaton,
                2 * n,
                n,
                dense_sampling(sigma, 6).chain(structured_sequences(&mut rng, 0..sigma, lengths)),
            );
        }
    }

    fn check_wfa(
        automaton: &impl BlackBoxAutomaton<Output = M>,
        training_len: usize,
        batch_len: usize,
        samples: impl IntoIterator<Item = Vec<usize>>,
    ) {
        let mut incremental = WfaLearning::<AddMulOperation<_>, _>::new(automaton);
        // One counterexample can require several rank refinements, even when
        // every longer word evaluates to zero (for example a nilpotent matrix).
        incremental.train(
            dense_sampling(automaton.sigma(), training_len)
                .flat_map(|sample| repeat_n(sample, batch_len.max(1))),
        );
        let mut batch = WfaLearning::<AddMulOperation<_>, _>::new(automaton);
        batch.batch_train(dense_sampling(automaton.sigma(), batch_len));
        for sample in samples {
            let expected = automaton.behavior(sample.iter().copied());
            assert_eq!(
                incremental.wfa().behavior(sample.iter().copied()),
                expected,
                "input={sample:?}"
            );
            assert_eq!(
                batch.wfa().behavior(sample.iter().copied()),
                expected,
                "input={sample:?}"
            );
        }
    }

    #[test]
    fn test_lstar_modular_languages() {
        for modulus in 1..=8 {
            for sigma in 1..=3 {
                for by_length in [false, true] {
                    let automaton = BlackBoxAutomatonImpl::new(sigma, |input| {
                        let value = if by_length {
                            input.len()
                        } else {
                            input.iter().sum()
                        };
                        value % modulus == 0
                    });
                    // A unary language may expose only one counterexample per
                    // pass. At most `modulus` state refinements are needed.
                    let training: Vec<_> = dense_sampling(sigma, modulus).collect();
                    let dfa = DfaLearning::new(&automaton)
                        .train((0..modulus).flat_map(|_| training.iter().cloned()));
                    for input in dense_sampling(sigma, if sigma == 3 { 8 } else { 12 }) {
                        assert_eq!(
                            dfa.behavior(input.iter().copied()),
                            automaton.behavior(input.iter().copied()),
                            "modulus={modulus}, sigma={sigma}, length={by_length}, input={input:?}"
                        );
                    }
                }
            }
        }
    }

    // Each cell contains all results of fully parenthesizing one substring.
    // This interval DP is independent of both automaton learning algorithms.
    fn reduction_results(input: &[usize], table: &[usize]) -> Vec<Vec<u8>> {
        let n = input.len();
        let mut results = vec![vec![0; n + 1]; n + 1];
        for (i, &x) in input.iter().enumerate() {
            results[i][i + 1] = 1 << x;
        }
        for len in 2..=n {
            for l in 0..=n - len {
                let r = l + len;
                for m in l + 1..r {
                    for a in 0..2 {
                        for b in 0..2 {
                            if results[l][m] >> a & results[m][r] >> b & 1 != 0 {
                                results[l][r] |= 1 << table[a * 2 + b];
                            }
                        }
                    }
                }
            }
        }
        results
    }

    #[test]
    fn test_lstar_reduction_languages() {
        for table in exhaustive_sequences(0..2, 4..=4) {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| {
                reduction_results(&input, &table)[0][input.len()] & 2 != 0
            });
            let dfa = DfaLearning::new(&automaton).train(dense_sampling(2, 4));
            for input in dense_sampling(2, 12) {
                assert_eq!(
                    dfa.behavior(input.iter().copied()),
                    automaton.behavior(input.iter().copied()),
                    "table={table:?}, input={input:?}"
                );
            }
        }
    }

    #[test]
    fn test_wfa_position_weights() {
        let mut rng = Xorshift::default();
        for sigma in 1..=3 {
            for ratio in -2..=2 {
                let automaton = BlackBoxAutomatonImpl::new(sigma, |input| {
                    let mut weight = M::from(1);
                    let mut sum = M::from(0);
                    for x in input {
                        sum += M::from(x) * weight;
                        weight *= M::from(ratio);
                    }
                    sum
                });
                let lengths = sample_usize(&mut rng, 16, 0..=64, 1000);
                check_wfa(
                    &automaton,
                    4,
                    3,
                    dense_sampling(sigma, if sigma == 3 { 8 } else { 12 })
                        .chain(structured_sequences(&mut rng, 0..sigma, lengths)),
                );
            }
        }
    }

    #[test]
    fn test_wfa_xor_sum() {
        // For sum = a + b, a ^ b <= sum. Enumerate distinct XOR results at
        // each sum, then prefix-sum their counts for every n < 2^13.
        let mut counts = vec![0usize; 1 << 13];
        for sum in 0..counts.len() {
            let mut seen = vec![false; sum + 1];
            for a in 0..=sum {
                seen[a ^ (sum - a)] = true;
            }
            counts[sum] = seen.into_iter().filter(|&x| x).count();
            if sum != 0 {
                counts[sum] += counts[sum - 1];
            }
        }
        let automaton = BlackBoxAutomatonImpl::new(2, |input| {
            let n = input.into_iter().fold(1, |n, x| n * 2 + x);
            M::from(counts[n])
        });
        check_wfa(&automaton, 4, 3, dense_sampling(2, 12));
    }

    #[test]
    fn test_wfa_reduction_substrings() {
        for table in exhaustive_sequences(0..2, 4..=4) {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| {
                M::from(
                    reduction_results(&input, &table)
                        .iter()
                        .flatten()
                        .filter(|&&mask| mask & 2 != 0)
                        .count(),
                )
            });
            check_wfa(&automaton, 6, 3, dense_sampling(2, 12));
        }
    }
}
