use std::collections::HashMap;

pub trait BlackBoxAutomaton {
    type Output;
    fn sigma(&self) -> usize; // Σ={0,1,...,sigma-1}
    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>;
}

#[derive(Debug, Clone)]
pub struct BlackBoxAutomatonImpl<F>
where
    F: Fn(Vec<usize>) -> bool,
{
    sigma: usize,
    behavior_fn: F,
}

impl<F> BlackBoxAutomatonImpl<F>
where
    F: Fn(Vec<usize>) -> bool,
{
    pub fn new(sigma: usize, behavior_fn: F) -> Self {
        Self { sigma, behavior_fn }
    }
}

impl<F> BlackBoxAutomaton for BlackBoxAutomatonImpl<F>
where
    F: Fn(Vec<usize>) -> bool,
{
    type Output = bool;

    fn sigma(&self) -> usize {
        self.sigma
    }

    fn behavior<I>(&self, input: I) -> Self::Output
    where
        I: IntoIterator<Item = usize>,
    {
        (self.behavior_fn)(input.into_iter().collect())
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
    pub table: Vec<Vec<bool>>,
    pub row_map: HashMap<Vec<bool>, usize>,
}

impl ObservationTable {
    pub fn add_prefix<A>(&mut self, automaton: A, prefix: Vec<usize>) -> usize
    where
        A: BlackBoxAutomaton<Output = bool>,
    {
        let row: Vec<_> = self
            .suffixes
            .iter()
            .map(|s| automaton.behavior(prefix.iter().cloned().chain(s.iter().cloned())))
            .collect();
        *self.row_map.entry(row.to_vec()).or_insert_with(|| {
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
            self.row_map.insert(row.to_vec(), i_prefix);
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
                    accept: observation_table.table[i_prefix][0],
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
