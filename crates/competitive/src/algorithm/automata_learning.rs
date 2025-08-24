use super::{BitSet, Field, Invertible, Matrix, SerdeByteStr};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
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

impl<F> WeightedFiniteAutomaton<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
{
    fn new(sigma: usize) -> Self {
        Self {
            initial_weights: Matrix::from_vec(vec![]),
            transitions: vec![Matrix::from_vec(vec![]); sigma],
            final_weights: Matrix::from_vec(vec![]),
        }
    }
}

impl<F> BlackBoxAutomaton for WeightedFiniteAutomaton<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: Debug,
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

#[derive(Debug, Clone)]
struct InputTraversal {
    sigma: usize,
    current: Vec<usize>,
}

impl InputTraversal {
    fn new(sigma: usize) -> Self {
        Self {
            sigma,
            current: Vec::new(),
        }
    }
    fn next(&mut self) {
        let mut carry = true;
        for i in (0..self.current.len()).rev() {
            self.current[i] += 1;
            if self.current[i] == self.sigma {
                self.current[i] = 0;
            } else {
                carry = false;
                break;
            }
        }
        if carry {
            self.current.push(0);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ObservationTable {
    pub prefixes: Vec<Vec<usize>>,
    pub suffixes: Vec<Vec<usize>>,
    pub table: Vec<BitSet>,
    pub row_map: HashMap<BitSet, usize>,
}

impl ObservationTable {
    pub fn add_prefix<A>(&mut self, automaton: A, prefix: Vec<usize>) -> usize
    where
        A: BlackBoxAutomaton<Output = bool>,
    {
        let row: BitSet = self
            .suffixes
            .iter()
            .map(|s| automaton.behavior(prefix.iter().cloned().chain(s.iter().cloned())))
            .collect();
        *self.row_map.entry(row.clone()).or_insert_with(|| {
            let idx = self.table.len();
            self.table.push(row);
            self.prefixes.push(prefix);
            idx
        })
    }
    pub fn add_suffix<A>(&mut self, automaton: A, suffix: Vec<usize>)
    where
        A: BlackBoxAutomaton<Output = bool>,
    {
        if self.suffixes.contains(&suffix) {
            return;
        }
        for (prefix, table) in self.prefixes.iter_mut().zip(&mut self.table) {
            table.push(automaton.behavior(prefix.iter().cloned().chain(suffix.iter().cloned())));
        }
        self.suffixes.push(suffix);
        self.row_map.clear();
        for (i_prefix, row) in self.table.iter().enumerate() {
            self.row_map.insert(row.clone(), i_prefix);
        }
    }
}

pub fn angluin_lstar<A, F>(automaton: A, terminate: F) -> DeterministicFiniteAutomaton
where
    A: BlackBoxAutomaton<Output = bool>,
    F: Fn(&ObservationTable, &[usize]) -> bool,
{
    let sigma = automaton.sigma();
    assert_ne!(sigma, 0, "Sigma must be greater than 0");
    let mut observation_table = ObservationTable::default();
    observation_table.add_suffix(&automaton, vec![]);
    observation_table.add_prefix(&automaton, vec![]);
    let mut traversal = InputTraversal::new(sigma);

    loop {
        let mut dfa = DeterministicFiniteAutomaton {
            states: vec![],
            initial_state: 0,
        };
        // close
        {
            let mut i_prefix = 0;
            while i_prefix < observation_table.prefixes.len() {
                let mut delta = vec![];
                for x in 0..sigma {
                    let prefix: Vec<usize> = observation_table.prefixes[i_prefix]
                        .iter()
                        .cloned()
                        .chain([x])
                        .collect();
                    let index = observation_table.add_prefix(&automaton, prefix);
                    delta.push(index);
                }
                dfa.states.push(DfaState {
                    delta,
                    accept: observation_table.table[i_prefix].get(0),
                });
                i_prefix += 1;
            }
        }
        // equiv
        let (counterexample, accepted) = {
            loop {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                if expected != result {
                    break (traversal.current.to_vec(), expected);
                }
                traversal.next();
                if terminate(&observation_table, &traversal.current) {
                    return dfa;
                }
            }
        };
        // split
        {
            let mut state = 0;
            for i in 0..counterexample.len() {
                state = dfa.states[state].delta[counterexample[i]];
                let result = automaton.behavior(
                    observation_table.prefixes[state]
                        .iter()
                        .cloned()
                        .chain(counterexample[i + 1..].iter().cloned()),
                );
                if accepted != result {
                    let new_prefix = counterexample[..=i].to_vec();
                    let new_suffix = counterexample[i + 1..].to_vec();
                    observation_table.add_suffix(&automaton, new_suffix);
                    observation_table.add_prefix(&automaton, new_prefix);
                    break;
                }
            }
        }
    }
}

pub struct WeightedObservationTable<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
{
    pub prefixes: Vec<Vec<usize>>,
    pub suffixes: Vec<Vec<usize>>,
    _marker: PhantomData<fn() -> F>,
}

impl<F: Debug> Debug for WeightedObservationTable<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeightedObservationTable")
            .field("prefixes", &self.prefixes)
            .field("suffixes", &self.suffixes)
            .finish()
    }
}

impl<F> Clone for WeightedObservationTable<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
{
    fn clone(&self) -> Self {
        Self {
            prefixes: self.prefixes.clone(),
            suffixes: self.suffixes.clone(),
            _marker: self._marker,
        }
    }
}

impl<F> Default for WeightedObservationTable<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
{
    fn default() -> Self {
        Self {
            prefixes: Default::default(),
            suffixes: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<F> WeightedObservationTable<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: PartialEq,
{
    fn construct_wfa<A>(&self, automaton: A) -> WeightedFiniteAutomaton<F>
    where
        A: BlackBoxAutomaton<Output = F::T>,
        F::T: Debug,
    {
        let n = self.prefixes.len();
        assert_eq!(self.suffixes.len(), n);
        let table = Matrix::from_fn((n, n), |i, j| {
            automaton.behavior(
                self.prefixes[i]
                    .iter()
                    .cloned()
                    .chain(self.suffixes[j].iter().cloned()),
            )
        });
        let inv = table
            .inverse()
            .expect("Observation table is not invertible");
        WeightedFiniteAutomaton::<F> {
            initial_weights: Matrix::from_fn(
                (1, n),
                |_, j| {
                    if j == 0 { F::one() } else { F::zero() }
                },
            ),
            transitions: (0..automaton.sigma())
                .map(|x| {
                    &Matrix::from_fn((n, n), |i, j| {
                        automaton.behavior(
                            self.prefixes[i]
                                .iter()
                                .cloned()
                                .chain([x])
                                .chain(self.suffixes[j].iter().cloned()),
                        )
                    }) * &inv
                })
                .collect(),
            final_weights: Matrix::from_fn((n, 1), |i, _| {
                automaton.behavior(self.prefixes[i].iter().cloned())
            }),
        }
    }
}

pub fn wfa_learning<F, A, T>(automaton: A, terminate: T) -> WeightedFiniteAutomaton<F>
where
    F: Field,
    F::Additive: Invertible,
    F::Multiplicative: Invertible,
    F::T: PartialEq,
    A: BlackBoxAutomaton<Output = F::T>,
    T: Fn(&WeightedObservationTable<F>, &[usize]) -> bool,
    F::T: Debug,
{
    let sigma = automaton.sigma();
    assert_ne!(sigma, 0, "Sigma must be greater than 0");
    let mut observation_table = WeightedObservationTable::<F>::default();
    {
        let mut traversal = InputTraversal::new(sigma);
        loop {
            if automaton.behavior(traversal.current.iter().cloned()) != F::zero() {
                observation_table.prefixes.push(vec![]);
                observation_table.suffixes.push(traversal.current.clone());
                break;
            }
            traversal.next();
            if terminate(&observation_table, &traversal.current) {
                return WeightedFiniteAutomaton::new(sigma);
            }
        }
    }
    let mut traversal = InputTraversal::new(sigma);

    loop {
        let wfa = observation_table.construct_wfa(&automaton);
        // equiv
        let counterexample = {
            loop {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = wfa.behavior(traversal.current.iter().cloned());
                if expected != result {
                    break traversal.current.clone();
                }
                traversal.next();
                if terminate(&observation_table, &traversal.current) {
                    return wfa;
                }
            }
        };
        // split
        {
            let mut state = wfa.final_weights.clone();
            for i in (0..counterexample.len()).rev() {
                state = &wfa.transitions[counterexample[i]] * &state;
                if (0..state.shape.0).any(|j| {
                    let result = automaton.behavior(
                        observation_table.prefixes[j]
                            .iter()
                            .cloned()
                            .chain(counterexample[i..].iter().cloned()),
                    );
                    state[j][0] != result
                }) {
                    observation_table
                        .prefixes
                        .push(counterexample[..=i].to_vec());
                    observation_table
                        .suffixes
                        .push(counterexample[i + 1..].to_vec());
                    break;
                }
            }
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
    fn test_input_traversal() {
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

            let mut traversal = InputTraversal::new(base);
            for e in expected {
                assert_eq!(traversal.current, e);
                traversal.next();
            }
        }
    }

    #[test]
    fn test_lstar() {
        {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| input.len() % 6 == 0);
            let dfa = angluin_lstar(&automaton, |_, input| input.len() > 6);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 12 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
            }
        }
        {
            let automaton =
                BlackBoxAutomatonImpl::new(3, |input| input.iter().sum::<usize>() % 4 == 0);
            let dfa = angluin_lstar(&automaton, |_, input| input.len() > 3);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 8 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
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
            let dfa = angluin_lstar(&automaton, |_, t| t.len() > 4);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 8 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
            }
        }
    }

    #[test]
    fn test_wfa_learning() {
        {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| {
                MInt998244353::from(input.iter().sum::<usize>())
            });
            let dfa =
                wfa_learning::<AddMulOperation<_>, _, _>(&automaton, |_, input| input.len() > 3);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 12 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
            }
        }
        {
            let automaton = BlackBoxAutomatonImpl::new(2, |input| {
                let mut s = MInt998244353::zero();
                let mut c = MInt998244353::one();
                for &x in &input {
                    s += MInt998244353::from(x) * c;
                    c = -c;
                }
                s
            });
            let dfa =
                wfa_learning::<AddMulOperation<_>, _, _>(&automaton, |_, input| input.len() > 4);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 12 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
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
            let dfa =
                wfa_learning::<AddMulOperation<_>, _, _>(&automaton, |_, input| input.len() > 4);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 6 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
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
            let dfa = wfa_learning::<AddMulOperation<_>, _, _>(&automaton, |_, t| t.len() > 6);
            let mut traversal = InputTraversal::new(automaton.sigma());
            while traversal.current.len() <= 8 {
                let expected = automaton.behavior(traversal.current.iter().cloned());
                let result = dfa.behavior(traversal.current.iter().cloned());
                assert_eq!(expected, result);
                traversal.next();
            }
        }
    }
}
