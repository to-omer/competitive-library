use crate::algebra::Monoid;

#[snippet::entry("automaton")]
pub trait Automaton {
    type Alphabet;
    type State;
    type Effect;

    fn initial(&self) -> Self::State;
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)>;
    fn accept(&self, state: &Self::State) -> bool;
}

#[snippet::entry("automaton")]
pub fn automaton_dp<A, M>(
    dfa: A,
    sigma: impl Iterator<Item = A::Alphabet> + Clone,
    len: usize,
    monoid: M,
    mul: impl Fn(&M::T, &A::Effect) -> M::T,
    init: M::T,
) -> M::T
where
    A: Automaton,
    A::State: Eq + std::hash::Hash,
    M: Monoid,
{
    let mut dp = std::collections::HashMap::new();
    let mut ndp = std::collections::HashMap::new();
    dp.insert(dfa.initial(), init);
    for _ in 0..len {
        for (state, value) in dp.drain() {
            for alph in sigma.clone() {
                if let Some((nstate, eff)) = dfa.next(&state, &alph) {
                    let nvalue = mul(&value, &eff);
                    ndp.entry(nstate)
                        .and_modify(|acc| *acc = monoid.operate(acc, &nvalue))
                        .or_insert(nvalue);
                }
            }
        }
        std::mem::swap(&mut dp, &mut ndp);
        ndp.clear();
    }
    let mut acc = monoid.unit();
    for (state, value) in dp.into_iter() {
        if dfa.accept(&state) {
            acc = monoid.operate(&acc, &value);
        }
    }
    acc
}

#[snippet::entry("automaton")]
pub struct IntersectionAutomaton<X: Automaton, Y: Automaton>(X, Y);
#[snippet::entry("automaton")]
impl<A, X, Y> Automaton for IntersectionAutomaton<X, Y>
where
    X: Automaton<Alphabet = A>,
    Y: Automaton<Alphabet = A>,
{
    type Alphabet = A;
    type State = (X::State, Y::State);
    type Effect = (X::Effect, Y::Effect);
    fn initial(&self) -> Self::State {
        (self.0.initial(), self.1.initial())
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        match (self.0.next(&state.0, alph), self.1.next(&state.1, alph)) {
            (Some((s0, e0)), Some((s1, e1))) => Some(((s0, s1), (e0, e1))),
            _ => None,
        }
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.0.accept(&state.0) && self.1.accept(&state.1)
    }
}

#[snippet::entry("automaton")]
pub struct UnionAutomaton<X: Automaton, Y: Automaton>(X, Y);
#[snippet::entry("automaton")]
impl<A, X, Y> Automaton for UnionAutomaton<X, Y>
where
    X: Automaton<Alphabet = A>,
    Y: Automaton<Alphabet = A>,
{
    type Alphabet = A;
    type State = (X::State, Y::State);
    type Effect = (X::Effect, Y::Effect);
    fn initial(&self) -> Self::State {
        (self.0.initial(), self.1.initial())
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        match (self.0.next(&state.0, alph), self.1.next(&state.1, alph)) {
            (Some((s0, e0)), Some((s1, e1))) => Some(((s0, s1), (e0, e1))),
            _ => None,
        }
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.0.accept(&state.0) || self.1.accept(&state.1)
    }
}

#[snippet::entry("automaton")]
pub struct ProductAutomaton<X: Automaton, Y: Automaton>(X, Y);
#[snippet::entry("automaton")]
impl<X: Automaton, Y: Automaton> Automaton for ProductAutomaton<X, Y> {
    type Alphabet = (X::Alphabet, Y::Alphabet);
    type State = (X::State, Y::State);
    type Effect = (X::Effect, Y::Effect);
    fn initial(&self) -> Self::State {
        (self.0.initial(), self.1.initial())
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        match (
            self.0.next(&state.0, &alph.0),
            self.1.next(&state.1, &alph.1),
        ) {
            (Some((s0, e0)), Some((s1, e1))) => Some(((s0, s1), (e0, e1))),
            _ => None,
        }
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.0.accept(&state.0) && self.1.accept(&state.1)
    }
}

#[snippet::entry("automaton")]
pub struct LessThanAutomaton<'a, T: Ord> {
    buf: &'a [T],
    eq: bool,
}
#[snippet::entry("automaton")]
impl<'a, T: Ord> LessThanAutomaton<'a, T> {
    pub fn new(buf: &'a [T], eq: bool) -> Self {
        Self { buf, eq }
    }
}
#[snippet::entry("automaton")]
impl<T: Ord> Automaton for LessThanAutomaton<'_, T> {
    type Alphabet = T;
    type State = (usize, bool);
    type Effect = ();
    fn initial(&self) -> Self::State {
        (0, true)
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        self.buf
            .get(state.0)
            .and_then(|c| match (state.1, c.cmp(alph)) {
                (true, std::cmp::Ordering::Equal) => Some(((state.0 + 1, true), ())),
                (true, std::cmp::Ordering::Less) => None,
                _ => Some(((state.0 + 1, false), ())),
            })
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.eq || !state.1
    }
}

#[snippet::entry("automaton")]
pub struct GreaterThanAutomaton<'a, T: Ord> {
    buf: &'a [T],
    eq: bool,
}
#[snippet::entry("automaton")]
impl<'a, T: Ord> GreaterThanAutomaton<'a, T> {
    pub fn new(buf: &'a [T], eq: bool) -> Self {
        Self { buf, eq }
    }
}
#[snippet::entry("automaton")]
impl<T: Ord> Automaton for GreaterThanAutomaton<'_, T> {
    type Alphabet = T;
    type State = (usize, bool);
    type Effect = ();
    fn initial(&self) -> Self::State {
        (0, true)
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        self.buf
            .get(state.0)
            .and_then(|c| match (state.1, c.cmp(alph)) {
                (true, std::cmp::Ordering::Equal) => Some(((state.0 + 1, true), ())),
                (true, std::cmp::Ordering::Greater) => None,
                _ => Some(((state.0 + 1, false), ())),
            })
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.eq || !state.1
    }
}

#[snippet::entry("automaton")]
pub struct ContainAutomaton<'a, T: Eq>(&'a T);
#[snippet::entry("automaton")]
impl<'a, T: Eq> Automaton for ContainAutomaton<'a, T> {
    type Alphabet = T;
    type State = bool;
    type Effect = bool;
    fn initial(&self) -> Self::State {
        false
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        Some((*state || self.0 == alph, *state ^ (self.0 == alph)))
    }
    fn accept(&self, state: &Self::State) -> bool {
        *state
    }
}

#[snippet::entry("automaton")]
pub struct ContainCounterAutomaton<'a, T: Eq>(&'a T);
#[snippet::entry("automaton")]
impl<'a, T: Eq> Automaton for ContainCounterAutomaton<'a, T> {
    type Alphabet = T;
    type State = usize;
    type Effect = usize;
    fn initial(&self) -> Self::State {
        0
    }
    fn next(
        &self,
        state: &Self::State,
        alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        let nstate = *state + (self.0 == alph) as usize;
        Some((nstate, nstate))
    }
    fn accept(&self, state: &Self::State) -> bool {
        *state > 0
    }
}

#[snippet::entry("automaton")]
#[derive(Debug, Clone)]
pub struct AlwaysAcceptingAutomaton<A>(std::marker::PhantomData<fn() -> A>);
#[snippet::entry("automaton")]
impl<A> AlwaysAcceptingAutomaton<A> {
    pub fn new() -> Self {
        Default::default()
    }
}
#[snippet::entry("automaton")]
impl<A> Default for AlwaysAcceptingAutomaton<A> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
#[snippet::entry("automaton")]
impl<A> Automaton for AlwaysAcceptingAutomaton<A> {
    type Alphabet = A;
    type State = ();
    type Effect = ();
    fn initial(&self) -> Self::State {}
    fn next(
        &self,
        _state: &Self::State,
        _alph: &Self::Alphabet,
    ) -> Option<(Self::State, Self::Effect)> {
        Some(((), ()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}
