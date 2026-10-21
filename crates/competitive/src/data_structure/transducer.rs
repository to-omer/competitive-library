use super::{AdditiveOperation, Container, ContainerEntry, Monoid, One, Zero};
use std::{
    borrow::Borrow,
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    hash::{Hash, RandomState},
    iter::Peekable,
    marker::PhantomData,
    mem::swap,
    ops::Add,
};

type Marker<T> = PhantomData<fn() -> T>;

pub trait Transducer {
    type Input;
    type Output;
    type State;
    fn start(&self) -> Self::State;
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)>;
    fn accept(&self, state: &Self::State) -> bool;
    /// Advances after processing the current layer.
    fn stepout(&mut self) {}
    fn dp<M>(self, init: M::T) -> TransducerDp<M, Self, HashMap<Self::State, M::T>>
    where
        Self: Sized,
        Self::State: Eq + Hash,
        M: Monoid,
    {
        let hash_builder = RandomState::new();
        self.dp_with(init, || HashMap::with_hasher(hash_builder.clone()))
    }
    fn dp_with<M, C>(self, init: M::T, create: impl FnMut() -> C) -> TransducerDp<M, Self, C>
    where
        Self: Sized,
        M: Monoid,
        C: Container<Key = Self::State, Value = M::T>,
    {
        TransducerDp::new(self, init, create)
    }
    fn counting<V>(self) -> TransducerDp<AdditiveOperation<V>, Self, HashMap<Self::State, V>>
    where
        Self: Sized,
        Self::State: Eq + Hash,
        V: Clone + Zero + One + Add<Output = V>,
    {
        self.dp(V::one())
    }
    fn counting_with<V, C>(
        self,
        create: impl FnMut() -> C,
    ) -> TransducerDp<AdditiveOperation<V>, Self, C>
    where
        Self: Sized,
        V: Clone + Zero + One + Add<Output = V>,
        C: Container<Key = Self::State, Value = V>,
    {
        self.dp_with(V::one(), create)
    }

    fn intersection<U>(self, other: U) -> IntersectionTransducer<(Self, U)>
    where
        Self: Sized,
        U: Transducer<Input = Self::Input>,
    {
        IntersectionTransducer((self, other))
    }
    fn product<U>(self, other: U) -> ProductTransducer<(Self, U)>
    where
        Self: Sized,
        U: Transducer,
    {
        ProductTransducer((self, other))
    }
    fn chain<U>(self, other: U) -> ChainTransducer<(Self, U)>
    where
        Self: Sized,
        U: Transducer<Input = Self::Output>,
    {
        ChainTransducer((self, other))
    }
    /// Appends a cloned input to each output as `(output, input)`, preserving the state.
    fn with_input(self) -> WithInputTransducer<Self>
    where
        Self: Sized,
    {
        WithInputTransducer::new(self)
    }
    fn map<U, F>(self, f: F) -> MapTransducer<Self, U, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        MapTransducer::new(self, f)
    }
    fn try_map<U, F>(self, f: F) -> TryMapTransducer<Self, U, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Option<U>,
    {
        TryMapTransducer::new(self, f)
    }
    /// Converts each input without adding a state component.
    fn map_input<I, F>(self, f: F) -> MapInputTransducer<Self, I, F>
    where
        Self: Sized,
        F: Fn(&I) -> Self::Input,
    {
        MapInputTransducer::new(self, f)
    }
    fn retain<F>(self, pred: F) -> RetainTransducer<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> bool,
    {
        RetainTransducer::new(self, pred)
    }
    fn with_fold<A, F>(self, init: A, f: F) -> FoldTransducer<Self, A, F>
    where
        Self: Sized,
        F: Fn(&A, &Self::Output) -> A,
    {
        FoldTransducer::new(self, init, f)
    }
    fn with_try_fold<A, F>(self, init: A, f: F) -> TryFoldTransducer<Self, A, F>
    where
        Self: Sized,
        F: Fn(&A, &Self::Output) -> Option<A>,
    {
        TryFoldTransducer::new(self, init, f)
    }
    fn accepting<F>(self, pred: F) -> AcceptTransducer<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::State) -> bool,
    {
        AcceptTransducer::new(self, pred)
    }
}

/// `start`, `relation` and `accept` are independent of the layer and call count.
pub trait StationaryTransducer: Transducer {}

impl<T> StationaryTransducer for &T where T: StationaryTransducer + ?Sized {}

impl<T> Transducer for &T
where
    T: StationaryTransducer + ?Sized,
{
    type Input = T::Input;
    type Output = T::Output;
    type State = T::State;

    fn start(&self) -> Self::State {
        T::start(self)
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        T::relation(self, state, input)
    }
    fn accept(&self, state: &Self::State) -> bool {
        T::accept(self, state)
    }
}

/// Aggregation follows the container's iteration order.
pub struct TransducerDp<M, T, C> {
    fst: T,
    pub dp: C,
    ndp: C,
    _marker: PhantomData<fn() -> M>,
}

impl<M, T, C> Clone for TransducerDp<M, T, C>
where
    T: Clone,
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            fst: self.fst.clone(),
            dp: self.dp.clone(),
            ndp: self.ndp.clone(),
            _marker: PhantomData,
        }
    }
}

impl<M, T, C> Debug for TransducerDp<M, T, C>
where
    T: Debug,
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransducerDp")
            .field("fst", &self.fst)
            .field("dp", &self.dp)
            .field("ndp", &self.ndp)
            .finish()
    }
}

impl<M, T, C> TransducerDp<M, T, C>
where
    M: Monoid,
    T: Transducer,
    C: Container<Key = T::State, Value = M::T>,
{
    /// Calls `create` twice; each call must return an independent empty container.
    #[inline]
    pub fn new(fst: T, init: M::T, mut create: impl FnMut() -> C) -> Self {
        let mut dp = create();
        let ndp = create();
        dp.insert(fst.start(), init);
        Self {
            fst,
            dp,
            ndp,
            _marker: PhantomData,
        }
    }
    /// Calls `sigma` once per reachable state to enumerate its candidate inputs.
    pub fn step<S, I, B>(&mut self, mut sigma: S)
    where
        S: FnMut(&T::State) -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
    {
        self.dp.drain().for_each(|(state, value)| {
            for input in sigma(&state) {
                if let Some((nstate, _)) = self.fst.relation(&state, input.borrow()) {
                    self.ndp
                        .entry(nstate)
                        .and_modify(|acc| M::operate_assign(acc, &value))
                        .or_insert_with(|| value.clone());
                }
            }
        });
        swap(&mut self.dp, &mut self.ndp);
        self.fst.stepout();
    }
    /// `effect(value, output)` must preserve `M::operate` for each output.
    pub fn step_effect<S, I, B, F>(&mut self, mut sigma: S, mut effect: F)
    where
        S: FnMut(&T::State) -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
        F: FnMut(&M::T, T::Output) -> M::T,
    {
        self.dp.drain().for_each(|(state, value)| {
            for input in sigma(&state) {
                if let Some((nstate, output)) = self.fst.relation(&state, input.borrow()) {
                    let nvalue = effect(&value, output);
                    self.ndp
                        .entry(nstate)
                        .and_modify(|acc| M::operate_assign(acc, &nvalue))
                        .or_insert(nvalue);
                }
            }
        });
        swap(&mut self.dp, &mut self.ndp);
        self.fst.stepout();
    }
    pub fn fold_accept(&self) -> M::T {
        let mut acc = M::unit();
        for (_, value) in self.iter_accept() {
            M::operate_assign(&mut acc, value);
        }
        acc
    }
    pub fn iter_accept(&self) -> impl Iterator<Item = (&T::State, &M::T)> {
        self.dp.iter().filter(|(state, _)| self.fst.accept(state))
    }
    pub fn map_fold_accept<U, F, D>(&self, mut f: F, mut map: D) -> D
    where
        F: FnMut(&T::State) -> U,
        D: Container<Key = U, Value = M::T>,
    {
        for (state, value) in self.iter_accept() {
            map.entry(f(state))
                .and_modify(|acc| M::operate_assign(acc, value))
                .or_insert_with(|| value.clone());
        }
        map
    }
    /// Advances `len` further layers and aggregates the accepting states.
    #[inline]
    pub fn run<S, I, B>(&mut self, len: usize, mut sigma: S) -> M::T
    where
        S: FnMut(&T::State) -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
    {
        for _ in 0..len {
            self.step(&mut sigma);
        }
        self.fold_accept()
    }
    pub fn run_effect<S, I, B, F>(&mut self, len: usize, mut sigma: S, mut effect: F) -> M::T
    where
        S: FnMut(&T::State) -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
        F: FnMut(&M::T, T::Output) -> M::T,
    {
        for _ in 0..len {
            self.step_effect(&mut sigma, &mut effect);
        }
        self.fold_accept()
    }
}

/// Runs each component from left to right, stopping at the first rejected transition.
#[derive(Debug, Clone)]
pub struct IntersectionTransducer<Tuple>(pub Tuple);

macro_rules! impl_intersection_transducer {
    (@impl $($T:ident)*, $($a:ident)*, $($b:ident)*) => {
        impl<A, $($T),*> StationaryTransducer for IntersectionTransducer<($($T,)*)>
        where
            $($T: StationaryTransducer<Input = A>,)*
        {}

        impl<A, $($T),*> Transducer for IntersectionTransducer<($($T,)*)>
        where
            $($T: Transducer<Input = A>,)*
        {
            type Input = A;
            type Output = ($($T::Output,)*);
            type State = ($($T::State,)*);
            fn start(&self) -> Self::State {
                let Self(($($a,)*)) = self;
                ($($a.start(),)*)
            }
            fn relation(&self, state: &Self::State, input: &Self::Input) -> Option<(Self::State, Self::Output)> {
                let Self(($($a,)*)) = self;
                let ($($b,)*) = state;
                $(let ($a, $b) = $a.relation($b, input)?;)*
                Some((($($a,)*), ($($b,)*)))
            }
            fn accept(&self, state: &Self::State) -> bool {
                let Self(($($a,)*)) = self;
                let ($($b,)*) = state;
                $($a.accept($b))&&*
            }
            fn stepout(&mut self) {
                let Self(($($a,)*)) = self;
                $($a.stepout();)*
            }
        }
    };
    (@inc $($T:ident)*, $($a:ident)*, $($b:ident)*, $TT:ident $aa:ident $bb:ident) => {
        impl_intersection_transducer!(@impl $($T)* $TT, $($a)* $aa, $($b)* $bb);
    };
    (@inc $($T:ident)*, $($a:ident)*, $($b:ident)*, $TT:ident $aa:ident $bb:ident $($tt:tt)*) => {
        impl_intersection_transducer!(@impl $($T)* $TT, $($a)* $aa, $($b)* $bb);
        impl_intersection_transducer!(@inc $($T)* $TT, $($a)* $aa, $($b)* $bb, $($tt)*);
    };
    ($($tt:tt)*) => {
        impl_intersection_transducer!(@inc , , , $($tt)*);
    };
}
impl_intersection_transducer!(
    T0 a0 b0
    T1 a1 b1
    T2 a2 b2
    T3 a3 b3
    T4 a4 b4
    T5 a5 b5
    T6 a6 b6
    T7 a7 b7
    T8 a8 b8
    T9 a9 b9
);

#[derive(Debug, Clone)]
pub struct ProductTransducer<Tuple>(pub Tuple);

macro_rules! impl_product_transducer {
    (@impl $($T:ident)*, $($a:ident)*, $($b:ident)*, $($c:ident)*) => {
        impl<$($T),*> StationaryTransducer for ProductTransducer<($($T,)*)>
        where
            $($T: StationaryTransducer,)*
        {}

        impl<$($T),*> Transducer for ProductTransducer<($($T,)*)>
        where
            $($T: Transducer,)*
        {
            type Input = ($($T::Input,)*);
            type Output = ($($T::Output,)*);
            type State = ($($T::State,)*);
            fn start(&self) -> Self::State {
                let Self(($($a,)*)) = self;
                ($($a.start(),)*)
            }
            fn relation(&self, state: &Self::State, ($($c,)*): &Self::Input) -> Option<(Self::State, Self::Output)> {
                let Self(($($a,)*)) = self;
                let ($($b,)*) = state;
                $(let ($a, $b) = $a.relation($b, $c)?;)*
                Some((($($a,)*), ($($b,)*)))
            }
            fn accept(&self, state: &Self::State) -> bool {
                let Self(($($a,)*)) = self;
                let ($($b,)*) = state;
                $($a.accept($b))&&*
            }
            fn stepout(&mut self) {
                let Self(($($a,)*)) = self;
                $($a.stepout();)*
            }
        }
    };
    (@inc $($T:ident)*, $($a:ident)*, $($b:ident)*, $($c:ident)*, $TT:ident $aa:ident $bb:ident $cc:ident) => {
        impl_product_transducer!(@impl $($T)* $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc);
    };
    (@inc $($T:ident)*, $($a:ident)*, $($b:ident)*, $($c:ident)*, $TT:ident $aa:ident $bb:ident $cc:ident $($tt:tt)*) => {
        impl_product_transducer!(@impl $($T)* $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc);
        impl_product_transducer!(@inc $($T)* $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc, $($tt)*);
    };
    ($($tt:tt)*) => {
        impl_product_transducer!(@inc , , , , $($tt)*);
    };
}
impl_product_transducer!(
    T0 a0 b0 c0
    T1 a1 b1 c1
    T2 a2 b2 c2
    T3 a3 b3 c3
    T4 a4 b4 c4
    T5 a5 b5 c5
    T6 a6 b6 c6
    T7 a7 b7 c7
    T8 a8 b8 c8
    T9 a9 b9 c9
);

#[derive(Debug, Clone)]
pub struct ChainTransducer<Tuple>(pub Tuple);

macro_rules! impl_chain_transducer {
    (@impl $T_head:ident, $($T_tail:ident)*, $($T_init:ident)*, $T_last:ident, $($T:ident)*, $($a:ident)*, $($b:ident)*, $($c:ident)*) => {
        impl<$($T),*> StationaryTransducer for ChainTransducer<($($T,)*)>
        where
            $T_head: StationaryTransducer,
            $($T_tail: StationaryTransducer<Input = $T_init::Output>,)*
        {}

        impl<$($T),*> Transducer for ChainTransducer<($($T,)*)>
        where
            $T_head: Transducer,
            $($T_tail: Transducer<Input = $T_init::Output>,)*
        {
            type Input = $T_head::Input;
            type Output = $T_last::Output;
            type State = ($($T::State,)*);
            fn start(&self) -> Self::State {
                let Self(($($a,)*)) = self;
                ($($a.start(),)*)
            }
            fn relation(&self, state: &Self::State, input: &Self::Input) -> Option<(Self::State, Self::Output)> {
                let Self(($($a,)*)) = self;
                let ($($b,)*) = state;
                $(let ($c, input) = $a.relation($b, &input)?;)*
                Some((($($c,)*), input))
            }
            fn accept(&self, state: &Self::State) -> bool {
                let Self(($($a,)*)) = self;
                let ($($b,)*) = state;
                $($a.accept($b))&&*
            }
            fn stepout(&mut self) {
                let Self(($($a,)*)) = self;
                $($a.stepout();)*
            }
        }
    };
    (@inc $T0:ident $($T:ident)*, $($a:ident)*, $($b:ident)*, $($c:ident)*, $TT:ident $aa:ident $bb:ident $cc:ident) => {
        impl_chain_transducer!(@impl $T0, $($T)* $TT, $T0 $($T)*, $TT, $T0 $($T)* $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc);
    };
    (@inc , $($a:ident)*, $($b:ident)*, $($c:ident)*, $TT:ident $aa:ident $bb:ident $cc:ident $($tt:tt)*) => {
        impl_chain_transducer!(@impl $TT, , , $TT,  $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc);
        impl_chain_transducer!(@inc $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc, $($tt)*);
    };
    (@inc $T0:ident $($T:ident)*, $($a:ident)*, $($b:ident)*, $($c:ident)*, $TT:ident $aa:ident $bb:ident $cc:ident $($tt:tt)*) => {
        impl_chain_transducer!(@impl $T0, $($T)* $TT, $T0 $($T)*, $TT, $T0 $($T)* $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc);
        impl_chain_transducer!(@inc $T0 $($T)* $TT, $($a)* $aa, $($b)* $bb, $($c)* $cc, $($tt)*);
    };
    ($($tt:tt)*) => {
        impl_chain_transducer!(@inc , , , , $($tt)*);
    };
}
impl_chain_transducer!(
    T0 a0 b0 c0
    T1 a1 b1 c1
    T2 a2 b2 c2
    T3 a3 b3 c3
    T4 a4 b4 c4
    T5 a5 b5 c5
    T6 a6 b6 c6
    T7 a7 b7 c7
    T8 a8 b8 c8
    T9 a9 b9 c9
);

#[derive(Debug)]
pub struct FunctionalTransducer<I, O, S, F, G, H> {
    fn_start: F,
    fn_relation: G,
    fn_accept: H,
    _marker: Marker<(I, O, S)>,
}
impl<I, O, S, F, G, H> Clone for FunctionalTransducer<I, O, S, F, G, H>
where
    F: Clone,
    G: Clone,
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            fn_start: self.fn_start.clone(),
            fn_relation: self.fn_relation.clone(),
            fn_accept: self.fn_accept.clone(),
            _marker: PhantomData,
        }
    }
}
impl<I, O, S, F, G, H> FunctionalTransducer<I, O, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &I) -> Option<(S, O)>,
    H: Fn(&S) -> bool,
{
    pub fn new(fn_start: F, fn_relation: G, fn_accept: H) -> Self {
        Self {
            fn_start,
            fn_relation,
            fn_accept,
            _marker: PhantomData,
        }
    }
}
impl<I, O, S, F, G, H> StationaryTransducer for FunctionalTransducer<I, O, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &I) -> Option<(S, O)>,
    H: Fn(&S) -> bool,
{
}

impl<I, O, S, F, G, H> Transducer for FunctionalTransducer<I, O, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &I) -> Option<(S, O)>,
    H: Fn(&S) -> bool,
{
    type Input = I;
    type Output = O;
    type State = S;
    fn start(&self) -> Self::State {
        (self.fn_start)()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        (self.fn_relation)(state, input)
    }
    fn accept(&self, state: &Self::State) -> bool {
        (self.fn_accept)(state)
    }
}

#[derive(Debug)]
pub struct MapTransducer<T, U, F> {
    inner: T,
    f: F,
    _marker: Marker<U>,
}
impl<T, U, F> Clone for MapTransducer<T, U, F>
where
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}
impl<T, U, F> MapTransducer<T, U, F> {
    pub fn new(inner: T, f: F) -> Self
    where
        T: Transducer,
        F: Fn(T::Output) -> U,
    {
        Self {
            inner,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, U, F> StationaryTransducer for MapTransducer<T, U, F>
where
    T: StationaryTransducer,
    F: Fn(T::Output) -> U,
{
}

impl<T, U, F> Transducer for MapTransducer<T, U, F>
where
    T: Transducer,
    F: Fn(T::Output) -> U,
{
    type Input = T::Input;
    type Output = U;
    type State = T::State;
    fn start(&self) -> Self::State {
        self.inner.start()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        let (next_state, output) = self.inner.relation(state, input)?;
        Some((next_state, (self.f)(output)))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug)]
pub struct TryMapTransducer<T, U, F> {
    inner: T,
    f: F,
    _marker: Marker<U>,
}
impl<T, U, F> Clone for TryMapTransducer<T, U, F>
where
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}
impl<T, U, F> TryMapTransducer<T, U, F> {
    pub fn new(inner: T, f: F) -> Self
    where
        T: Transducer,
        F: Fn(T::Output) -> Option<U>,
    {
        Self {
            inner,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, U, F> StationaryTransducer for TryMapTransducer<T, U, F>
where
    T: StationaryTransducer,
    F: Fn(T::Output) -> Option<U>,
{
}

impl<T, U, F> Transducer for TryMapTransducer<T, U, F>
where
    T: Transducer,
    F: Fn(T::Output) -> Option<U>,
{
    type Input = T::Input;
    type Output = U;
    type State = T::State;
    fn start(&self) -> Self::State {
        self.inner.start()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        let (next_state, output) = self.inner.relation(state, input)?;
        (self.f)(output).map(|output| (next_state, output))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug, Clone)]
pub struct WithInputTransducer<T> {
    inner: T,
}
impl<T> WithInputTransducer<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}
impl<T> StationaryTransducer for WithInputTransducer<T>
where
    T: StationaryTransducer,
    T::Input: Clone,
{
}
impl<T> Transducer for WithInputTransducer<T>
where
    T: Transducer,
    T::Input: Clone,
{
    type Input = T::Input;
    type Output = (T::Output, T::Input);
    type State = T::State;
    fn start(&self) -> Self::State {
        self.inner.start()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        let (next_state, output) = self.inner.relation(state, input)?;
        Some((next_state, (output, input.clone())))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug)]
pub struct MapInputTransducer<T, I, F> {
    inner: T,
    f: F,
    _marker: Marker<I>,
}
impl<T, I, F> Clone for MapInputTransducer<T, I, F>
where
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}
impl<T, I, F> MapInputTransducer<T, I, F> {
    pub fn new(inner: T, f: F) -> Self
    where
        T: Transducer,
        F: Fn(&I) -> T::Input,
    {
        Self {
            inner,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, I, F> StationaryTransducer for MapInputTransducer<T, I, F>
where
    T: StationaryTransducer,
    F: Fn(&I) -> T::Input,
{
}

impl<T, I, F> Transducer for MapInputTransducer<T, I, F>
where
    T: Transducer,
    F: Fn(&I) -> T::Input,
{
    type Input = I;
    type Output = T::Output;
    type State = T::State;
    fn start(&self) -> Self::State {
        self.inner.start()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        self.inner.relation(state, &(self.f)(input))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug, Clone)]
pub struct RetainTransducer<T, F> {
    inner: T,
    pred: F,
}
impl<T, F> RetainTransducer<T, F> {
    pub fn new(inner: T, pred: F) -> Self
    where
        T: Transducer,
        F: Fn(&T::Output) -> bool,
    {
        Self { inner, pred }
    }
}
impl<T, F> StationaryTransducer for RetainTransducer<T, F>
where
    T: StationaryTransducer,
    F: Fn(&T::Output) -> bool,
{
}

impl<T, F> Transducer for RetainTransducer<T, F>
where
    T: Transducer,
    F: Fn(&T::Output) -> bool,
{
    type Input = T::Input;
    type Output = T::Output;
    type State = T::State;
    fn start(&self) -> Self::State {
        self.inner.start()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        let (next_state, output) = self.inner.relation(state, input)?;
        (self.pred)(&output).then_some((next_state, output))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug, Clone)]
pub struct FoldTransducer<T, A, F> {
    inner: T,
    init: A,
    f: F,
}
impl<T, A, F> FoldTransducer<T, A, F> {
    pub fn new(inner: T, init: A, f: F) -> Self
    where
        T: Transducer,
        F: Fn(&A, &T::Output) -> A,
    {
        Self { inner, init, f }
    }
}
impl<T, A, F> StationaryTransducer for FoldTransducer<T, A, F>
where
    T: StationaryTransducer,
    A: Clone,
    F: Fn(&A, &T::Output) -> A,
{
}

impl<T, A, F> Transducer for FoldTransducer<T, A, F>
where
    T: Transducer,
    A: Clone,
    F: Fn(&A, &T::Output) -> A,
{
    type Input = T::Input;
    type Output = T::Output;
    type State = (T::State, A);
    fn start(&self) -> Self::State {
        (self.inner.start(), self.init.clone())
    }
    fn relation(
        &self,
        (state, acc): &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        let (next_state, output) = self.inner.relation(state, input)?;
        let next_acc = (self.f)(acc, &output);
        Some(((next_state, next_acc), output))
    }
    fn accept(&self, (state, _): &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug, Clone)]
pub struct TryFoldTransducer<T, A, F> {
    inner: T,
    init: A,
    f: F,
}
impl<T, A, F> TryFoldTransducer<T, A, F> {
    pub fn new(inner: T, init: A, f: F) -> Self
    where
        T: Transducer,
        F: Fn(&A, &T::Output) -> Option<A>,
    {
        Self { inner, init, f }
    }
}
impl<T, A, F> StationaryTransducer for TryFoldTransducer<T, A, F>
where
    T: StationaryTransducer,
    A: Clone,
    F: Fn(&A, &T::Output) -> Option<A>,
{
}

impl<T, A, F> Transducer for TryFoldTransducer<T, A, F>
where
    T: Transducer,
    A: Clone,
    F: Fn(&A, &T::Output) -> Option<A>,
{
    type Input = T::Input;
    type Output = T::Output;
    type State = (T::State, A);
    fn start(&self) -> Self::State {
        (self.inner.start(), self.init.clone())
    }
    fn relation(
        &self,
        (state, acc): &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        let (next_state, output) = self.inner.relation(state, input)?;
        let next_acc = (self.f)(acc, &output)?;
        Some(((next_state, next_acc), output))
    }
    fn accept(&self, (state, _): &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug, Clone)]
pub struct AcceptTransducer<T, F> {
    inner: T,
    pred: F,
}
impl<T, F> AcceptTransducer<T, F> {
    pub fn new(inner: T, pred: F) -> Self
    where
        T: Transducer,
        F: Fn(&T::State) -> bool,
    {
        Self { inner, pred }
    }
}
impl<T, F> StationaryTransducer for AcceptTransducer<T, F>
where
    T: StationaryTransducer,
    F: Fn(&T::State) -> bool,
{
}

impl<T, F> Transducer for AcceptTransducer<T, F>
where
    T: Transducer,
    F: Fn(&T::State) -> bool,
{
    type Input = T::Input;
    type Output = T::Output;
    type State = T::State;
    fn start(&self) -> Self::State {
        self.inner.start()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        self.inner.relation(state, input)
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state) && (self.pred)(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug)]
pub struct EqualTransducer<T>(PhantomData<fn() -> T>);
impl<T> Clone for EqualTransducer<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<T> EqualTransducer<T> {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<T> Default for EqualTransducer<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> StationaryTransducer for EqualTransducer<T> where T: PartialEq {}

impl<T> Transducer for EqualTransducer<T>
where
    T: PartialEq,
{
    type Input = (T, T);
    type Output = ();
    type State = ();
    fn start(&self) -> Self::State {}
    fn relation(
        &self,
        _state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        (input.0 == input.1).then_some(((), ()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}

#[derive(Debug)]
/// DFA to accept Less/Greater than (or equal to) in lexicographical order
pub struct LexicographicalTransducer<T> {
    ordering: Ordering,
    equal: bool,
    _marker: PhantomData<fn() -> T>,
}
impl<T> Clone for LexicographicalTransducer<T> {
    fn clone(&self) -> Self {
        Self {
            ordering: self.ordering,
            equal: self.equal,
            _marker: PhantomData,
        }
    }
}
impl<T> LexicographicalTransducer<T> {
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
impl<T> StationaryTransducer for LexicographicalTransducer<T> where T: Ord {}

impl<T> Transducer for LexicographicalTransducer<T>
where
    T: Ord,
{
    type Input = (T, T);
    type Output = ();
    /// is equal
    type State = bool;
    fn start(&self) -> Self::State {
        true
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        if !state {
            return Some((false, ()));
        }
        match input.0.cmp(&input.1) {
            Ordering::Equal => Some((true, ())),
            ord if ord == self.ordering => Some((false, ())),
            _ => None,
        }
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.equal || !state
    }
}

#[derive(Debug)]
/// DFA to accept Less/Greater than (or equal to) in reversed lexicographical order
pub struct RevLexicographicalTransducer<T> {
    ordering: Ordering,
    equal: bool,
    _marker: PhantomData<fn() -> T>,
}
impl<T> Clone for RevLexicographicalTransducer<T> {
    fn clone(&self) -> Self {
        Self {
            ordering: self.ordering,
            equal: self.equal,
            _marker: PhantomData,
        }
    }
}
impl<T> RevLexicographicalTransducer<T> {
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
impl<T> StationaryTransducer for RevLexicographicalTransducer<T> where T: Ord {}

impl<T> Transducer for RevLexicographicalTransducer<T>
where
    T: Ord,
{
    type Input = (T, T);
    type Output = ();
    /// accept
    type State = bool;
    fn start(&self) -> Self::State {
        self.equal
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        Some((
            match input.0.cmp(&input.1) {
                Ordering::Equal => *state,
                ord => ord == self.ordering,
            },
            (),
        ))
    }
    fn accept(&self, state: &Self::State) -> bool {
        *state
    }
}

#[derive(Debug)]
pub struct SequenceTransducer<'a, T, A> {
    sequence: &'a [T],
    _marker: PhantomData<fn() -> A>,
}
impl<'a, T, A> Clone for SequenceTransducer<'a, T, A> {
    fn clone(&self) -> Self {
        Self {
            sequence: self.sequence,
            _marker: PhantomData,
        }
    }
}
impl<'a, T, A> SequenceTransducer<'a, T, A> {
    pub fn new(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            _marker: PhantomData,
        }
    }
}
impl<T, A> Transducer for SequenceTransducer<'_, T, A>
where
    T: Clone,
{
    type Input = A;
    type Output = T;
    type State = ();
    fn start(&self) -> Self::State {}
    fn relation(
        &self,
        _state: &Self::State,
        _input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        self.sequence.first().map(|c| ((), c.clone()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
    fn stepout(&mut self) {
        if !self.sequence.is_empty() {
            self.sequence = &self.sequence[1..];
        }
    }
}

#[derive(Debug)]
pub struct RevSequenceTransducer<'a, T, A> {
    sequence: &'a [T],
    _marker: PhantomData<fn() -> A>,
}
impl<'a, T, A> Clone for RevSequenceTransducer<'a, T, A> {
    fn clone(&self) -> Self {
        Self {
            sequence: self.sequence,
            _marker: PhantomData,
        }
    }
}
impl<'a, T, A> RevSequenceTransducer<'a, T, A> {
    pub fn new(sequence: &'a [T]) -> Self {
        Self {
            sequence,
            _marker: PhantomData,
        }
    }
}
impl<T, A> Transducer for RevSequenceTransducer<'_, T, A>
where
    T: Clone,
{
    type Input = A;
    type Output = T;
    type State = ();
    fn start(&self) -> Self::State {}
    fn relation(
        &self,
        _state: &Self::State,
        _input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        self.sequence.last().map(|c| ((), c.clone()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
    fn stepout(&mut self) {
        if !self.sequence.is_empty() {
            self.sequence = &self.sequence[..self.sequence.len() - 1];
        }
    }
}

pub struct IteratorTransducer<I, A>
where
    I: Iterator,
{
    iter: RefCell<Peekable<I>>,
    _marker: PhantomData<fn() -> A>,
}
impl<I, A> Clone for IteratorTransducer<I, A>
where
    I: Iterator<Item: Clone> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            _marker: self._marker,
        }
    }
}
impl<I, A> Debug for IteratorTransducer<I, A>
where
    I: Iterator<Item: Debug> + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("IteratorTransducer")
            .field("iter", &self.iter)
            .field("_marker", &self._marker)
            .finish()
    }
}
impl<I, A> IteratorTransducer<I, A>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        Self::new_peekable(iter.peekable())
    }
    pub fn new_peekable(iter: Peekable<I>) -> Self {
        Self {
            iter: RefCell::new(iter),
            _marker: PhantomData,
        }
    }
}
impl<I, A> Transducer for IteratorTransducer<I, A>
where
    I: Iterator<Item: Clone>,
{
    type Input = A;
    type Output = I::Item;
    type State = ();
    fn start(&self) -> Self::State {}
    fn relation(
        &self,
        _state: &Self::State,
        _input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        self.iter.borrow_mut().peek().cloned().map(|c| ((), c))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
    fn stepout(&mut self) {
        self.iter.borrow_mut().next();
    }
}

#[derive(Debug)]
pub struct MonoidalTransducer<M>(PhantomData<fn() -> M>);
impl<M> Clone for MonoidalTransducer<M> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<M> MonoidalTransducer<M> {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<M> Default for MonoidalTransducer<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<M> StationaryTransducer for MonoidalTransducer<M> where M: Monoid {}

impl<M> Transducer for MonoidalTransducer<M>
where
    M: Monoid,
{
    type Input = M::T;
    type Output = ();
    type State = M::T;
    fn start(&self) -> Self::State {
        M::unit()
    }
    fn relation(
        &self,
        state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        Some((M::operate(state, input), ()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct IdentityTransducer<I>(PhantomData<fn() -> I>);
impl<I> Clone for IdentityTransducer<I> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<I> IdentityTransducer<I> {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<I> Default for IdentityTransducer<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<I> StationaryTransducer for IdentityTransducer<I> where I: Clone {}

impl<I> Transducer for IdentityTransducer<I>
where
    I: Clone,
{
    type Input = I;
    type Output = I;
    type State = ();
    fn start(&self) -> Self::State {}
    fn relation(
        &self,
        _state: &Self::State,
        input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        Some(((), input.clone()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct AlwaysAcceptingTransducer<A>(PhantomData<fn() -> A>);
impl<A> Clone for AlwaysAcceptingTransducer<A> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<A> AlwaysAcceptingTransducer<A> {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<A> Default for AlwaysAcceptingTransducer<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<A> StationaryTransducer for AlwaysAcceptingTransducer<A> {}

impl<A> Transducer for AlwaysAcceptingTransducer<A> {
    type Input = A;
    type Output = ();
    type State = ();
    fn start(&self) -> Self::State {}
    fn relation(
        &self,
        _state: &Self::State,
        _input: &Self::Input,
    ) -> Option<(Self::State, Self::Output)> {
        Some(((), ()))
    }
    fn accept(&self, _state: &Self::State) -> bool {
        true
    }
}

/// build transducer
///
/// - `transducer!(A)`
/// - `<= seq`, `seq >=`: [`LexicographicalTransducer::less_than_or_equal()`](LexicographicalTransducer::less_than_or_equal) with [`SequenceTransducer`]
/// - `>= seq`, `seq <=`: [`LexicographicalTransducer::greater_than_or_equal()`](LexicographicalTransducer::greater_than_or_equal) with [`SequenceTransducer`]
/// - `< seq`, `seq >`: [`LexicographicalTransducer::less_than()`](LexicographicalTransducer::less_than) with [`SequenceTransducer`]
/// - `> seq`, `seq <`: [`LexicographicalTransducer::greater_than()`](LexicographicalTransducer::greater_than) with [`SequenceTransducer`]
/// - `!<= seq`, `seq !>=`: [`RevLexicographicalTransducer::less_than_or_equal()`](RevLexicographicalTransducer::less_than_or_equal) with [`RevSequenceTransducer`]
/// - `!>= seq`, `seq !<=`: [`RevLexicographicalTransducer::greater_than_or_equal()`](RevLexicographicalTransducer::greater_than_or_equal) with [`RevSequenceTransducer`]
/// - `!< seq`, `seq !>`: [`RevLexicographicalTransducer::less_than()`](RevLexicographicalTransducer::less_than) with [`RevSequenceTransducer`]
/// - `!> seq`, `seq !<`: [`RevLexicographicalTransducer::greater_than()`](RevLexicographicalTransducer::greater_than) with [`RevSequenceTransducer`]
/// - `<=`: [`LexicographicalTransducer::less_than_or_equal()`](LexicographicalTransducer::less_than_or_equal)
/// - `>=`: [`LexicographicalTransducer::greater_than_or_equal()`](LexicographicalTransducer::greater_than_or_equal)
/// - `<`: [`LexicographicalTransducer::less_than()`](LexicographicalTransducer::less_than)
/// - `>`: [`LexicographicalTransducer::greater_than()`](LexicographicalTransducer::greater_than)
/// - `!<=`: [`RevLexicographicalTransducer::less_than_or_equal()`](RevLexicographicalTransducer::less_than_or_equal)
/// - `!>=`: [`RevLexicographicalTransducer::greater_than_or_equal()`](RevLexicographicalTransducer::greater_than_or_equal)
/// - `!<`: [`RevLexicographicalTransducer::less_than()`](RevLexicographicalTransducer::less_than)
/// - `!>`: [`RevLexicographicalTransducer::greater_than()`](RevLexicographicalTransducer::greater_than)
/// - `=> f g h`: [`FunctionalTransducer::new(f, g, h)`](FunctionalTransducer)
/// - `@id`: [`IdentityTransducer::new()`](IdentityTransducer)
/// - `@it e`: [`IteratorTransducer::new(e)`](IteratorTransducer)
/// - `@map f`: `@id |>` [`map(f)`](Transducer::map)
/// - `@try_map f`: `@id |>` [`try_map(f)`](Transducer::try_map)
/// - `@seq e`: [`SequenceTransducer::new(e)`](SequenceTransducer)
/// - `@rseq e`: [`RevSequenceTransducer::new(e)`](RevSequenceTransducer)
/// - `@`: [`AlwaysAcceptingTransducer::new()`](AlwaysAcceptingTransducer)
/// - `A |> method(args...)`: [`Transducer`]`::method(A, args...)`
/// - `A |> map(f)`: [`Transducer::map(A, f)`](Transducer::map)
/// - `A |> try_map(f)`: [`Transducer::try_map(A, f)`](Transducer::try_map)
/// - `A |> retain(f)`: [`Transducer::retain(A, f)`](Transducer::retain)
/// - `A |> with_fold(init, f)`: [`Transducer::with_fold(A, init, f)`](Transducer::with_fold)
/// - `A |> with_try_fold(init, f)`: [`Transducer::with_try_fold(A, init, f)`](Transducer::with_try_fold)
/// - `A |> accepting(f)`: [`Transducer::accepting(A, f)`](Transducer::accepting)
/// - `A . B`: [`ChainTransducer((A, B))`](ChainTransducer)
/// - `A * B`: [`ProductTransducer((A, B))`](ProductTransducer)
/// - `A & B`: [`IntersectionTransducer((A, B))`](IntersectionTransducer)
/// - Use a block for Rust expressions containing DSL operators: `{ machine.clone() }`.
#[macro_export]
macro_rules! transducer {
    (@check $e:expr)                                         => {{ #[inline(always)] fn check_transucer<T>(fst: T) -> T where T: Transducer { fst } check_transucer($e) }};
    (@inner ($($t:tt)*))                                     => { $crate::transducer!(@inner $($t)*) };
    (@inner => $f:expr, $g:expr, $h:expr $(,)?)                => { $crate::transducer!(@check FunctionalTransducer::new($f, $g, $h)) };
    (@inner = $e:expr)                                       => { $crate::transducer!(((@id & (@seq &$e)) . =)) };
    (@inner <= $e:expr)                                      => { $crate::transducer!(((@id & (@seq &$e)) . <=)) };
    (@inner >= $e:expr)                                      => { $crate::transducer!(((@id & (@seq &$e)) . >=)) };
    (@inner < $e:expr)                                       => { $crate::transducer!(((@id & (@seq &$e)) . <)) };
    (@inner > $e:expr)                                       => { $crate::transducer!(((@id & (@seq &$e)) . >)) };
    (@inner !<= $e:expr)                                     => { $crate::transducer!(((@id & (@rseq &$e)) . !<=)) };
    (@inner !>= $e:expr)                                     => { $crate::transducer!(((@id & (@rseq &$e)) . !>=)) };
    (@inner !< $e:expr)                                      => { $crate::transducer!(((@id & (@rseq &$e)) . !<)) };
    (@inner !> $e:expr)                                      => { $crate::transducer!(((@id & (@rseq &$e)) . !>)) };
    (@inner $e:ident =)                                      => { $crate::transducer!(@inner = $e) };
    (@inner $e:ident <=)                                     => { $crate::transducer!(@inner >= $e) };
    (@inner $e:ident >=)                                     => { $crate::transducer!(@inner <= $e) };
    (@inner $e:ident <)                                      => { $crate::transducer!(@inner > $e) };
    (@inner $e:ident >)                                      => { $crate::transducer!(@inner < $e) };
    (@inner $e:ident !<=)                                    => { $crate::transducer!(@inner !>= $e) };
    (@inner $e:ident !>=)                                    => { $crate::transducer!(@inner !<= $e) };
    (@inner $e:ident !<)                                     => { $crate::transducer!(@inner !> $e) };
    (@inner $e:ident !>)                                     => { $crate::transducer!(@inner !< $e) };
    (@inner =)                                               => { $crate::transducer!(@check EqualTransducer::new()) };
    (@inner <=)                                              => { $crate::transducer!(@check LexicographicalTransducer::less_than_or_equal()) };
    (@inner >=)                                              => { $crate::transducer!(@check LexicographicalTransducer::greater_than_or_equal()) };
    (@inner <)                                               => { $crate::transducer!(@check LexicographicalTransducer::less_than()) };
    (@inner >)                                               => { $crate::transducer!(@check LexicographicalTransducer::greater_than()) };
    (@inner !<=)                                             => { $crate::transducer!(@check RevLexicographicalTransducer::less_than_or_equal()) };
    (@inner !>=)                                             => { $crate::transducer!(@check RevLexicographicalTransducer::greater_than_or_equal()) };
    (@inner !<)                                              => { $crate::transducer!(@check RevLexicographicalTransducer::less_than()) };
    (@inner !>)                                              => { $crate::transducer!(@check RevLexicographicalTransducer::greater_than()) };
    (@inner @id)                                             => { $crate::transducer!(@check IdentityTransducer::new()) };
    (@inner @it $e:expr)                                     => { $crate::transducer!(@check IteratorTransducer::new($e)) };
    (@inner @map $f:expr)                                    => { $crate::transducer!(@check Transducer::map(IdentityTransducer::new(), $f)) };
    (@inner @try_map $f:expr)                                => { $crate::transducer!(@check Transducer::try_map(IdentityTransducer::new(), $f)) };
    (@inner @seq $e:expr)                                    => { $crate::transducer!(@check SequenceTransducer::new($e)) };
    (@inner @rseq $e:expr)                                   => { $crate::transducer!(@check RevSequenceTransducer::new($e)) };
    (@inner @<$t:ty>)                                        => { $crate::transducer!(@check AlwaysAcceptingTransducer::<$t>::new()) };
    (@inner @)                                               => { $crate::transducer!(@check AlwaysAcceptingTransducer::new()) };
    (@inner $e:ident)                                        => { $crate::transducer!(@check $e) };
    (@inner $e:block)                                        => { $crate::transducer!(@check $e) };
    (@inner $($t:tt)*)                                       => { $crate::transducer!(@inter [] [] $($t)*) };
    (@inter [$([$($a:tt)*])*])                               => { $crate::transducer!(@check IntersectionTransducer(($($crate::transducer!(@inner $($a)*),)*))) };
    (@inter [] [$($b:tt)*])                                  => { $crate::transducer!(@prod [] [] $($b)*) };
    (@inter [$($a:tt)*] [$($b:tt)*])                         => { $crate::transducer!(@inter [$($a)* [$($b)*]]) };
    (@inter [$($a:tt)*] [$($b:tt)*] & $($t:tt)*)             => { $crate::transducer!(@inter [$($a)* [$($b)*]] [] $($t)*) };
    (@inter [$($a:tt)*] [$($b:tt)*] $op:tt $($t:tt)*)        => { $crate::transducer!(@inter [$($a)*] [$($b)* $op] $($t)*) };
    (@prod [$([$($a:tt)*])*])                                => { $crate::transducer!(@check ProductTransducer(($($crate::transducer!(@inner $($a)*),)*))) };
    (@prod [] [$($b:tt)*])                                   => { $crate::transducer!(@chain [] [] $($b)*) };
    (@prod [$($a:tt)*] [$($b:tt)*])                          => { $crate::transducer!(@prod [$($a)* [$($b)*]]) };
    (@prod [$($a:tt)*] [$($b:tt)*] * $($t:tt)*)              => { $crate::transducer!(@prod [$($a)* [$($b)*]] [] $($t)*) };
    (@prod [$($a:tt)*] [$($b:tt)*] $op:tt $($t:tt)*)         => { $crate::transducer!(@prod [$($a)*] [$($b)* $op] $($t)*) };
    (@chain [$([$($a:tt)*])*])                               => { $crate::transducer!(@check ChainTransducer(($($crate::transducer!(@wrap [] $($a)*),)*))) };
    (@chain [] [$($b:tt)*])                                  => { $crate::transducer!(@wrap [] $($b)*) };
    (@chain [$($a:tt)*] [$($b:tt)*])                         => { $crate::transducer!(@chain [$($a)* [$($b)*]]) };
    (@chain [$($a:tt)*] [$($b:tt)*] . $($t:tt)*)             => { $crate::transducer!(@chain [$($a)* [$($b)*]] [] $($t)*) };
    (@chain [$($a:tt)*] [$($b:tt)*] $op:tt $($t:tt)*)        => { $crate::transducer!(@chain [$($a)*] [$($b)* $op] $($t)*) };
    (@wrap [$($b:tt)*])                                      => { $crate::transducer!(@inner $($b)*) };
    (@wrap [$($b:tt)*] |> $($t:tt)*)                         => { $crate::transducer!(@wrap_apply [$crate::transducer!(@inner $($b)*)] $($t)*) };
    (@wrap [$($b:tt)*] $op:tt $($t:tt)*)                     => { $crate::transducer!(@wrap [$($b)* $op] $($t)*) };
    (@wrapped [$e:expr])                                     => { $crate::transducer!(@check $e) };
    (@wrapped [$e:expr] |> $($t:tt)*)                        => { $crate::transducer!(@wrap_apply [$e] $($t)*) };
    (@wrap_apply [$e:expr] $method:ident($($args:expr),* $(,)?) $($t:tt)*) => { $crate::transducer!(@wrapped [Transducer::$method($e $(, $args)*)] $($t)*) };
    (@id $($t:tt)*)                                          => { $crate::transducer!(@inner @id $($t)*) };
    (@it $($t:tt)*)                                          => { $crate::transducer!(@inner @it $($t)*) };
    (@map $($t:tt)*)                                         => { $crate::transducer!(@inner @map $($t)*) };
    (@try_map $($t:tt)*)                                     => { $crate::transducer!(@inner @try_map $($t)*) };
    (@seq $($t:tt)*)                                         => { $crate::transducer!(@inner @seq $($t)*) };
    (@rseq $($t:tt)*)                                        => { $crate::transducer!(@inner @rseq $($t)*) };
    (@$tag:ident $($t:tt)*)                                  => { ::std::compile_error!(::std::stringify!($tag, $($t)*)) };
    ($($t:tt)*)                                              => {{ $crate::transducer!(@inner $($t)*) }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algebra::{AdditiveOperation, ConcatenateOperation, MinOperation},
        data_structure::{FibonacciHasher, VecMap},
        num::mint_basic::MInt998244353,
        tools::{
            NotEmptySegment, ToDigitSequence, Xorshift,
            testutil::{exhaustive_sequences, sequence_cases},
        },
    };
    use std::{cell::Cell, collections::HashMap, hash::BuildHasherDefault, iter::repeat_n};

    #[test]
    fn test_lexicographical_bound_prefixes() {
        for bound in exhaustive_sequences(0..3, 0..=4) {
            for input in exhaustive_sequences(0..3, 0..=5) {
                let machines = [
                    transducer!(< bound),
                    transducer!(<= bound),
                    transducer!(>= bound),
                    transducer!(> bound),
                    transducer!(bound >),
                    transducer!(bound >=),
                    transducer!(bound <=),
                    transducer!(bound <),
                ];
                for (kind, mut machine) in machines.into_iter().enumerate() {
                    let mut state = Some(machine.start());
                    for len in 0..=input.len() {
                        if len > 0 {
                            state = state.and_then(|s| {
                                machine.relation(&s, &input[len - 1]).map(|(s, ())| s)
                            });
                            machine.stepout();
                        }
                        let expected = len <= bound.len() && {
                            let ordering = input[..len].cmp(&bound[..len]);
                            match kind % 4 {
                                0 => ordering.is_lt(),
                                1 => ordering.is_le(),
                                2 => ordering.is_ge(),
                                _ => ordering.is_gt(),
                            }
                        };
                        assert_eq!(state.is_some_and(|s| machine.accept(&s)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn test_revlexicographical_bound_prefixes() {
        for bound in exhaustive_sequences(0..3, 0..=4) {
            for input in exhaustive_sequences(0..3, 0..=5) {
                let machines = [
                    transducer!(!< bound),
                    transducer!(!<= bound),
                    transducer!(!>= bound),
                    transducer!(!> bound),
                    transducer!(bound !>),
                    transducer!(bound !>=),
                    transducer!(bound !<=),
                    transducer!(bound !<),
                ];
                for (kind, mut machine) in machines.into_iter().enumerate() {
                    let mut state = Some(machine.start());
                    for len in 0..=input.len() {
                        if len > 0 {
                            state = state.and_then(|s| {
                                machine.relation(&s, &input[len - 1]).map(|(s, ())| s)
                            });
                            machine.stepout();
                        }
                        let expected = len <= bound.len() && {
                            let ordering =
                                input[..len].iter().rev().cmp(&bound[bound.len() - len..]);
                            match kind % 4 {
                                0 => ordering.is_lt(),
                                1 => ordering.is_le(),
                                2 => ordering.is_ge(),
                                _ => ordering.is_gt(),
                            }
                        };
                        assert_eq!(state.is_some_and(|s| machine.accept(&s)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn test_equal_bound_prefixes() {
        for bound in exhaustive_sequences(0..3, 0..=4) {
            for input in exhaustive_sequences(0..3, 0..=5) {
                for mut machine in [transducer!(= bound), transducer!(bound =)] {
                    let mut state = Some(machine.start());
                    for len in 0..=input.len() {
                        if len > 0 {
                            state = state.and_then(|s| {
                                machine.relation(&s, &input[len - 1]).map(|(s, ())| s)
                            });
                            machine.stepout();
                        }
                        let expected = len <= bound.len() && input[..len] == bound[..len];
                        assert_eq!(state.is_some_and(|s| machine.accept(&s)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_sequence_chain() {
        for bound in exhaustive_sequences(0..3, 0..=3) {
            for input in exhaustive_sequences(0..3, 0..=4) {
                let inner = transducer!(=> || 0usize,
                    |&sum, &(input, bound)| Some((sum + input * bound, input + bound)),
                    |_| true)
                .intersection(SequenceTransducer::new(&bound));
                let forward = trace(
                    &mut IdentityTransducer::new()
                        .intersection(SequenceTransducer::new(&bound))
                        .chain(inner.clone()),
                    input.iter().copied(),
                );
                let backward = trace(
                    &mut IdentityTransducer::new()
                        .intersection(RevSequenceTransducer::new(&bound))
                        .chain(inner),
                    input.iter().copied(),
                );
                for (reversed, result) in [(false, forward), (true, backward)] {
                    let mut sum = 0;
                    let expected: Vec<_> = input
                        .iter()
                        .zip(&bound)
                        .enumerate()
                        .map(|(i, (&input, &tag))| {
                            let digit = bound[if reversed { bound.len() - i - 1 } else { i }];
                            sum += input * digit;
                            ((((), ()), (sum, ())), (input + digit, tag))
                        })
                        .collect();
                    assert_eq!(result, expected);
                }
            }
        }
    }

    #[test]
    fn test_popcount_sum() {
        type M = (AdditiveOperation<u64>, AdditiveOperation<u64>);
        for upper in 0usize..256 {
            let digits = upper.to_digit_sequence_radix_len(2, 8);
            for k in 0..=9 {
                let machine = transducer!(=> || 0usize, |&ones, &bit| Some((ones + bit, ())), |&ones| ones == k)
                    .intersection(transducer!(<= digits))
                    .with_input();
                let candidates = |&(ones, _): &(_, _)| 0..=usize::from(ones < k);
                let expected = (0..=upper)
                    .filter(|x| x.count_ones() as usize == k)
                    .fold((0, 0), |(sum, count), x| (sum + x as u64, count + 1));
                let effect =
                    |&(sum, count): &(u64, u64), (_, bit)| (2 * sum + count * bit as u64, count);
                assert_eq!(
                    machine
                        .clone()
                        .dp::<M>((0, 1))
                        .run_effect(8, candidates, effect),
                    expected
                );
                let create =
                    || VecMap::with_len(20, |&(ones, (_, equal))| ones * 2 + usize::from(equal));
                assert_eq!(
                    machine
                        .clone()
                        .dp_with::<M, _>((0, 1), create)
                        .run_effect(8, candidates, effect),
                    expected
                );
                assert_eq!(machine.counting::<u64>().run(8, candidates), expected.1);
                let count = transducer!(=> || 0usize, |&ones, &bit| Some((ones + bit, ())), |&ones| ones == k)
                    .intersection(transducer!(<= digits))
                    .counting::<MInt998244353>()
                    .run(8, candidates);
                assert_eq!(count, MInt998244353::from(expected.1));
            }
        }
    }

    #[test]
    fn test_payment() {
        let digit_sums: Vec<usize> = (0usize..2000)
            .map(|x| x.to_string().bytes().map(|c| (c - b'0') as usize).sum())
            .collect();
        for n in 0usize..1000 {
            let machine = transducer!(=> || 0usize, |&carry, &(change, digit)| {
                let total = carry + change + digit;
                Some((total / 10, total % 10 + change))
            }, |&carry| carry == 0);
            let mut dp = machine
                .dp_with::<MinOperation<usize>, _>(0, || VecMap::with_len(2, |&carry| carry));
            for digit in n.to_digit_sequence().into_iter().rev().chain([0]) {
                dp.step_effect(
                    |_| (0..10).map(|change| (change, digit)),
                    |&cost, edge_cost| cost + edge_cost,
                );
            }
            // Removing common multiples of 1000 cannot increase the total digit sum.
            let expected = (n..n + 1000)
                .map(|paid| digit_sums[paid] + digit_sums[paid - n])
                .min()
                .unwrap();
            assert_eq!(dp.fold_accept(), expected, "n={n}");
        }
    }

    #[test]
    fn test_leading_zero_constraints() {
        for upper in 0usize..1000 {
            let digits = upper.to_digit_sequence_radix_len(10, 3);
            let machine = transducer!(=> || (false, 0usize, 0usize),
            |&(started, remainder, mask), &digit| {
                let started = started || digit != 0;
                let mask = if started { mask | 1usize << digit } else { mask };
                Some(((started, (remainder * 10 + digit) % 7, mask), ()))
            },
            |&(started, remainder, mask)| {
                started && remainder == 0 && mask.count_ones() <= 2 && mask & (1 << 3) != 0
            })
            .intersection(transducer!(>= [0, 0, 7]))
            .intersection(transducer!(<= digits));
            let expected = (7..=upper)
                .filter(|x| {
                    let s = x.to_string();
                    let mut bytes = s.bytes().collect::<Vec<_>>();
                    bytes.sort_unstable();
                    bytes.dedup();
                    x % 7 == 0 && bytes.len() <= 2 && s.contains('3')
                })
                .count();
            let create = || {
                HashMap::with_capacity_and_hasher(
                    128,
                    BuildHasherDefault::<FibonacciHasher>::default(),
                )
            };
            assert_eq!(
                machine.counting_with::<usize, _>(create).run(3, |_| 0..10),
                expected
            );
        }
    }

    #[test]
    fn test_transducer_map_input() {
        let mut rng = Xorshift::default();
        for inputs in transducer_inputs(&mut rng) {
            let machine = sum_transducer(0).accepting(|sum| sum.is_multiple_of(2));
            let mut mapped = machine.map_input(|&(x, y): &(usize, usize)| x + y);
            let mut sum = 0;
            for (i, &x) in inputs.iter().enumerate() {
                let previous = sum;
                sum += i + x;
                assert_eq!(mapped.relation(&previous, &(i, x)), Some((sum, sum)));
                assert_eq!(mapped.accept(&sum), sum.is_multiple_of(2));
                mapped.stepout();
            }
        }
    }

    #[test]
    fn test_transducer_short_circuit() {
        for reject in 0..=3 {
            let calls = [Cell::new(0), Cell::new(0), Cell::new(0)];
            let machine = |i: usize| {
                let calls = &calls;
                transducer!(=> || (), move |&(), &x: &usize| {
                    calls[i].set(calls[i].get() + 1);
                    (i != reject).then_some(((), x))
                }, |_| true)
            };
            let intersection = IntersectionTransducer((machine(0), machine(1), machine(2)));
            let product = ProductTransducer((machine(0), machine(1), machine(2)));
            assert_eq!(
                intersection.relation(&((), (), ()), &7).is_some(),
                reject == 3
            );
            for (i, count) in calls.iter().enumerate() {
                assert_eq!(count.replace(0), usize::from(i <= reject));
            }
            assert_eq!(
                product.relation(&((), (), ()), &(1, 2, 3)).is_some(),
                reject == 3
            );
            for (i, count) in calls.iter().enumerate() {
                assert_eq!(count.get(), usize::from(i <= reject));
            }
        }
        let filter = transducer!(=> || (), |&(), &keep: &bool| keep.then_some(((), ())), |_| true);
        let mut intersection = filter
            .clone()
            .intersection(SequenceTransducer::new(&[3, 5]));
        assert!(intersection.relation(&((), ()), &false).is_none());
        intersection.stepout();
        assert_eq!(
            intersection.relation(&((), ()), &true),
            Some((((), ()), ((), 5)))
        );
        let mut product = filter.product(SequenceTransducer::new(&[3, 5]));
        assert!(product.relation(&((), ()), &(false, ())).is_none());
        product.stepout();
        assert_eq!(
            product.relation(&((), ()), &(true, ())),
            Some((((), ()), ((), 5)))
        );
    }

    #[test]
    fn test_transducer_empty_layer() {
        let machine = SequenceTransducer::<_, usize>::new(&[2usize, 3, 5])
            .map_input(|&(): &()| 0)
            .intersection(AlwaysAcceptingTransducer::new());
        let initial = machine.start();
        let mut dp = machine.counting::<usize>();
        dp.step(|_| [] as [(); 0]);
        assert_eq!(dp.fold_accept(), 0);
        dp.dp.insert(initial, 1);
        dp.step_effect(|_| [()], |&weight, (output, ())| weight * output);
        assert_eq!(dp.fold_accept(), 3);
        let mut branch = dp.clone();
        assert_eq!(
            branch.run_effect(1, |_| [()], |&weight, (output, ())| weight * output),
            15
        );
        dp.step(|_| [] as [(); 0]);
        dp.dp.insert(initial, 1);
        dp.step(|_| [()]);
        assert_eq!(dp.fold_accept(), 0);
    }

    #[test]
    fn test_transducer_owned_outputs() {
        struct Output(usize);
        struct State(usize);
        let machine = FunctionalTransducer::new(
            || State(0),
            |state: &State, &x: &usize| Some((State(state.0 + x), Output(x))),
            |state: &State| state.0 <= 4,
        )
        .with_input()
        .map(|(Output(x), input)| Output(x + input))
        .try_map(|Output(x)| (x <= 4).then_some(Output(x)))
        .map_input(|x: &Output| x.0);
        let mut calls = 0;
        let create = || {
            calls += 1;
            VecMap::with_len(7, |state: &State| state.0)
        };
        let mut dp = machine
            .clone()
            .dp_with::<AdditiveOperation<usize>, _>(1, create);
        assert_eq!(calls, 2);
        dp.step_effect(
            |_| [Output(1), Output(2), Output(3)],
            |&weight, Output(x)| weight * x,
        );
        assert_eq!(
            dp.iter_accept().map(|(s, &v)| (s.0, v)).collect::<Vec<_>>(),
            [(1, 2), (2, 4)]
        );
        dp.step_effect(
            |_| [Output(1), Output(2), Output(3)],
            |&weight, Output(x)| weight * x,
        );
        assert_eq!(dp.fold_accept(), 36);
        dp.step_effect(
            |_| [Output(1), Output(2), Output(3)],
            |&weight, Output(x)| weight * x,
        );
        assert_eq!(dp.fold_accept(), 56);
    }

    #[test]
    fn test_transducer_ordered_aggregation() {
        for len in 0..=8 {
            let machine = transducer!(=> || 0usize, |_, &bit: &usize| Some((bit, bit)), |_| true);
            let actual = machine
                .dp_with::<ConcatenateOperation<usize>, _>(vec![0], || {
                    VecMap::with_len(2, |&state| state)
                })
                .run_effect(
                    len,
                    |_| 0..2,
                    |weights, bit| weights.iter().map(|x| 2 * x + bit).collect(),
                );
            let expected: Vec<_> = (0..1 << len)
                .map(|bits| (0..len).fold(0, |value, i| 2 * value + ((bits >> i) & 1)))
                .collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_transducer_expression_leaves() {
        let machine = transducer!(=> || 0usize, |&s, &x: &usize| Some((s + x, ())), |_| true);
        let leaf = transducer!({ machine.clone() });
        assert_eq!(leaf.relation(&0, &1), Some((1, ())));
        let leaf = transducer!(machine);
        assert_eq!(leaf.relation(&0, &2), Some((2, ())));
        let composition = transducer!(
            leaf & { FunctionalTransducer::new(|| 0usize, |&s, &x| Some((s + x, ())), |_| true) }
        );
        assert_eq!(composition.relation(&(0, 0), &3), Some(((3, 3), ((), ()))));
    }

    #[test]
    fn test_equal_transducer() {
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for ((l, r), radix) in rng
            .random_iter((NotEmptySegment(10usize.pow(9)), 2..=10))
            .take(Q)
        {
            let rr = r.to_digit_sequence_radix(radix);
            let ll = l.to_digit_sequence_radix_len(radix, rr.len());
            let n = r - l;
            assert_eq!(
                n,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & =)
                    .counting::<usize>()
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
        }
    }

    #[test]
    fn test_lexicographical_transducer() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for ((l, r), radix) in rng
            .random_iter((NotEmptySegment(10usize.pow(9)), 2..=10))
            .take(Q)
        {
            let rr = r.to_digit_sequence_radix(radix);
            let ll = l.to_digit_sequence_radix_len(radix, rr.len());
            let n = r - l;
            assert_eq!(
                n * (n + 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & <=)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
            assert_eq!(
                n * (n + 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & >=)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & <)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & >)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
        }
    }

    #[test]
    fn test_revlexicographical_transducer() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for ((l, r), radix) in rng
            .random_iter((NotEmptySegment(10usize.pow(9)), 2..=10))
            .take(Q)
        {
            let rr = r.to_digit_sequence_radix(radix);
            let ll = l.to_digit_sequence_radix_len(radix, rr.len());
            let n = r - l;
            assert_eq!(
                n * (n + 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !<=)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
            assert_eq!(
                n * (n + 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !>=)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !<)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !>)
                    .dp::<A>(1)
                    .run(ll.len(), |_| (0..radix * radix)
                        .map(|x| (x / radix, x % radix)))
            );
        }
    }

    #[test]
    fn test_lexicographical_sequence() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r) in rng.random_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(n + 1, transducer!(<= nd).dp::<A>(1).run(nd.len(), |_| 0..r));
            assert_eq!(n, transducer!(< nd).dp::<A>(1).run(nd.len(), |_| 0..r));
            assert_eq!(
                r.pow(nd.len() as _) - n,
                transducer!(>= nd).dp::<A>(1).run(nd.len(), |_| 0..r)
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                transducer!(> nd).dp::<A>(1).run(nd.len(), |_| 0..r)
            );
        }
    }

    #[test]
    fn test_revlexicographical_sequence() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r) in rng.random_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(
                n + 1,
                transducer!(!<= nd).dp::<A>(1).run(nd.len(), |_| 0..r)
            );
            assert_eq!(n, transducer!(!< nd).dp::<A>(1).run(nd.len(), |_| 0..r));
            assert_eq!(
                r.pow(nd.len() as _) - n,
                transducer!(!>= nd).dp::<A>(1).run(nd.len(), |_| 0..r)
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                transducer!(!> nd).dp::<A>(1).run(nd.len(), |_| 0..r)
            );
        }
    }

    #[test]
    fn test_prim() {
        type A = AdditiveOperation<usize>;
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        for (n, r, c) in rng
            .random_iter((0..10usize.pow(18), 2..=10, 2..200))
            .take(Q)
        {
            let nd = n.to_digit_sequence_radix(r);
            let fst = transducer!((< nd) & (=> || 0usize, |s, a| Some(((s * r + a) % c, ())), |s| *s == 0));
            assert_eq!(
                n.div_ceil(c),
                fst.clone().dp::<A>(1).run(nd.len(), |_| 0..r)
            );

            assert_eq!(
                n.div_ceil(c),
                fst.dp_with::<A, _>(1, || VecMap::new(|&((_, s0), s1)| s1 * 2 + s0 as usize))
                    .run(nd.len(), |_| 0..r)
            );
        }
    }

    #[test]
    fn test_add_lte() {
        const Q: usize = 100;
        let mut rng = Xorshift::default();
        // (x, y) where x + a <= y, l <= x, y <= r
        for ((l, r), a) in rng
            .random_iter((NotEmptySegment(100usize), 0usize..100))
            .take(Q)
        {
            let ll = l.to_digit_sequence_radix_len(2, 20);
            let rr = r.to_digit_sequence_radix_len(2, 20);
            let aa = a.to_digit_sequence_radix_len(2, 20);

            let fst = transducer!(
                ((ll !<=) * (!<= rr)) & (
                    (
                        (
                            ((@rseq &aa) & (@id))
                            |> map(|(a, (x, _y))| x + a)
                            . (=> || 0usize, |s, i| Some(((s + i) / 2, (s + i) % 2)), |s| *s == 0)
                        ) & (@id |> map(|(_x, y)| y))
                    ) . (!<=)
                )
            );

            let result = fst
                .counting::<usize>()
                .run(20, |_| (0usize..4).map(|bit| (bit & 1, (bit >> 1) & 1)));
            let expected: usize = (l..=r)
                .map(|x| (l..=r).filter(|&y| x + a <= y).count())
                .sum();
            assert_eq!(expected, result);
        }
    }

    fn trace<T>(
        fst: &mut T,
        inputs: impl IntoIterator<Item = T::Input>,
    ) -> Vec<(T::State, T::Output)>
    where
        T: Transducer<State: Clone>,
    {
        let mut state = fst.start();
        let mut results = vec![];
        for input in inputs {
            if let Some((next_state, output)) = fst.relation(&state, &input) {
                results.push((next_state.clone(), output));
                state = next_state;
                fst.stepout();
            } else {
                break;
            }
        }
        results
    }

    fn sum_transducer(
        initial: usize,
    ) -> impl Transducer<Input = usize, State = usize, Output = usize> + Clone {
        transducer!(=> move || initial,
            |state: &usize, input: &usize| Some((state + input, state + input)),
            |_: &usize| true)
    }

    fn transducer_inputs(rng: &mut Xorshift) -> impl Iterator<Item = Vec<usize>> {
        sequence_cases(rng, 0..=3, 0..=256)
            .map(|values| values.into_iter().map(|i| [0, 1, 2, 10][i]).collect())
    }

    #[test]
    fn test_transducer_with_input() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let base = sum_transducer(initial).accepting(|sum| sum.is_multiple_of(2));
                let mut with_input = base.clone().with_input();
                assert_eq!(with_input.start(), initial);
                assert_eq!(
                    trace(&mut with_input, inputs.iter().copied()),
                    sums.iter()
                        .zip(&inputs)
                        .map(|(&s, &x)| (s, (s, x)))
                        .collect::<Vec<_>>()
                );
                for state in std::iter::once(initial).chain(sums) {
                    assert_eq!(with_input.accept(&state), state.is_multiple_of(2));
                }
            }
            let mut with_input = SequenceTransducer::new(&inputs).with_input();
            for i in 0..=inputs.len() {
                assert_eq!(
                    with_input.relation(&(), &i),
                    inputs.get(i).map(|&value| ((), (value, i)))
                );
                with_input.stepout();
            }
        }
    }

    #[test]
    fn test_transducer_intersection() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let other_initial = initial + 1;
                let mut intersection =
                    sum_transducer(initial).intersection(sum_transducer(other_initial));
                assert_eq!(intersection.start(), (initial, other_initial));
                assert_eq!(
                    trace(&mut intersection, inputs.iter().copied()),
                    sums.iter()
                        .map(|&s| ((s, s + 1), (s, s + 1)))
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn test_transducer_product() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            let n = inputs.len();
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let other_initial = initial + 1;
                let other: Vec<usize> = rng.random_iter(0..=10).take(n).collect();
                let expected: Vec<_> = sums
                    .iter()
                    .zip(&other)
                    .scan(other_initial, |sum, (&left, &right)| {
                        *sum += right;
                        Some(((left, *sum), (left, *sum)))
                    })
                    .collect();
                let mut product = sum_transducer(initial).product(sum_transducer(other_initial));
                assert_eq!(product.start(), (initial, other_initial));
                assert_eq!(
                    trace(&mut product, inputs.iter().copied().zip(other)),
                    expected
                );
            }
        }
    }

    #[test]
    fn test_transducer_chain() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let base = sum_transducer(initial);
                let other_initial = initial + 1;
                for factor in (0..=2).chain([rng.random(0..=1024)]) {
                    let mut chain = base
                        .clone()
                        .chain(sum_transducer(other_initial).map(|x: usize| x * factor));
                    assert_eq!(chain.start(), (initial, other_initial));
                    assert_eq!(
                        trace(&mut chain, inputs.iter().copied()),
                        sums.iter()
                            .scan(other_initial, |sum, &value| {
                                *sum += value;
                                Some(((value, *sum), *sum * factor))
                            })
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    #[test]
    fn test_transducer_map() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                for factor in (0..=2).chain([rng.random(0..=1024)]) {
                    let mut mapped = transducer!((=> || initial,
                |state: &usize, input: &usize| Some((state + input, state + input)),
                |state: &usize| state.is_multiple_of(2)) |> map(|x: usize| x * factor));
                    assert_eq!(mapped.start(), initial);
                    assert_eq!(mapped.accept(&mapped.start()), initial.is_multiple_of(2));
                    let actual = trace(&mut mapped, inputs.iter().copied());
                    assert_eq!(
                        actual,
                        sums.iter().map(|&s| (s, s * factor)).collect::<Vec<_>>()
                    );
                    for (state, _) in actual {
                        assert_eq!(mapped.accept(&state), state.is_multiple_of(2));
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_try_map() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let base = sum_transducer(initial).accepting(|sum: &usize| sum.is_multiple_of(2));
                for factor in (0..=2).chain([rng.random(0..=1024)]) {
                    let mut limits: Vec<_> = [initial]
                        .into_iter()
                        .chain(sums.iter().copied())
                        .flat_map(|s| [s.saturating_sub(1), s, s + 1])
                        .collect();
                    limits.sort_unstable();
                    limits.dedup();
                    for limit in limits {
                        let mut filtered = base
                            .clone()
                            .try_map(|x: usize| (x <= limit).then_some(x * factor));
                        assert_eq!(
                            filtered.accept(&filtered.start()),
                            initial.is_multiple_of(2)
                        );
                        let actual = trace(&mut filtered, inputs.iter().copied());
                        assert_eq!(
                            actual,
                            sums.iter()
                                .take_while(|&&s| s <= limit)
                                .map(|&s| (s, s * factor))
                                .collect::<Vec<_>>()
                        );
                        for (state, _) in actual {
                            assert_eq!(filtered.accept(&state), state.is_multiple_of(2));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_retain() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let base = sum_transducer(initial).accepting(|sum: &usize| sum.is_multiple_of(2));
                let mut limits: Vec<_> = [initial]
                    .into_iter()
                    .chain(sums.iter().copied())
                    .flat_map(|s| [s.saturating_sub(1), s, s + 1])
                    .collect();
                limits.sort_unstable();
                limits.dedup();
                for limit in limits {
                    let mut retained = RetainTransducer::new(base.clone(), |x| x.le(&limit));
                    assert_eq!(
                        retained.accept(&retained.start()),
                        initial.is_multiple_of(2)
                    );
                    let actual = trace(&mut retained, inputs.iter().copied());
                    assert_eq!(
                        actual,
                        sums.iter()
                            .take_while(|&&s| s <= limit)
                            .map(|&s| (s, s))
                            .collect::<Vec<_>>()
                    );
                    for (state, _) in actual {
                        assert_eq!(retained.accept(&state), state.is_multiple_of(2));
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_try_map_input() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                for factor in (0..=2).chain([rng.random(0..=1024)]) {
                    let mut limits: Vec<_> = [initial]
                        .into_iter()
                        .chain(inputs.iter().copied())
                        .flat_map(|s| [s.saturating_sub(1), s, s + 1])
                        .collect();
                    limits.sort_unstable();
                    limits.dedup();
                    for limit in limits {
                        let mut mapped =
                            transducer!(@try_map |x: usize| (x <= limit).then_some(x * factor));
                        assert_eq!(mapped.start(), ());
                        assert_eq!(
                            trace(&mut mapped, inputs.iter().copied()),
                            inputs
                                .iter()
                                .take_while(|&&x| x <= limit)
                                .map(|&x| ((), x * factor))
                                .collect::<Vec<_>>()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_with_fold() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let expected: Vec<_> = inputs
                    .iter()
                    .scan((0, initial), |(sum, acc), x| {
                        *sum += x;
                        *acc += *sum;
                        Some(((*sum, *acc), *sum))
                    })
                    .collect();
                let base = sum_transducer(0).accepting(|sum| sum.is_multiple_of(2));
                // Method syntax requires inference of the closure's argument types.
                let mut folded = FoldTransducer::new(base, initial, |acc, x| acc.add(x));
                assert_eq!(folded.start(), (0, initial));
                assert!(folded.accept(&folded.start()));
                let actual = trace(&mut folded, inputs.iter().copied());
                assert_eq!(actual, expected);
                for (state, _) in actual {
                    assert_eq!(folded.accept(&state), state.0.is_multiple_of(2));
                }
            }
        }
    }

    #[test]
    fn test_transducer_with_try_fold() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let expected: Vec<_> = inputs
                    .iter()
                    .scan((0, initial), |(sum, acc), x| {
                        *sum += x;
                        *acc += *sum;
                        Some(((*sum, *acc), *sum))
                    })
                    .collect();
                let mut limits: Vec<_> = [initial]
                    .into_iter()
                    .chain(expected.iter().map(|&((_, acc), _)| acc))
                    .flat_map(|s| [s.saturating_sub(1), s, s + 1])
                    .collect();
                limits.sort_unstable();
                limits.dedup();
                for limit in limits {
                    let base = sum_transducer(0).accepting(|sum| sum.is_multiple_of(2));
                    let mut folded = TryFoldTransducer::new(base, initial, |acc, x| {
                        let sum = acc.add(x);
                        (sum <= limit).then_some(sum)
                    });
                    assert_eq!(folded.start(), (0, initial));
                    assert!(folded.accept(&folded.start()));
                    let actual = trace(&mut folded, inputs.iter().copied());
                    assert_eq!(
                        actual,
                        expected
                            .iter()
                            .copied()
                            .take_while(|&((_, acc), _)| acc <= limit)
                            .collect::<Vec<_>>()
                    );
                    for (state, _) in actual {
                        assert_eq!(folded.accept(&state), state.0.is_multiple_of(2));
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_accepting() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            for initial in (0..=2).chain([rng.random(0..=1024)]) {
                let sums: Vec<_> = inputs
                    .iter()
                    .scan(initial, |sum, x| {
                        *sum += x;
                        Some(*sum)
                    })
                    .collect();
                let base = sum_transducer(initial).accepting(|sum: &usize| sum.is_multiple_of(2));
                let mut limits: Vec<_> = [initial]
                    .into_iter()
                    .chain(sums.iter().copied())
                    .flat_map(|s| [s.saturating_sub(1), s, s + 1])
                    .collect();
                limits.sort_unstable();
                limits.dedup();
                for limit in limits {
                    let accepting = AcceptTransducer::new(base.clone(), |state| state.ge(&limit));
                    let mut state = accepting.start();
                    assert_eq!(
                        accepting.accept(&state),
                        initial.is_multiple_of(2) && initial >= limit
                    );
                    for (&x, &sum) in inputs.iter().zip(&sums) {
                        let (next, output) = accepting.relation(&state, &x).unwrap();
                        assert_eq!((next, output), (sum, sum));
                        assert_eq!(
                            accepting.accept(&next),
                            sum.is_multiple_of(2) && sum >= limit
                        );
                        state = next;
                    }
                }
            }
        }
    }

    #[test]
    fn test_transducer_composition_rejection() {
        for outputs in exhaustive_sequences([None, Some(0usize), Some(1)], 2..=2) {
            for accepting in exhaustive_sequences([false, true], 2..=2) {
                let left = transducer!(=> || (),
                    |_: &(), _: &usize| outputs[0].map(|x| ((), x)),
                    |_: &()| accepting[0]);
                let right = transducer!(=> || (),
                    |_: &(), _: &usize| outputs[1].map(|x| ((), x)),
                    |_: &()| accepting[1]);
                let intersection = left.clone().intersection(right.clone());
                let product = left.clone().product(right.clone());
                let chain = left.chain(right);
                assert_eq!(
                    intersection.accept(&intersection.start()),
                    accepting[0] && accepting[1]
                );
                assert_eq!(
                    product.accept(&product.start()),
                    accepting[0] && accepting[1]
                );
                assert_eq!(chain.accept(&chain.start()), accepting[0] && accepting[1]);
                for input in 0..=2 {
                    let paired = outputs[0].zip(outputs[1]).map(|output| (((), ()), output));
                    assert_eq!(intersection.relation(&((), ()), &input), paired);
                    assert_eq!(product.relation(&((), ()), &(input, input)), paired);
                    assert_eq!(
                        chain.relation(&((), ()), &input),
                        outputs[0].and(outputs[1]).map(|x| (((), ()), x))
                    );
                }
            }
        }
    }

    #[test]
    fn test_transducer_iterator() {
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            let n = inputs.len();
            let mut iter = IteratorTransducer::<_, ()>::new(inputs.iter().copied());
            assert_eq!(
                trace(&mut iter, repeat_n((), n + 1)),
                inputs.iter().map(|&x| ((), x)).collect::<Vec<_>>()
            );
            assert!(trace(&mut iter, [()]).is_empty());
        }
    }

    #[test]
    fn test_transducer_monoidal() {
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            let monoidal = MonoidalTransducer::<ConcatenateOperation<usize>>::new();
            let mut state = monoidal.start();
            assert!(state.is_empty());
            assert!(monoidal.accept(&state));
            for (i, &x) in inputs.iter().enumerate() {
                let (next, ()) = monoidal.relation(&state, &vec![x]).unwrap();
                assert_eq!(next, inputs[..=i]);
                assert!(monoidal.accept(&next));
                state = next;
            }
        }
    }

    #[test]
    fn test_transducer_always_accepting() {
        let mut input_rng = Xorshift::default();
        for inputs in transducer_inputs(&mut input_rng) {
            let n = inputs.len();
            let mut always = transducer!(@<usize>);
            assert_eq!(
                trace(&mut always, inputs.iter().copied()),
                vec![((), ()); n]
            );
            let mut always = transducer!(@);
            assert_eq!(trace(&mut always, inputs), vec![((), ()); n]);
        }
    }

    #[test]
    fn test_transducer_map_fold_accept() {
        let mut rng = Xorshift::default();
        let mut cases: Vec<_> = exhaustive_sequences(0..3, 0..=4).collect();
        for _ in 0..1000 {
            let width = rng.random(0..=4);
            cases.push(rng.random_iter(0..=10).take(width).collect());
        }
        for inputs in cases {
            for n in 0..=6 {
                let mut dp = transducer!(=> || 0usize,
                |state: &usize, input: &usize| Some((state + input, state + input)),
                |state: &usize| state.is_multiple_of(2))
                .with_input()
                .counting::<usize>();
                let mut expected = vec![0usize];
                for _ in 0..n {
                    dp.step(|_| inputs.iter().copied());
                    expected = expected
                        .iter()
                        .flat_map(|&s| inputs.iter().map(move |&x| s + x))
                        .collect();
                }
                for divisor in 1..=10 {
                    let mut histogram = HashMap::new();
                    for sum in expected.iter().filter(|sum| sum.is_multiple_of(2)) {
                        *histogram.entry(sum % divisor).or_insert(0) += 1;
                    }
                    assert_eq!(
                        dp.map_fold_accept(|state| state % divisor, HashMap::new()),
                        histogram
                    );
                }
            }
        }
        let cases = exhaustive_sequences([0, 1, 2, 10], 0..=3)
            .chain(transducer_inputs(&mut rng).filter(|inputs| inputs.len() > 3));
        for inputs in cases {
            let machine = transducer!(=> || 0usize, |_, &(token, state)| Some((state, token)),
                |state| state.is_multiple_of(2));
            let mut dp = machine.dp_with::<ConcatenateOperation<usize>, _>(vec![0], || {
                VecMap::with_len(11, |&state| state)
            });
            dp.step_effect(
                |_| inputs.iter().copied().enumerate(),
                |weights, token| weights.iter().map(|_| token).collect(),
            );
            let mut tokens: Vec<_> = (0..inputs.len())
                .filter(|&i| inputs[i].is_multiple_of(2))
                .collect();
            tokens.sort_by_key(|&i| inputs[i]);
            for divisor in 1..=10 {
                let mut expected = HashMap::<usize, Vec<usize>>::new();
                for &token in &tokens {
                    expected
                        .entry(inputs[token] % divisor)
                        .or_default()
                        .push(token);
                }
                assert_eq!(
                    dp.map_fold_accept(|state| state % divisor, HashMap::new()),
                    expected
                );
            }
        }
    }

    #[test]
    fn test_transducer_step_and_run_effect() {
        type M = AdditiveOperation<usize>;
        let mut rng = Xorshift::default();
        let mut cases: Vec<_> = exhaustive_sequences(0..3, 0..=4).collect();
        for _ in 0..1000 {
            let width = rng.random(0..=4);
            cases.push(rng.random_iter(0..=10).take(width).collect());
        }
        for inputs in cases {
            for n in 0..=6 {
                let initial = rng.random(0..=10);
                let base = transducer!(=> || 0usize,
                |_: &usize, input: &usize| Some((0usize, *input)),
                |_: &usize| true);
                let mut dp = base.clone().dp::<M>(initial);
                let mut expected = initial;
                for layer in 0..n {
                    dp.step_effect(|_| inputs.iter().copied(), |value, output| value * output);
                    expected = initial * inputs.iter().sum::<usize>().pow((layer + 1) as u32);
                    assert_eq!(dp.fold_accept(), expected);
                }
                let total = base
                    .dp_with::<M, _>(initial, || VecMap::with_len(1, |&state| state))
                    .run_effect(
                        n,
                        |_| inputs.iter().copied(),
                        |value, output| value * output,
                    );
                assert_eq!(total, expected);
            }
        }
    }
}
