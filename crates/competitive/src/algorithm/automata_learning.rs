use super::{BitSet, Field, Invertible, Matrix, RandomSpec, SerdeByteStr, Xorshift};
use std::{
    cell::RefCell,
    collections::HashMap,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
{
    pub initial_weights: Matrix<F>,
    pub transitions: Vec<Matrix<F>>,
    pub final_weights: Matrix<F>,
}

impl<F> Debug for WeightedFiniteAutomaton<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: Debug,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: SerdeByteStr,
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
        let result = dfa.behavior(sample.iter().cloned());
        if expected == result {
            return false;
        }
        let mut state = 0;
        for i in 0..sample.len() {
            state = dfa.states[state].delta[sample[i]];
            let result = self.automaton.behavior(
                self.prefixes[state]
                    .iter()
                    .cloned()
                    .chain(sample[i + 1..].iter().cloned()),
            );
            if expected != result {
                let new_prefix = sample[..=i].to_vec();
                let new_suffix = sample[i + 1..].to_vec();
                self.add_suffix(new_suffix);
                self.add_prefix(new_prefix);
                break;
            }
        }
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: Debug,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
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
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: PartialEq,
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
        let result = self.wfa.behavior(sample.iter().cloned());
        if expected == result {
            return None;
        }
        let mut state = self.wfa.final_weights.clone();
        for i in (0..sample.len()).rev() {
            state = &self.wfa.transitions[sample[i]] * &state;
            if (0..state.shape.0).any(|j| {
                let result = self.automaton.behavior(
                    self.prefixes[j]
                        .iter()
                        .cloned()
                        .chain(sample[i..].iter().cloned()),
                );
                state[j][0] != result
            }) {
                return Some((sample[..=i].to_vec(), sample[i + 1..].to_vec()));
            }
        }
        unreachable!("Failed to split sample");
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::AddMulOperation,
        num::{One as _, Zero as _, mint_basic::MInt998244353},
    };
    use std::collections::{HashSet, VecDeque};

    #[test]
    fn test_dense_sampling() {
        for base in 1usize..=10 {
            let mut expected = vec![];
            for len in 0..=3 {
                for n in 0..base.pow(len) {
                    let mut n = n;
                    let mut current = vec![];
                    for _ in 0..len {
                        current.push(n % base);
                        n /= base;
                    }
                    current.reverse();
                    expected.push(current);
                }
            }

            for (expected, result) in expected.into_iter().zip(dense_sampling(base, 3)) {
                assert_eq!(expected, result);
            }
        }
    }

    #[test]
    fn test_lstar() {
        {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| input.len() % 6 == 0);
            let dfa = DfaLearning::new(&automaton).train(dense_sampling(2, 6));
            for sample in dense_sampling(automaton.sigma(), 12) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = dfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
        {
            let automaton =
                BlackBoxAutomatonImpl::new(3, |input| input.iter().sum::<usize>() % 4 == 0);
            let dfa = DfaLearning::new(&automaton).train(dense_sampling(3, 4));
            for sample in dense_sampling(automaton.sigma(), 8) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = dfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
        for i in 0usize..16 {
            let a = i >> 3 & 1;
            let b = i >> 2 & 1;
            let c = i >> 1 & 1;
            let d = i & 1;
            let naive = |t: &[usize]| {
                let mut set = HashSet::new();
                let mut deq = VecDeque::new();
                deq.push_back(t.to_vec());
                set.insert(t.to_vec());
                while let Some(t) = deq.pop_front() {
                    for i in 0..t.len().saturating_sub(1) {
                        let x = match (t[i], t[i + 1]) {
                            (0, 0) => a,
                            (0, 1) => b,
                            (1, 0) => c,
                            (1, 1) => d,
                            _ => unreachable!(),
                        };
                        let mut t = t.to_vec();
                        t.remove(i);
                        t[i] = x;
                        if set.insert(t.to_vec()) {
                            deq.push_back(t);
                        }
                    }
                }
                set.contains(&vec![1])
            };
            let automaton = BlackBoxAutomatonImpl::new(2, |t| naive(&t));
            let dfa = DfaLearning::new(&automaton).train(dense_sampling(2, 4));
            for sample in dense_sampling(automaton.sigma(), 8) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = dfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
    }

    #[test]
    fn test_wfa_learning() {
        {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| {
                MInt998244353::from(input.iter().sum::<usize>())
            });
            let mut wl = WfaLearning::<AddMulOperation<_>, _>::new(&automaton);
            wl.train(dense_sampling(2, 3));
            let wfa = wl.wfa();
            for sample in dense_sampling(automaton.sigma(), 12) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = wfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
        {
            let automaton = BlackBoxAutomatonImpl::new(3, |input| {
                let mut s = MInt998244353::zero();
                let mut c = MInt998244353::one();
                for &x in &input {
                    s += MInt998244353::from(x) * c;
                    c = -c;
                }
                s
            });
            let mut wl = WfaLearning::<AddMulOperation<_>, _>::new(&automaton);
            wl.train(dense_sampling(3, 4));
            let wfa = wl.wfa();
            for sample in dense_sampling(automaton.sigma(), 6).chain(random_sampling(
                automaton.sigma(),
                6..=12,
                0.1,
            )) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = wfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
        {
            // Xor Sum
            let automaton = BlackBoxAutomatonImpl::new(2, |input| {
                let mut n = 1; // prevent leading zero
                for x in input {
                    n = n * 2 + x;
                }
                let mut s = MInt998244353::zero();
                for u in 0..=n {
                    for v in 0..=n {
                        let mut ok = false;
                        for a in 0..=n {
                            let b = u ^ a;
                            ok |= a + b == v;
                        }
                        s += MInt998244353::new(ok as _);
                    }
                }
                s
            });
            let mut wl = WfaLearning::<AddMulOperation<_>, _>::new(&automaton);
            wl.train(dense_sampling(2, 4));
            let wfa = wl.wfa();
            for sample in dense_sampling(automaton.sigma(), 6).chain(random_sampling(
                automaton.sigma(),
                6..=12,
                0.1,
            )) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = wfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
        for i in 0usize..16 {
            let a = i >> 3 & 1;
            let b = i >> 2 & 1;
            let c = i >> 1 & 1;
            let d = i & 1;
            let naive = |t: &[usize]| {
                let mut set = HashSet::new();
                let mut deq = VecDeque::new();
                deq.push_back(t.to_vec());
                set.insert(t.to_vec());
                while let Some(t) = deq.pop_front() {
                    for i in 0..t.len().saturating_sub(1) {
                        let x = match (t[i], t[i + 1]) {
                            (0, 0) => a,
                            (0, 1) => b,
                            (1, 0) => c,
                            (1, 1) => d,
                            _ => unreachable!(),
                        };
                        let mut t = t.to_vec();
                        t.remove(i);
                        t[i] = x;
                        if set.insert(t.to_vec()) {
                            deq.push_back(t);
                        }
                    }
                }
                set.contains(&vec![1])
            };
            let naive = |t: &[usize]| {
                let mut s = MInt998244353::zero();
                for l in 0..t.len() {
                    for r in l + 1..=t.len() {
                        if naive(&t[l..r]) {
                            s += MInt998244353::one();
                        }
                    }
                }
                s
            };
            let automaton = BlackBoxAutomatonImpl::new(2, |t| naive(&t));
            let mut wl = WfaLearning::<AddMulOperation<_>, _>::new(&automaton);
            wl.train(dense_sampling(2, 6));
            let wfa = wl.wfa();
            for sample in dense_sampling(automaton.sigma(), 8).chain(random_sampling(
                automaton.sigma(),
                9..=12,
                0.1,
            )) {
                let expected = automaton.behavior(sample.iter().cloned());
                let result = wfa.behavior(sample.iter().cloned());
                assert_eq!(expected, result);
            }
        }
    }
}
