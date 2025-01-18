use super::{
    Container, ContainerEntry, ContainerFactory, FixedVecMapFactory, HashMapFactory, Monoid,
    VecMap, VecMapFactory,
};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    hash::Hash,
    marker::PhantomData,
    mem::swap,
};

pub trait Automaton {
    type Alphabet;
    type State;
    fn initial(&self) -> Self::State;
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State>;
    fn accept(&self, state: &Self::State) -> bool;
    fn dp<M>(&self, init: M::T) -> InitAutomatonDp<M, &Self>
    where
        Self: Sized,
        M: Monoid,
    {
        InitAutomatonDp::new(self, init)
    }
    fn filter<G, H>(self, g: G, h: H) -> FilterAutomaton<Self, G, H>
    where
        Self: Sized,
        G: Fn(&Self::State, &Self::Alphabet) -> bool,
        H: Fn(&Self::State) -> bool,
    {
        FilterAutomaton::new(self, g, h)
    }
    fn map<S, F, G, H>(self, f: F, g: G, h: H) -> MappingAutomaton<Self, S, F, G, H>
    where
        Self: Sized,
        F: Fn() -> S,
        G: Fn(&(Self::State, S), &Self::Alphabet) -> Option<S>,
        H: Fn(&(Self::State, S)) -> bool,
    {
        MappingAutomaton::new(self, f, g, h)
    }
}

impl<A> Automaton for &A
where
    A: Automaton,
{
    type Alphabet = A::Alphabet;
    type State = A::State;
    fn initial(&self) -> Self::State {
        A::initial(self)
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        A::next(self, state, alph)
    }
    fn accept(&self, state: &Self::State) -> bool {
        A::accept(self, state)
    }
}

#[derive(Debug, Clone)]
pub struct InitAutomatonDp<M, A>
where
    M: Monoid,
    A: Automaton,
{
    dfa: A,
    init: M::T,
}

impl<M, A> InitAutomatonDp<M, A>
where
    M: Monoid,
    A: Automaton,
{
    pub fn new(dfa: A, init: M::T) -> Self {
        Self { dfa, init }
    }
    pub fn with_factory<F>(self, factory: F) -> Automatondp<M, A, F::Container>
    where
        F: ContainerFactory,
        F::Container: Container<Key = A::State, Value = M::T>,
    {
        Automatondp::new(self.dfa, self.init, factory)
    }
    pub fn with_hashmap(self) -> Automatondp<M, A, HashMap<A::State, M::T>>
    where
        A::State: Eq + Hash,
    {
        Automatondp::new(self.dfa, self.init, HashMapFactory::default())
    }
    pub fn with_vecmap<F>(
        self,
        key_to_index: F,
    ) -> Automatondp<M, A, VecMap<false, A::State, M::T, F>>
    where
        F: Fn(&A::State) -> usize + Clone,
    {
        Automatondp::new(self.dfa, self.init, VecMapFactory::new(key_to_index))
    }
    pub fn with_fixed_vecmap<F>(
        self,
        key_to_index: F,
        len: usize,
    ) -> Automatondp<M, A, VecMap<true, A::State, M::T, F>>
    where
        F: Fn(&A::State) -> usize + Clone,
    {
        Automatondp::new(
            self.dfa,
            self.init,
            FixedVecMapFactory::new(key_to_index, len),
        )
    }
}

#[derive(Clone)]
pub struct Automatondp<M, A, C>
where
    M: Monoid,
    A: Automaton,
    C: Container<Key = A::State, Value = M::T>,
{
    dfa: A,
    pub dp: C,
    ndp: C,
    _marker: PhantomData<fn() -> M>,
}

impl<M, A, C> Debug for Automatondp<M, A, C>
where
    M: Monoid,
    A: Automaton + Debug,
    A::State: Debug,
    M::T: Debug,
    C: Container<Key = A::State, Value = M::T> + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Automatondp")
            .field("dfa", &self.dfa)
            .field("dp", &self.dp)
            .field("ndp", &self.ndp)
            .finish()
    }
}

impl<M, A, C> Automatondp<M, A, C>
where
    M: Monoid,
    A: Automaton,
    C: Container<Key = A::State, Value = M::T>,
{
    pub fn new<F>(dfa: A, init: M::T, factory: F) -> Self
    where
        F: ContainerFactory<Container = C>,
    {
        let mut dp = factory.create_container();
        let ndp = factory.create_container();
        dp.insert(dfa.initial(), init);
        Self {
            dfa,
            dp,
            ndp,
            _marker: PhantomData,
        }
    }
    pub fn step<S, I, B>(&mut self, mut sigma: S)
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<A::Alphabet>,
    {
        for (state, value) in self.dp.drain() {
            for alph in sigma() {
                if let Some(nstate) = self.dfa.next(&state, alph.borrow()) {
                    self.ndp
                        .entry(nstate)
                        .and_modify(|acc| *acc = M::operate(acc, &value))
                        .or_insert_with(|| value.clone());
                }
            }
        }
        swap(&mut self.dp, &mut self.ndp);
    }
    pub fn step_effect<S, I, B, F>(&mut self, mut sigma: S, mut effect: F)
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<A::Alphabet>,
        F: FnMut(&M::T, &A::State, &A::Alphabet, &A::State) -> M::T,
    {
        for (state, value) in self.dp.drain() {
            for alph in sigma() {
                if let Some(nstate) = self.dfa.next(&state, alph.borrow()) {
                    let nvalue = effect(&value, &state, alph.borrow(), &nstate);
                    self.ndp
                        .entry(nstate)
                        .and_modify(|acc| *acc = M::operate(acc, &nvalue))
                        .or_insert(nvalue);
                }
            }
        }
        swap(&mut self.dp, &mut self.ndp);
    }
    pub fn fold_accept(&self) -> M::T {
        let mut acc = M::unit();
        for (state, value) in self.dp.iter() {
            if self.dfa.accept(state) {
                acc = M::operate(&acc, value);
            }
        }
        acc
    }
    pub fn map_fold_accept<U, F, D>(&self, mut f: F, mut map: D) -> D
    where
        F: FnMut(&A::State) -> U,
        D: Container<Key = U, Value = M::T>,
    {
        for (state, value) in self.dp.iter() {
            if self.dfa.accept(state) {
                map.entry(f(state))
                    .and_modify(|acc| *acc = M::operate(acc, value))
                    .or_insert_with(|| value.clone());
            }
        }
        map
    }
    pub fn run<S, I, B>(&mut self, mut sigma: S, len: usize) -> M::T
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<A::Alphabet>,
    {
        for _ in 0..len {
            self.step(&mut sigma);
        }
        self.fold_accept()
    }
    pub fn run_effect<S, I, B, F>(&mut self, mut sigma: S, len: usize, mut effect: F) -> M::T
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<A::Alphabet>,
        F: FnMut(&M::T, &A::State, &A::Alphabet, &A::State) -> M::T,
    {
        for _ in 0..len {
            self.step_effect(&mut sigma, &mut effect);
        }
        self.fold_accept()
    }
}

#[derive(Debug, Clone)]
pub struct IntersectionAutomaton<Tuple>(pub Tuple);

macro_rules! impl_intersection_automaton {
    (@impl $($T:ident)*, $($a:ident)*, $($s:ident)*) => {
        impl<A, $($T),*> Automaton for IntersectionAutomaton<($($T,)*)>
        where
            $($T: Automaton<Alphabet = A>,)*
        {
            type Alphabet = A;
            type State = ($($T::State,)*);
            fn initial(&self) -> Self::State {
                let Self(($($a,)*)) = self;
                ($($a.initial(),)*)
            }
            fn next(&self, ($($s,)*): &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
                let Self(($($a,)*)) = self;
                match ($($a.next($s, alph),)*) {
                    ($(Some($s),)*) => Some(($($s,)*)),
                    _ => None,
                }
            }
            fn accept(&self, ($($s,)*): &Self::State) -> bool {
                let Self(($($a,)*)) = self;
                $($a.accept($s))&&*
            }
        }
    };
    (@inc $($T:ident)*, $($a:ident)*, $($s:ident)*, $U:ident $b:ident $t:ident) => {
        impl_intersection_automaton!(@impl $($T)* $U, $($a)* $b, $($s)* $t);
    };
    (@inc $($T:ident)*, $($a:ident)*, $($s:ident)*, $U:ident $b:ident $t:ident $($tt:tt)*) => {
        impl_intersection_automaton!(@impl $($T)* $U, $($a)* $b, $($s)* $t);
        impl_intersection_automaton!(@inc $($T)* $U, $($a)* $b, $($s)* $t, $($tt)*);
    };
    ($($tt:tt)*) => {
        impl_intersection_automaton!(@inc , , , $($tt)*);
    };
}
impl_intersection_automaton!(T0 t0 s0 T1 t1 s1 T2 t2 s2 T3 t3 s3 T4 t4 s4 T5 t5 s5);

#[derive(Debug, Clone)]
pub struct UnionAutomaton<Tuple>(pub Tuple);

macro_rules! impl_union_automaton {
    (@impl $($T:ident)*, $($a:ident)*, $($s:ident)*) => {
        impl<A, $($T),*> Automaton for UnionAutomaton<($($T,)*)>
        where
            $($T: Automaton<Alphabet = A>,)*
        {
            type Alphabet = A;
            type State = ($($T::State,)*);
            fn initial(&self) -> Self::State {
                let Self(($($a,)*)) = self;
                ($($a.initial(),)*)
            }
            fn next(&self, ($($s,)*): &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
                let Self(($($a,)*)) = self;
                match ($($a.next($s, alph),)*) {
                    ($(Some($s),)*) => Some(($($s,)*)),
                    _ => None,
                }
            }
            fn accept(&self, ($($s,)*): &Self::State) -> bool {
                let Self(($($a,)*)) = self;
                $($a.accept($s))||*
            }
        }
    };
    (@inc $($T:ident)*, $($a:ident)*, $($s:ident)*, $U:ident $b:ident $t:ident) => {
        impl_union_automaton!(@impl $($T)* $U, $($a)* $b, $($s)* $t);
    };
    (@inc $($T:ident)*, $($a:ident)*, $($s:ident)*, $U:ident $b:ident $t:ident $($tt:tt)*) => {
        impl_union_automaton!(@impl $($T)* $U, $($a)* $b, $($s)* $t);
        impl_union_automaton!(@inc $($T)* $U, $($a)* $b, $($s)* $t, $($tt)*);
    };
    ($($tt:tt)*) => {
        impl_union_automaton!(@inc , , , $($tt)*);
    };
}
impl_union_automaton!(T0 t0 s0 T1 t1 s1 T2 t2 s2 T3 t3 s3 T4 t4 s4 T5 t5 s5);

#[derive(Debug, Clone)]
pub struct ProductAutomaton<Tuple>(pub Tuple);

macro_rules! impl_product_automaton {
    (@impl $($T:ident)*, $($a:ident)*, $($s:ident)*, $($c:ident)*) => {
        impl<$($T),*> Automaton for ProductAutomaton<($($T,)*)>
        where
            $($T: Automaton,)*
        {
            type Alphabet = ($($T::Alphabet,)*);
            type State = ($($T::State,)*);
            fn initial(&self) -> Self::State {
                let Self(($($a,)*)) = self;
                ($($a.initial(),)*)
            }
            fn next(&self, ($($s,)*): &Self::State, ($($c,)*): &Self::Alphabet) -> Option<Self::State> {
                let Self(($($a,)*)) = self;
                match ($($a.next($s, $c),)*) {
                    ($(Some($s),)*) => Some(($($s,)*)),
                    _ => None,
                }
            }
            fn accept(&self, ($($s,)*): &Self::State) -> bool {
                let Self(($($a,)*)) = self;
                $($a.accept($s))&&*
            }
        }
    };
    (@inc $($T:ident)*, $($a:ident)*, $($s:ident)*, $($c:ident)*, $U:ident $b:ident $t:ident $d:ident) => {
        impl_product_automaton!(@impl $($T)* $U, $($a)* $b, $($s)* $t, $($c)* $d);
    };
    (@inc $($T:ident)*, $($a:ident)*, $($s:ident)*, $($c:ident)*, $U:ident $b:ident $t:ident $d:ident $($tt:tt)*) => {
        impl_product_automaton!(@impl $($T)* $U, $($a)* $b, $($s)* $t, $($c)* $d);
        impl_product_automaton!(@inc $($T)* $U, $($a)* $b, $($s)* $t, $($c)* $d, $($tt)*);
    };
    ($($tt:tt)*) => {
        impl_product_automaton!(@inc , , , , $($tt)*);
    };
}
impl_product_automaton!(T0 t0 s0 c0 T1 t1 s1 c1 T2 t2 s2 c2 T3 t3 s3 c3 T4 t4 s4 c4 T5 t5 s5 c5);

#[derive(Debug, Clone)]
pub struct FunctionalAutomaton<A, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &A) -> Option<S>,
    H: Fn(&S) -> bool,
{
    fn_initial: F,
    fn_next: G,
    fn_accept: H,
    _marker: PhantomData<fn() -> (A, S)>,
}
impl<A, S, F, G, H> FunctionalAutomaton<A, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &A) -> Option<S>,
    H: Fn(&S) -> bool,
{
    pub fn new(fn_initial: F, fn_next: G, fn_accept: H) -> Self {
        Self {
            fn_initial,
            fn_next,
            fn_accept,
            _marker: PhantomData,
        }
    }
}
impl<A, S, F, G, H> Automaton for FunctionalAutomaton<A, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &A) -> Option<S>,
    H: Fn(&S) -> bool,
{
    type Alphabet = A;
    type State = S;
    fn initial(&self) -> Self::State {
        (self.fn_initial)()
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        (self.fn_next)(state, alph)
    }
    fn accept(&self, state: &Self::State) -> bool {
        (self.fn_accept)(state)
    }
}

pub struct FilterAutomaton<A, G, H>
where
    A: Automaton,
    G: Fn(&A::State, &A::Alphabet) -> bool,
    H: Fn(&A::State) -> bool,
{
    dfa: A,
    fn_next: G,
    fn_accept: H,
    _marker: PhantomData<fn() -> A>,
}
impl<A, G, H> FilterAutomaton<A, G, H>
where
    A: Automaton,
    G: Fn(&A::State, &A::Alphabet) -> bool,
    H: Fn(&A::State) -> bool,
{
    pub fn new(dfa: A, fn_next: G, fn_accept: H) -> Self {
        Self {
            dfa,
            fn_next,
            fn_accept,
            _marker: PhantomData,
        }
    }
}
impl<A, G, H> Automaton for FilterAutomaton<A, G, H>
where
    A: Automaton,
    G: Fn(&A::State, &A::Alphabet) -> bool,
    H: Fn(&A::State) -> bool,
{
    type Alphabet = A::Alphabet;
    type State = A::State;
    fn initial(&self) -> Self::State {
        self.dfa.initial()
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        self.dfa
            .next(state, alph)
            .and_then(|s| (self.fn_next)(state, alph).then_some(s))
    }
    fn accept(&self, state: &Self::State) -> bool {
        (self.fn_accept)(state)
    }
}

#[derive(Debug, Clone)]
pub struct MappingAutomaton<A, S, F, G, H>
where
    A: Automaton,
    F: Fn() -> S,
    G: Fn(&(A::State, S), &A::Alphabet) -> Option<S>,
    H: Fn(&(A::State, S)) -> bool,
{
    dfa: A,
    fn_initial: F,
    fn_next: G,
    fn_accept: H,
    _marker: PhantomData<fn() -> S>,
}
impl<A, S, F, G, H> MappingAutomaton<A, S, F, G, H>
where
    A: Automaton,
    F: Fn() -> S,
    G: Fn(&(A::State, S), &A::Alphabet) -> Option<S>,
    H: Fn(&(A::State, S)) -> bool,
{
    pub fn new(dfa: A, fn_initial: F, fn_next: G, fn_accept: H) -> Self {
        Self {
            dfa,
            fn_initial,
            fn_next,
            fn_accept,
            _marker: PhantomData,
        }
    }
}
impl<A, S, F, G, H> Automaton for MappingAutomaton<A, S, F, G, H>
where
    A: Automaton,
    F: Fn() -> S,
    G: Fn(&(A::State, S), &A::Alphabet) -> Option<S>,
    H: Fn(&(A::State, S)) -> bool,
{
    type Alphabet = A::Alphabet;
    type State = (A::State, S);
    fn initial(&self) -> Self::State {
        (self.dfa.initial(), (self.fn_initial)())
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        self.dfa
            .next(&state.0, alph)
            .and_then(|s| (self.fn_next)(state, alph).map(|ss| (s, ss)))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.dfa.accept(&state.0) && (self.fn_accept)(state)
    }
}

#[derive(Debug, Clone)]
pub struct AlphabetMappingAutomaton<A, S, B, F, G, H>
where
    A: Automaton,
    F: Fn() -> S,
    G: Fn(&S, &B) -> Option<(S, A::Alphabet)>,
    H: Fn(&S) -> bool,
{
    dfa: A,
    fn_initial: F,
    fn_next: G,
    fn_accept: H,
    _marker: PhantomData<fn() -> (S, B)>,
}
impl<A, S, B, F, G, H> AlphabetMappingAutomaton<A, S, B, F, G, H>
where
    A: Automaton,
    F: Fn() -> S,
    G: Fn(&S, &B) -> Option<(S, A::Alphabet)>,
    H: Fn(&S) -> bool,
{
    pub fn new(dfa: A, fn_initial: F, fn_next: G, fn_accept: H) -> Self {
        Self {
            dfa,
            fn_initial,
            fn_next,
            fn_accept,
            _marker: PhantomData,
        }
    }
}
impl<A, S, B, F, G, H> Automaton for AlphabetMappingAutomaton<A, S, B, F, G, H>
where
    A: Automaton,
    F: Fn() -> S,
    G: Fn(&S, &B) -> Option<(S, A::Alphabet)>,
    H: Fn(&S) -> bool,
{
    type Alphabet = B;
    type State = (A::State, S);
    fn initial(&self) -> Self::State {
        (self.dfa.initial(), (self.fn_initial)())
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        (self.fn_next)(&state.1, alph)
            .and_then(|(s1, a)| self.dfa.next(&state.0, &a).map(|s0| (s0, s1)))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.dfa.accept(&state.0) && (self.fn_accept)(&state.1)
    }
}

#[derive(Debug, Clone)]
/// DFA to accept Less/Greater than (or equal to) the sequence
pub struct LexicographicalSequenceAutomaton<'a, T> {
    sequence: &'a [T],
    ordering: Ordering,
    equal: bool,
}
impl<'a, T> LexicographicalSequenceAutomaton<'a, T> {
    pub fn less_than(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Less,
            equal: false,
        }
    }
    pub fn less_than_or_equal(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Less,
            equal: true,
        }
    }
    pub fn greater_than(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Greater,
            equal: false,
        }
    }
    pub fn greater_than_or_equal(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Greater,
            equal: true,
        }
    }
}
impl<T> Automaton for LexicographicalSequenceAutomaton<'_, T>
where
    T: Ord,
{
    type Alphabet = T;
    /// (next position of sequence, is equal)
    type State = (usize, bool);
    fn initial(&self) -> Self::State {
        (0, true)
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        self.sequence
            .get(state.0)
            .and_then(|c| match (state.1, c.cmp(alph)) {
                (true, Ordering::Equal) => Some((state.0 + 1, true)),
                (true, ord) if ord == self.ordering => None,
                _ => Some((state.0 + 1, false)),
            })
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.equal || !state.1
    }
}

#[derive(Debug, Clone)]
/// DFA to accept Less/Greater than (or equal to) the reversed sequence
pub struct RevLexicographicalSequenceAutomaton<'a, T> {
    sequence: &'a [T],
    ordering: Ordering,
    equal: bool,
}
impl<'a, T> RevLexicographicalSequenceAutomaton<'a, T> {
    pub fn less_than(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Less,
            equal: false,
        }
    }
    pub fn less_than_or_equal(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Less,
            equal: true,
        }
    }
    pub fn greater_than(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Greater,
            equal: false,
        }
    }
    pub fn greater_than_or_equal(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            ordering: Ordering::Greater,
            equal: true,
        }
    }
}
impl<T> Automaton for RevLexicographicalSequenceAutomaton<'_, T>
where
    T: Ord,
{
    type Alphabet = T;
    /// (next position of sequence, is equal)
    type State = (usize, Ordering);
    fn initial(&self) -> Self::State {
        (self.sequence.len(), Ordering::Equal)
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        let index = state.0.wrapping_add(!0);
        self.sequence
            .get(index)
            .map(|c| (index, alph.cmp(c).then(state.1)))
    }
    fn accept(&self, state: &Self::State) -> bool {
        state.1 == self.ordering || self.equal && matches!(state.1, Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
/// DFA to accept Less/Greater than (or equal to) in lexicographical order
pub struct LexicographicalAutomaton<T> {
    ordering: Ordering,
    equal: bool,
    _marker: PhantomData<fn() -> T>,
}
impl<T> LexicographicalAutomaton<T> {
    pub fn less_than() -> Self {
        Self {
            ordering: Ordering::Less,
            equal: false,
            _marker: PhantomData,
        }
    }
    pub fn less_than_or_equal() -> Self {
        Self {
            ordering: Ordering::Less,
            equal: true,
            _marker: PhantomData,
        }
    }
    pub fn greater_than() -> Self {
        Self {
            ordering: Ordering::Greater,
            equal: false,
            _marker: PhantomData,
        }
    }
    pub fn greater_than_or_equal() -> Self {
        Self {
            ordering: Ordering::Greater,
            equal: true,
            _marker: PhantomData,
        }
    }
}
impl<T> Automaton for LexicographicalAutomaton<T>
where
    T: Ord,
{
    type Alphabet = (T, T);
    /// is equal
    type State = bool;
    fn initial(&self) -> Self::State {
        true
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        match (state, alph.1.cmp(&alph.0)) {
            (true, Ordering::Equal) => Some(true),
            (true, ord) if ord == self.ordering => None,
            _ => Some(false),
        }
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.equal || !state
    }
}

#[derive(Debug, Clone)]
/// DFA to accept Less/Greater than (or equal to) in reversed lexicographical order
pub struct RevLexicographicalAutomaton<T> {
    ordering: Ordering,
    equal: bool,
    _marker: PhantomData<fn() -> T>,
}
impl<T> RevLexicographicalAutomaton<T> {
    pub fn less_than() -> Self {
        Self {
            ordering: Ordering::Less,
            equal: false,
            _marker: PhantomData,
        }
    }
    pub fn less_than_or_equal() -> Self {
        Self {
            ordering: Ordering::Less,
            equal: true,
            _marker: PhantomData,
        }
    }
    pub fn greater_than() -> Self {
        Self {
            ordering: Ordering::Greater,
            equal: false,
            _marker: PhantomData,
        }
    }
    pub fn greater_than_or_equal() -> Self {
        Self {
            ordering: Ordering::Greater,
            equal: true,
            _marker: PhantomData,
        }
    }
}
impl<T> Automaton for RevLexicographicalAutomaton<T>
where
    T: Ord,
{
    type Alphabet = (T, T);
    /// is equal
    type State = Ordering;
    fn initial(&self) -> Self::State {
        Ordering::Equal
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        Some(alph.0.cmp(&alph.1).then(*state))
    }
    fn accept(&self, state: &Self::State) -> bool {
        *state == self.ordering || self.equal && matches!(state, Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
pub struct MonoidalAutomaton<M>(PhantomData<fn() -> M>)
where
    M: Monoid;
impl<M> MonoidalAutomaton<M>
where
    M: Monoid,
{
    pub fn new() -> Self {
        Default::default()
    }
}
impl<M> Default for MonoidalAutomaton<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<M> Automaton for MonoidalAutomaton<M>
where
    M: Monoid,
{
    type Alphabet = M::T;
    type State = M::T;
    fn initial(&self) -> Self::State {
        M::unit()
    }
    fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
        Some(M::operate(state, alph))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct AlwaysAcceptingAutomaton<A>(PhantomData<fn() -> A>);
impl<A> AlwaysAcceptingAutomaton<A> {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<A> Default for AlwaysAcceptingAutomaton<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<A> Automaton for AlwaysAcceptingAutomaton<A> {
    type Alphabet = A;
    type State = ();
    fn initial(&self) -> Self::State {}
    fn next(&self, _state: &Self::State, _alph: &Self::Alphabet) -> Option<Self::State> {
        Some(())
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}

pub trait ToDigitSequence: Sized {
    fn to_digit_sequence(&self) -> Vec<Self>;
    fn to_digit_sequence_radix(&self, radix: Self) -> Vec<Self>;
    fn to_digit_sequence_len(&self, len: usize) -> Vec<Self>;
    fn to_digit_sequence_radix_len(&self, radix: Self, len: usize) -> Vec<Self>;
}

macro_rules! impl_to_digit_sequence {
    ($($t:ty)*) => {
        $(impl ToDigitSequence for $t {
            fn to_digit_sequence(&self) -> Vec<$t> {
                self.to_digit_sequence_radix(10)
            }
            fn to_digit_sequence_radix(&self, radix: Self) -> Vec<$t> {
                let mut x = *self;
                let mut res: Vec<$t> = vec![];
                while x > 0 {
                    res.push(x % radix);
                    x /= radix;
                }
                res.reverse();
                res
            }
            fn to_digit_sequence_len(&self, len: usize) -> Vec<$t> {
                self.to_digit_sequence_radix_len(10, len)
            }
            fn to_digit_sequence_radix_len(&self, radix: Self, len: usize) -> Vec<$t> {
                let mut x = *self;
                let mut res: Vec<$t> = vec![0; len];
                for r in res.iter_mut().rev() {
                    if x == 0 {
                        break;
                    }
                    *r = x % radix;
                    x /= radix;
                }
                res
            }
        })*
    };
}
impl_to_digit_sequence!(u8 u16 u32 u64 u128 usize);

/// build automaton
///
/// - `automaton!(A)`
/// - `<= seq`, `seq >=`: [`LexicographicalSequenceAutomaton::less_than_or_equal(&seq)`](`LexicographicalSequenceAutomaton::less_than_or_equal`)
/// - `>= seq`, `seq <=`: [`LexicographicalSequenceAutomaton::greater_than_or_equal(&seq)`](`LexicographicalSequenceAutomaton::greater_than_or_equal`)
/// - `< seq`, `seq >`: [`LexicographicalSequenceAutomaton::less_than(&seq)`](`LexicographicalSequenceAutomaton::less_than`)
/// - `> seq`, `seq <`: [`LexicographicalSequenceAutomaton::greater_than(&seq)`](`LexicographicalSequenceAutomaton::greater_than`)
/// - `!<= seq`, `seq !>=`: [`RevLexicographicalSequenceAutomaton::less_than_or_equal(&seq)`](`RevLexicographicalSequenceAutomaton::less_than_or_equal`)
/// - `!>= seq`, `seq !<=`: [`RevLexicographicalSequenceAutomaton::greater_than_or_equal(&seq)`](`RevLexicographicalSequenceAutomaton::greater_than_or_equal`)
/// - `!< seq`, `seq !>`: [`RevLexicographicalSequenceAutomaton::less_than(&seq)`](`RevLexicographicalSequenceAutomaton::less_than`)
/// - `!> seq`, `seq !<`: [`RevLexicographicalSequenceAutomaton::greater_than(&seq)`](`RevLexicographicalSequenceAutomaton::greater_than`)
/// - `<=`: [`LexicographicalAutomaton::less_than_or_equal()`](`LexicographicalAutomaton::less_than_or_equal`)
/// - `>=`: [`LexicographicalAutomaton::greater_than_or_equal()`](`LexicographicalAutomaton::greater_than_or_equal`)
/// - `<`: [`LexicographicalAutomaton::less_than()`](`LexicographicalAutomaton::less_than`)
/// - `>`: [`LexicographicalAutomaton::greater_than()`](`LexicographicalAutomaton::greater_than`)
/// - `!<=`: [`RevLexicographicalAutomaton::less_than_or_equal()`](`RevLexicographicalAutomaton::less_than_or_equal`)
/// - `!>=`: [`RevLexicographicalAutomaton::greater_than_or_equal()`](`RevLexicographicalAutomaton::greater_than_or_equal`)
/// - `!<`: [`RevLexicographicalAutomaton::less_than()`](`RevLexicographicalAutomaton::less_than`)
/// - `!>`: [`RevLexicographicalAutomaton::greater_than()`](`RevLexicographicalAutomaton::greater_than`)
/// - `=> f g h`: [`FunctionalAutomaton::new(f, g, h)`](`FunctionalAutomaton`)
/// - `=> (A) f g h`: [`MappingAutomaton::new(A, f, g, h)`](`MappingAutomaton`)
/// - `=> f g h (A)`: [`AlphabetMappingAutomaton::new(A, f, g, h)`](`AlphabetMappingAutomaton`)
/// - `@`: [`AlwaysAcceptingAutomaton::new()`](`AlwaysAcceptingAutomaton`)
/// - `A * B`: [`ProductAutomaton((A, B))`](`ProductAutomaton`)
/// - `A & B`: [`IntersectionAutomaton((A, B))`](`IntersectionAutomaton`)
/// - `A | B`: [`UnionAutomaton((A, B))`](`UnionAutomaton`)
#[macro_export]
macro_rules! automaton {
    (@inner ($($t:tt)*))                                     => { $crate::automaton!(@inner $($t)*) };
    (@inner <= $e:expr)                                      => { LexicographicalSequenceAutomaton::less_than_or_equal(&$e) };
    (@inner >= $e:expr)                                      => { LexicographicalSequenceAutomaton::greater_than_or_equal(&$e) };
    (@inner < $e:expr)                                       => { LexicographicalSequenceAutomaton::less_than(&$e) };
    (@inner > $e:expr)                                       => { LexicographicalSequenceAutomaton::greater_than(&$e) };
    (@inner !<= $e:expr)                                     => { RevLexicographicalSequenceAutomaton::less_than_or_equal(&$e) };
    (@inner !>= $e:expr)                                     => { RevLexicographicalSequenceAutomaton::greater_than_or_equal(&$e) };
    (@inner !< $e:expr)                                      => { RevLexicographicalSequenceAutomaton::less_than(&$e) };
    (@inner !> $e:expr)                                      => { RevLexicographicalSequenceAutomaton::greater_than(&$e) };
    (@inner $e:ident >=)                                     => { LexicographicalSequenceAutomaton::less_than_or_equal(&$e) };
    (@inner $e:ident <=)                                     => { LexicographicalSequenceAutomaton::greater_than_or_equal(&$e) };
    (@inner $e:ident >)                                      => { LexicographicalSequenceAutomaton::less_than(&$e) };
    (@inner $e:ident <)                                      => { LexicographicalSequenceAutomaton::greater_than(&$e) };
    (@inner $e:ident !>=)                                    => { RevLexicographicalSequenceAutomaton::less_than_or_equal(&$e) };
    (@inner $e:ident !<=)                                    => { RevLexicographicalSequenceAutomaton::greater_than_or_equal(&$e) };
    (@inner $e:ident !>)                                     => { RevLexicographicalSequenceAutomaton::less_than(&$e) };
    (@inner $e:ident !<)                                     => { RevLexicographicalSequenceAutomaton::greater_than(&$e) };
    (@inner <=)                                              => { LexicographicalAutomaton::less_than_or_equal() };
    (@inner >=)                                              => { LexicographicalAutomaton::greater_than_or_equal() };
    (@inner <)                                               => { LexicographicalAutomaton::less_than() };
    (@inner >)                                               => { LexicographicalAutomaton::greater_than() };
    (@inner !<=)                                             => { RevLexicographicalAutomaton::less_than_or_equal() };
    (@inner !>=)                                             => { RevLexicographicalAutomaton::greater_than_or_equal() };
    (@inner !<)                                              => { RevLexicographicalAutomaton::less_than() };
    (@inner !>)                                              => { RevLexicographicalAutomaton::greater_than() };
    (@inner => ($($t:tt)*) $f:expr, $g:expr, $h:expr $(,)?)  => { MappingAutomaton::new($crate::automaton!(@inner $($t)*), $f, $g, $h) };
    (@inner => $f:expr, $g:expr, $h:expr, ($($t:tt)*) $(,)?) => { AlphabetMappingAutomaton::new($crate::automaton!(@inner $($t)*), $f, $g, $h) };
    (@inner => $f:expr, $g:expr, $h:expr $(,)?)              => { FunctionalAutomaton::new($f, $g, $h) };
    (@inner @<$t:ty>)                                        => { AlwaysAcceptingAutomaton::<$t>::new() };
    (@inner @)                                               => { AlwaysAcceptingAutomaton::new() };
    (@inner $($t:tt)*)                                       => { $crate::automaton!(@union [] [] $($t)*) };
    (@union [$([$($a:tt)*])*])                               => { UnionAutomaton(($($crate::automaton!(@inner $($a)*),)*)) };
    (@union [] [$($b:tt)*])                                  => { $crate::automaton!(@inter [] [] $($b)*) };
    (@union [$($a:tt)*] [$($b:tt)*])                         => { $crate::automaton!(@union [$($a)* [$($b)*]]) };
    (@union [$($a:tt)*] [$($b:tt)*] | $($t:tt)*)             => { $crate::automaton!(@union [$($a)* [$($b)*]] [] $($t)*) };
    (@union [$($a:tt)*] [$($b:tt)*] $op:tt $($t:tt)*)        => { $crate::automaton!(@union [$($a)*] [$($b)* $op] $($t)*) };
    (@inter [$([$($a:tt)*])*])                               => { IntersectionAutomaton(($($crate::automaton!(@inner $($a)*),)*)) };
    (@inter [] [$($b:tt)*])                                  => { $crate::automaton!(@prod [] [] $($b)*) };
    (@inter [$($a:tt)*] [$($b:tt)*])                         => { $crate::automaton!(@inter [$($a)* [$($b)*]]) };
    (@inter [$($a:tt)*] [$($b:tt)*] & $($t:tt)*)             => { $crate::automaton!(@inter [$($a)* [$($b)*]] [] $($t)*) };
    (@inter [$($a:tt)*] [$($b:tt)*] $op:tt $($t:tt)*)        => { $crate::automaton!(@inter [$($a)*] [$($b)* $op] $($t)*) };
    (@prod [$([$($a:tt)*])*])                                => { ProductAutomaton(($($crate::automaton!(@inner $($a)*),)*)) };
    (@prod [] [$($b:tt)*])                                   => { $($b)* };
    (@prod [$($a:tt)*] [$($b:tt)*])                          => { $crate::automaton!(@prod [$($a)* [$($b)*]]) };
    (@prod [$($a:tt)*] [$($b:tt)*] * $($t:tt)*)              => { $crate::automaton!(@prod [$($a)* [$($b)*]] [] $($t)*) };
    (@prod [$($a:tt)*] [$($b:tt)*] $op:tt $($t:tt)*)         => { $crate::automaton!(@prod [$($a)*] [$($b)* $op] $($t)*) };
    (@$tag:ident $($t:tt)*)                                  => { compile_error!(stringify!($tag, $($t)*)) };
    ($($t:tt)*)                                              => { $crate::automaton!(@inner $($t)*) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{algebra::AdditiveOperation, automaton, tools::Xorshift};

    #[test]
    fn test_lexicographical_sequence() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r) in rng.gen_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(
                n + 1,
                automaton!(<= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                n,
                automaton!(< nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n,
                automaton!(>= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                automaton!(> nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
        }
    }

    #[test]
    fn test_revlexicographical_sequence() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r) in rng.gen_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(
                n + 1,
                automaton!(!<= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                n,
                automaton!(!< nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n,
                automaton!(!>= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                automaton!(!> nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
        }
    }

    #[test]
    fn test_lexicographical() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r) in rng.gen_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(
                n + 1,
                automaton!((=>
                    || 0usize,
                    |&i, &a| Some((i + 1, (a, nd[i]))),
                    |&_| true,
                    (<=)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
            assert_eq!(
                n,
                automaton!((=>
                    || 0usize,
                    |&i, &a| Some((i + 1, (a, nd[i]))),
                    |&_| true,
                    (<)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n,
                automaton!((=>
                    || 0usize,
                    |&i, &a| Some((i + 1, (a, nd[i]))),
                    |&_| true,
                    (>=)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                automaton!((=>
                    || 0usize,
                    |&i, &a| Some((i + 1, (a, nd[i]))),
                    |&_| true,
                    (>)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
        }
    }

    #[test]
    fn test_revlexicographical() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r) in rng.gen_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(
                n + 1,
                automaton!((=>
                    || nd.len(),
                    |&i, &a| Some((i.wrapping_sub(1), (a, nd[i - 1]))),
                    |&_| true,
                    (!<=)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
            assert_eq!(
                n,
                automaton!((=>
                    || nd.len(),
                    |&i, &a| Some((i.wrapping_sub(1), (a, nd[i - 1]))),
                    |&_| true,
                    (!<)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n,
                automaton!((=>
                    || nd.len(),
                    |&i, &a| Some((i.wrapping_sub(1), (a, nd[i - 1]))),
                    |&_| true,
                    (!>=)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                automaton!((=>
                    || nd.len(),
                    |&i, &a| Some((i.wrapping_sub(1), (a, nd[i - 1]))),
                    |&_| true,
                    (!>)
                ))
                .dp::<A>(1)
                .with_hashmap()
                .run(|| 0..r, nd.len())
            );
        }
    }

    struct C(usize, usize);
    impl Automaton for C {
        type Alphabet = usize;
        type State = usize;
        fn initial(&self) -> Self::State {
            0
        }
        fn next(&self, state: &Self::State, alph: &Self::Alphabet) -> Option<Self::State> {
            Some((state * self.1 + alph) % self.0)
        }
        fn accept(&self, state: &Self::State) -> bool {
            *state == 0
        }
    }

    #[test]
    fn test_prim() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r, c) in rng.gen_iter((0..10usize.pow(18), 2..=10, 2..200)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            let dfa = automaton!((< nd) & (C(c, r)));
            assert_eq!(
                n.div_ceil(c),
                dfa.dp::<A>(1).with_hashmap().run(|| 0..r, nd.len())
            );

            let dfa =
                automaton!((< nd) & (=> || 0usize, |s, a| Some((s * r + a) % c), |s| *s == 0));
            assert_eq!(
                n.div_ceil(c),
                dfa.dp::<A>(1).with_hashmap().run(|| 0..r, nd.len())
            );
            assert_eq!(
                n.div_ceil(c),
                dfa.dp::<A>(1)
                    .with_vecmap(|s| s.1 * 2 + (s.0).1 as usize)
                    .run(|| 0..r, nd.len())
            );
        }
    }
}
