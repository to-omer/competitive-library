use super::{
    Container, ContainerEntry, ContainerFactory, FixedVecMapFactory, HashMapFactory, Monoid,
    VecMap, VecMapFactory,
};
use std::{
    borrow::Borrow,
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    hash::Hash,
    iter::Peekable,
    marker::PhantomData,
    mem::swap,
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
    fn stepout(&mut self) {}
    fn dp<M>(self, init: M::T) -> InitTransducerDp<M, Self>
    where
        Self: Sized,
        M: Monoid,
    {
        InitTransducerDp::new(self, init)
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
    fn with_input(self) -> IntersectionTransducer<(Self, IdentityTransducer<Self::Input>)>
    where
        Self: Sized,
    {
        IntersectionTransducer((self, IdentityTransducer::new()))
    }
    fn map<U, F>(self, f: F) -> MapTransducer<Self, U, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> U,
    {
        MapTransducer::new(self, f)
    }
    fn try_map<U, F>(self, f: F) -> TryMapTransducer<Self, U, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> Option<U>,
    {
        TryMapTransducer::new(self, f)
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
        A: Clone,
        F: Fn(&A, &Self::Output) -> A,
    {
        FoldTransducer::new(self, init, f)
    }
    fn with_try_fold<A, F>(self, init: A, f: F) -> TryFoldTransducer<Self, A, F>
    where
        Self: Sized,
        A: Clone,
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

#[derive(Debug, Clone)]
pub struct InitTransducerDp<M, A>
where
    M: Monoid,
    A: Transducer,
{
    fst: A,
    init: M::T,
}

impl<M, A> InitTransducerDp<M, A>
where
    M: Monoid,
    A: Transducer,
{
    pub fn new(fst: A, init: M::T) -> Self {
        Self { fst, init }
    }
    pub fn with_factory<F>(self, factory: F) -> Transducerdp<M, A, F::Container>
    where
        F: ContainerFactory<Container: Container<Key = A::State, Value = M::T>>,
    {
        Transducerdp::new(self.fst, self.init, factory)
    }
    pub fn with_hashmap(self) -> Transducerdp<M, A, HashMap<A::State, M::T>>
    where
        A: Transducer<State: Eq + Hash>,
    {
        self.with_factory(HashMapFactory::default())
    }
    pub fn with_vecmap<F>(
        self,
        key_to_index: F,
    ) -> Transducerdp<M, A, VecMap<false, A::State, M::T, F>>
    where
        F: Fn(&A::State) -> usize + Clone,
    {
        self.with_factory(VecMapFactory::new(key_to_index))
    }
    pub fn with_fixed_vecmap<F>(
        self,
        key_to_index: F,
        len: usize,
    ) -> Transducerdp<M, A, VecMap<true, A::State, M::T, F>>
    where
        F: Fn(&A::State) -> usize + Clone,
    {
        self.with_factory(FixedVecMapFactory::new(key_to_index, len))
    }
}

#[derive(Clone)]
pub struct Transducerdp<M, T, C>
where
    M: Monoid,
    T: Transducer,
    C: Container<Key = T::State, Value = M::T>,
{
    fst: T,
    pub dp: C,
    ndp: C,
    _marker: PhantomData<fn() -> M>,
}

impl<M, T, C> Debug for Transducerdp<M, T, C>
where
    M: Monoid<T: Debug>,
    T: Transducer<State: Debug> + Debug,
    C: Container<Key = T::State, Value = M::T> + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transducerdp")
            .field("fst", &self.fst)
            .field("dp", &self.dp)
            .field("ndp", &self.ndp)
            .finish()
    }
}

impl<M, T, C> Transducerdp<M, T, C>
where
    M: Monoid,
    T: Transducer,
    C: Container<Key = T::State, Value = M::T>,
{
    pub fn new<F>(fst: T, init: M::T, factory: F) -> Self
    where
        F: ContainerFactory<Container = C>,
    {
        let mut dp = factory.create_container();
        let ndp = factory.create_container();
        dp.insert(fst.start(), init);
        Self {
            fst,
            dp,
            ndp,
            _marker: PhantomData,
        }
    }
    pub fn step<S, I, B>(&mut self, mut sigma: S)
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
    {
        for (state, value) in self.dp.drain() {
            for input in sigma() {
                if let Some((nstate, _)) = self.fst.relation(&state, input.borrow()) {
                    self.ndp
                        .entry(nstate)
                        .and_modify(|acc| M::operate_assign(acc, &value))
                        .or_insert_with(|| value.clone());
                }
            }
        }
        swap(&mut self.dp, &mut self.ndp);
        self.fst.stepout();
    }
    pub fn step_effect<S, I, B, F>(&mut self, mut sigma: S, mut effect: F)
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
        F: FnMut(&M::T, &T::Output) -> M::T,
    {
        for (state, value) in self.dp.drain() {
            for input in sigma() {
                if let Some((nstate, output)) = self.fst.relation(&state, input.borrow()) {
                    let nvalue = effect(&value, &output);
                    self.ndp
                        .entry(nstate)
                        .and_modify(|acc| M::operate_assign(acc, &nvalue))
                        .or_insert(nvalue);
                }
            }
        }
        swap(&mut self.dp, &mut self.ndp);
        self.fst.stepout();
    }
    pub fn fold_accept(&self) -> M::T {
        let mut acc = M::unit();
        for (state, value) in self.dp.iter() {
            if self.fst.accept(state) {
                M::operate_assign(&mut acc, value);
            }
        }
        acc
    }
    pub fn map_fold_accept<U, F, D>(&self, mut f: F, mut map: D) -> D
    where
        F: FnMut(&T::State) -> U,
        D: Container<Key = U, Value = M::T>,
    {
        for (state, value) in self.dp.iter() {
            if self.fst.accept(state) {
                map.entry(f(state))
                    .and_modify(|acc| M::operate_assign(acc, value))
                    .or_insert_with(|| value.clone());
            }
        }
        map
    }
    pub fn run<S, I, B>(&mut self, mut sigma: S, len: usize) -> M::T
    where
        S: FnMut() -> I,
        I: IntoIterator<Item = B>,
        B: Borrow<T::Input>,
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
        B: Borrow<T::Input>,
        F: FnMut(&M::T, &T::Output) -> M::T,
    {
        for _ in 0..len {
            self.step_effect(&mut sigma, &mut effect);
        }
        self.fold_accept()
    }
}

#[derive(Debug, Clone)]
pub struct IntersectionTransducer<Tuple>(pub Tuple);

macro_rules! impl_intersection_transducer {
    (@impl $($T:ident)*, $($a:ident)*, $($b:ident)*) => {
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
                match ($($a.relation($b, input),)*) {
                    ($(Some(($a, $b)),)*) => Some((($($a,)*), ($($b,)*))),
                    _ => None,
                }
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
                match ($($a.relation($b, $c),)*) {
                    ($(Some(($a, $b)),)*) => Some((($($a,)*), ($($b,)*))),
                    _ => None,
                }
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

#[derive(Debug, Clone)]
pub struct FunctionalTransducer<I, O, S, F, G, H>
where
    F: Fn() -> S,
    G: Fn(&S, &I) -> Option<(S, O)>,
    H: Fn(&S) -> bool,
{
    fn_start: F,
    fn_relation: G,
    fn_accept: H,
    _marker: Marker<(I, O, S)>,
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

#[derive(Debug, Clone)]
pub struct MapTransducer<T, U, F> {
    inner: T,
    f: F,
    _marker: Marker<U>,
}
impl<T, U, F> MapTransducer<T, U, F> {
    pub fn new(inner: T, f: F) -> Self {
        Self {
            inner,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, U, F> Transducer for MapTransducer<T, U, F>
where
    T: Transducer,
    F: Fn(&T::Output) -> U,
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
        Some((next_state, (self.f)(&output)))
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.inner.accept(state)
    }
    fn stepout(&mut self) {
        self.inner.stepout();
    }
}

#[derive(Debug, Clone)]
pub struct TryMapTransducer<T, U, F> {
    inner: T,
    f: F,
    _marker: Marker<U>,
}
impl<T, U, F> TryMapTransducer<T, U, F> {
    pub fn new(inner: T, f: F) -> Self {
        Self {
            inner,
            f,
            _marker: PhantomData,
        }
    }
}
impl<T, U, F> Transducer for TryMapTransducer<T, U, F>
where
    T: Transducer,
    F: Fn(&T::Output) -> Option<U>,
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
        (self.f)(&output).map(|output| (next_state, output))
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
    pub fn new(inner: T, pred: F) -> Self {
        Self { inner, pred }
    }
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
    pub fn new(inner: T, init: A, f: F) -> Self {
        Self { inner, init, f }
    }
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
    pub fn new(inner: T, init: A, f: F) -> Self {
        Self { inner, init, f }
    }
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
    pub fn new(inner: T, pred: F) -> Self {
        Self { inner, pred }
    }
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

#[derive(Debug, Clone)]
pub struct EqualTransducer<T>(PhantomData<fn() -> T>);
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

#[derive(Debug, Clone)]
/// DFA to accept Less/Greater than (or equal to) in lexicographical order
pub struct LexicographicalTransducer<T> {
    ordering: Ordering,
    equal: bool,
    _marker: PhantomData<fn() -> T>,
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
        match (state, input.1.cmp(&input.0)) {
            (true, Ordering::Equal) => Some((true, ())),
            (true, ord) if ord == self.ordering => None,
            _ => Some((false, ())),
        }
    }
    fn accept(&self, state: &Self::State) -> bool {
        self.equal || !state
    }
}

#[derive(Debug, Clone)]
/// DFA to accept Less/Greater than (or equal to) in reversed lexicographical order
pub struct RevLexicographicalTransducer<T> {
    ordering: Ordering,
    equal: bool,
    _marker: PhantomData<fn() -> T>,
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

#[derive(Debug, Clone)]
pub struct SequenceTransducer<'a, T, A> {
    sequence: &'a [T],
    _marker: PhantomData<fn() -> A>,
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

#[derive(Debug, Clone)]
pub struct RevSequenceTransducer<'a, T, A> {
    sequence: &'a [T],
    _marker: PhantomData<fn() -> A>,
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

#[derive(Debug, Clone)]
pub struct MonoidalTransducer<M>(PhantomData<fn() -> M>)
where
    M: Monoid;
impl<M> MonoidalTransducer<M>
where
    M: Monoid,
{
    pub fn new() -> Self {
        Default::default()
    }
}
impl<M> Default for MonoidalTransducer<M>
where
    M: Monoid,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}
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

#[derive(Debug, Clone)]
pub struct IdentityTransducer<I>(PhantomData<fn() -> I>);
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

#[derive(Debug, Clone)]
pub struct AlwaysAcceptingTransducer<A>(PhantomData<fn() -> A>);
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
/// - `<= seq`, `seq >=`: [`LexicographicalTransducer::less_than_or_equal()`](`LexicographicalTransducer::less_than_or_equal`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `>= seq`, `seq <=`: [`LexicographicalTransducer::greater_than_or_equal()`](`LexicographicalTransducer::greater_than_or_equal`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `< seq`, `seq >`: [`LexicographicalTransducer::less_than()`](`LexicographicalTransducer::less_than`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `> seq`, `seq <`: [`LexicographicalTransducer::greater_than()`](`LexicographicalTransducer::greater_than`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `!<= seq`, `seq !>=`: [`RevLexicographicalTransducer::less_than_or_equal()`](`RevLexicographicalTransducer::less_than_or_equal`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `!>= seq`, `seq !<=`: [`RevLexicographicalTransducer::greater_than_or_equal()`](`RevLexicographicalTransducer::greater_than_or_equal`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `!< seq`, `seq !>`: [`RevLexicographicalTransducer::less_than()`](`RevLexicographicalTransducer::less_than`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `!> seq`, `seq !<`: [`RevLexicographicalTransducer::greater_than()`](`RevLexicographicalTransducer::greater_than`) with [`SequenceTransducer`](`SequenceTransducer`)
/// - `<=`: [`LexicographicalTransducer::less_than_or_equal()`](`LexicographicalTransducer::less_than_or_equal`)
/// - `>=`: [`LexicographicalTransducer::greater_than_or_equal()`](`LexicographicalTransducer::greater_than_or_equal`)
/// - `<`: [`LexicographicalTransducer::less_than()`](`LexicographicalTransducer::less_than`)
/// - `>`: [`LexicographicalTransducer::greater_than()`](`LexicographicalTransducer::greater_than`)
/// - `!<=`: [`RevLexicographicalTransducer::less_than_or_equal()`](`RevLexicographicalTransducer::less_than_or_equal`)
/// - `!>=`: [`RevLexicographicalTransducer::greater_than_or_equal()`](`RevLexicographicalTransducer::greater_than_or_equal`)
/// - `!<`: [`RevLexicographicalTransducer::less_than()`](`RevLexicographicalTransducer::less_than`)
/// - `!>`: [`RevLexicographicalTransducer::greater_than()`](`RevLexicographicalTransducer::greater_than`)
/// - `=> f g h`: [`FunctionalTransducer::new(f, g, h)`](`FunctionalTransducer`)
/// - `@id`: [`IdentityTransducer::new()`](`IdentityTransducer`)
/// - `@it e`: [`IteratorTransducer::new(e)`](`IteratorTransducer`)
/// - `@map f`: `@id |> map(f)`
/// - `@try_map f`: `@id |> try_map(f)`
/// - `@seq e`: [`SequenceTransducer::new(e)`](`SequenceTransducer`)
/// - `@rseq e`: [`RevSequenceTransducer::new(e)`](`RevSequenceTransducer`)
/// - `@`: [`AlwaysAcceptingTransducer::new()`](`AlwaysAcceptingTransducer`)
/// - `A |> method(args...)`: `Transducer::method(A, args...)`
/// - `A |> map(f)`: `Transducer::map(A, f)`
/// - `A |> try_map(f)`: `Transducer::try_map(A, f)`
/// - `A |> retain(f)`: `Transducer::retain(A, f)`
/// - `A |> with_fold(init, f)`: `Transducer::with_fold(A, init, f)`
/// - `A |> with_try_fold(init, f)`: `Transducer::with_try_fold(A, init, f)`
/// - `A |> accepting(f)`: `Transducer::accepting(A, f)`
/// - `A . B`: [`ChainTransducer((A, B))`](`ChainTransducer`)
/// - `A * B`: [`ProductTransducer((A, B))`](`ProductTransducer`)
/// - `A & B`: [`IntersectionTransducer((A, B))`](`IntersectionTransducer`)
#[macro_export]
macro_rules! transducer {
    (@check $e:expr)                                         => {{ #[inline(always)] fn check_transucer<T>(fst: T) -> T where T: Transducer { fst } check_transucer($e) }};
    (@inner ($($t:tt)*))                                     => { $crate::transducer!(@inner $($t)*) };
    (@inner => $f:expr, $g:expr, $h:expr $(,)?)              => { $crate::transducer!(@check FunctionalTransducer::new($f, $g, $h)) };
    (@inner = $e:expr)                                       => { $crate::transducer!(((@id & (@seq &$e)) . =)) };
    (@inner <= $e:expr)                                      => { $crate::transducer!(((@id & (@seq &$e)) . <=)) };
    (@inner >= $e:expr)                                      => { $crate::transducer!(((@id & (@seq &$e)) . >=)) };
    (@inner < $e:expr)                                       => { $crate::transducer!(((@id & (@seq &$e)) . <)) };
    (@inner > $e:expr)                                       => { $crate::transducer!(((@id & (@seq &$e)) . >)) };
    (@inner !<= $e:expr)                                     => { $crate::transducer!(((@id & (@rseq &$e)) . !<=)) };
    (@inner !>= $e:expr)                                     => { $crate::transducer!(((@id & (@rseq &$e)) . !>=)) };
    (@inner !< $e:expr)                                      => { $crate::transducer!(((@id & (@rseq &$e)) . !<)) };
    (@inner !> $e:expr)                                      => { $crate::transducer!(((@id & (@rseq &$e)) . !>)) };
    (@inner $e:ident =)                                      => { $crate::transducer!((((@seq &$e) & @id) . =)) };
    (@inner $e:ident <=)                                     => { $crate::transducer!((((@seq &$e) & @id) . <=)) };
    (@inner $e:ident >=)                                     => { $crate::transducer!((((@seq &$e) & @id) . >=)) };
    (@inner $e:ident <)                                      => { $crate::transducer!((((@seq &$e) & @id) . <)) };
    (@inner $e:ident >)                                      => { $crate::transducer!((((@seq &$e) & @id) . >)) };
    (@inner $e:ident !<=)                                    => { $crate::transducer!((((@rseq &$e) & @id) . !<=)) };
    (@inner $e:ident !>=)                                    => { $crate::transducer!((((@rseq &$e) & @id) . !>=)) };
    (@inner $e:ident !<)                                     => { $crate::transducer!((((@rseq &$e) & @id) . !<)) };
    (@inner $e:ident !>)                                     => { $crate::transducer!((((@rseq &$e) & @id) . !>)) };
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
        algebra::AdditiveOperation,
        tools::{NotEmptySegment, ToDigitSequence, Xorshift},
    };
    use std::collections::HashMap;

    #[test]
    fn test_equal_transducer() {
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
                n,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & =)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
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
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
            );
            assert_eq!(
                n * (n + 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & >=)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & <)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll <=) & (< rr)) * ((ll <=) & (< rr))) & >)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
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
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
            );
            assert_eq!(
                n * (n + 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !>=)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !<)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
            );
            assert_eq!(
                n * (n - 1) / 2,
                transducer!((((ll !<=) & (!< rr)) * ((ll !<=) & (!< rr))) & !>)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(
                        || (0..radix * radix).map(|x| (x / radix, x % radix)),
                        ll.len()
                    )
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
            assert_eq!(
                n + 1,
                transducer!(<= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                n,
                transducer!(< nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n,
                transducer!(>= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                transducer!(> nd)
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
        for (n, r) in rng.random_iter((0..10usize.pow(18), 2..=10)).take(Q) {
            let nd = n.to_digit_sequence_radix(r);
            assert_eq!(
                n + 1,
                transducer!(!<= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                n,
                transducer!(!< nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n,
                transducer!(!>= nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
            );
            assert_eq!(
                r.pow(nd.len() as _) - n - 1,
                transducer!(!> nd)
                    .dp::<A>(1)
                    .with_hashmap()
                    .run(|| 0..r, nd.len())
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
                fst.clone().dp::<A>(1).with_hashmap().run(|| 0..r, nd.len())
            );

            assert_eq!(
                n.div_ceil(c),
                fst.dp::<A>(1)
                    .with_vecmap(|&((_, s0), s1): &((((), ()), bool), usize)| s1 * 2 + s0 as usize)
                    .run(|| 0..r, nd.len())
            );
        }
    }

    #[test]
    fn test_add_lte() {
        type A = AdditiveOperation<usize>;
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
                            |> map(|&(a, (x, _y))| x + a)
                            . (=> || 0usize, |s, i| Some(((s + i) / 2, (s + i) % 2)), |s| *s == 0)
                        ) & (@id |> map(|&(_x, y)| y))
                    ) . (!<=)
                )
            );

            let result = fst
                .dp::<A>(1)
                .with_hashmap()
                .run(|| (0usize..4).map(|bit| (bit & 1, (bit >> 1) & 1)), 20);
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

    #[test]
    fn test_transducer_with_input() {
        let mut with_input = transducer!(
            (=> || 0usize,
                |state: &usize, input: &usize| Some((state + *input, state + *input)),
                |_: &usize| true)
            |> with_input()
        );
        let start = with_input.start();
        assert_eq!(start, (0usize, ()));
        let log = trace(&mut with_input, [1usize]);
        assert_eq!(log, vec![((1usize, ()), (1usize, 1usize))]);
    }

    #[test]
    fn test_transducer_intersection() {
        let mut intersection = transducer!(=> || 0usize,
            |state: &usize, input: &usize| Some((state + *input, state + *input)),
            |_: &usize| true)
        .intersection(IdentityTransducer::<usize>::new());
        let start = intersection.start();
        assert_eq!(start, (0usize, ()));
        let log = trace(&mut intersection, [2usize]);
        assert_eq!(log, vec![((2usize, ()), (2usize, 2usize))]);
    }

    #[test]
    fn test_transducer_product() {
        let mut product = transducer!(=> || 0usize,
            |state: &usize, input: &usize| Some((state + *input, state + *input)),
            |_: &usize| true)
        .product(AlwaysAcceptingTransducer::<usize>::new());
        let start = product.start();
        assert_eq!(start, (0usize, ()));
        let log = trace(&mut product, [(2usize, 7usize)]);
        assert_eq!(log, vec![((2usize, ()), (2usize, ()))]);
    }

    #[test]
    fn test_transducer_chain() {
        let mut chain = transducer!(=> || 0usize,
            |state: &usize, input: &usize| Some((state + *input, state + *input)),
            |_: &usize| true)
        .chain(IdentityTransducer::<usize>::new());
        let start = chain.start();
        assert_eq!(start, (0usize, ()));
        let log = trace(&mut chain, [2usize]);
        assert_eq!(log, vec![((2usize, ()), 2usize)]);

        let mut chain = transducer!(=> || 0usize,
            |state: &usize, input: &usize| Some((state + *input, state + *input)),
            |_: &usize| true)
        .chain(transducer!(@map |x: &usize| *x + 1usize));
        assert_eq!(chain.start(), (0usize, ()));
        assert_eq!(trace(&mut chain, [2usize]), vec![((2usize, ()), 3usize)]);
    }

    #[test]
    fn test_transducer_map() {
        let mut mapped = transducer!(
            (=> || 0usize,
                |state: &usize, input: &usize| Some((state + *input, state + *input)),
                |_: &usize| true)
            |> map(|output: &usize| *output * 2usize)
        );
        let start = mapped.start();
        assert_eq!(start, 0usize);
        let log = trace(&mut mapped, [3usize]);
        assert_eq!(log, vec![(3usize, 6usize)]);

        let mut mapped = transducer!(@map |input: &usize| *input + 10);
        assert_eq!(mapped.start(), ());
        assert_eq!(trace(&mut mapped, [1usize]), vec![((), 11usize)]);
    }

    #[test]
    fn test_transducer_try_map() {
        let mut filtered = transducer!(
            (=> || 0usize,
                |state: &usize, input: &usize| Some((state + *input, state + *input)),
                |_: &usize| true)
            |> try_map(|output: &usize| {
                if (*output).is_multiple_of(2) {
                    Some(*output / 2)
                } else {
                    None
                }
            })
        );
        assert_eq!(filtered.start(), 0usize);
        assert!(trace(&mut filtered, [1usize]).is_empty());
        assert_eq!(trace(&mut filtered, [2usize]), vec![(2usize, 1usize)]);

        let mut mapped = transducer!(@try_map |input: &usize| {
            (*input != 0).then_some(*input + 10)
        });
        assert_eq!(mapped.start(), ());
        assert!(trace(&mut mapped, [0usize]).is_empty());
        assert_eq!(trace(&mut mapped, [1usize]), vec![((), 11usize)]);
    }

    #[test]
    fn test_transducer_retain() {
        let mut retained = transducer!(
            (=> || 0usize,
                |state: &usize, input: &usize| Some((state + 1, *input)),
                |_: &usize| true)
            |> retain(|output: &usize| output.is_multiple_of(2))
        );
        assert!(trace(&mut retained, [1usize]).is_empty());
        assert_eq!(trace(&mut retained, [2usize]), vec![(1usize, 2usize)]);
    }

    #[test]
    fn test_transducer_with_fold() {
        let mut folded = transducer!(
            @id |> with_fold(0usize, |acc: &usize, output: &usize| *acc + *output)
        );
        assert_eq!(folded.start(), ((), 0usize));
        assert_eq!(
            trace(&mut folded, [2usize, 3usize]),
            vec![(((), 2usize), 2usize), (((), 5usize), 3usize)]
        );
    }

    #[test]
    fn test_transducer_with_try_fold() {
        let mut folded = transducer!(@id |> with_try_fold(
            0usize,
            |acc: &usize, output: &usize| {
                let next = *acc + *output;
                (next <= 4).then_some(next)
            }
        ));
        assert_eq!(
            trace(&mut folded, [2usize, 1usize]),
            vec![(((), 2usize), 2usize), (((), 3usize), 1usize)]
        );
        assert_eq!(
            trace(&mut folded, [2usize, 3usize]),
            vec![(((), 2usize), 2usize)]
        );
    }

    #[test]
    fn test_transducer_accepting() {
        let accepting = transducer!(
            (=> || 0usize,
                |state: &usize, input: &usize| Some((state + *input, state + *input)),
                |_: &usize| true)
            |> accepting(|state: &usize| *state >= 3)
        );

        assert!(!accepting.accept(&0usize));

        let (state, output) = accepting.relation(&0usize, &1usize).unwrap();
        assert_eq!((state, output), (1usize, 1usize));
        assert!(!accepting.accept(&state));

        let (state, output) = accepting.relation(&state, &2usize).unwrap();
        assert_eq!((state, output), (3usize, 3usize));
        assert!(accepting.accept(&state));
    }

    #[test]
    fn test_transducer_map_fold_accept() {
        type M = AdditiveOperation<usize>;
        let mut dp = transducer!(=> || 0usize,
            |state: &usize, input: &usize| Some((state + *input, state + *input)),
            |_: &usize| true)
        .with_input()
        .dp::<M>(1usize)
        .with_hashmap();
        dp.step(|| vec![1usize, 3usize].into_iter());
        let mut histogram = dp.map_fold_accept(|&(state, _)| state % 2, HashMap::new());
        assert_eq!(histogram.remove(&1usize), Some(2usize));
        assert!(histogram.is_empty());
    }

    #[test]
    fn test_transducer_step_effect() {
        type M = AdditiveOperation<usize>;
        let mut dp = transducer!(=> || 0usize,
            |_: &usize, input: &usize| Some((0usize, *input)),
            |_: &usize| true)
        .dp::<M>(1usize)
        .with_hashmap();
        dp.step_effect(
            || vec![1usize, 2usize].into_iter(),
            |value, output| value + output,
        );
        assert_eq!(dp.fold_accept(), 5usize);
    }

    #[test]
    fn test_transducer_run_effect() {
        type M = AdditiveOperation<usize>;
        let mut dp = transducer!(=> || 0usize,
            |_: &usize, input: &usize| Some((0usize, *input)),
            |_: &usize| true)
        .dp::<M>(1usize)
        .with_fixed_vecmap(|state: &usize| *state, 1);
        let total = dp.run_effect(
            || std::iter::once(1usize),
            3,
            |value, output| value + output,
        );
        assert_eq!(total, 4usize);
    }

    #[test]
    fn test_iterator_transducer() {
        let mut iter = IteratorTransducer::<_, ()>::new(vec![10usize, 20usize].into_iter());
        assert_eq!(
            trace(&mut iter, [(), ()]),
            vec![((), 10usize), ((), 20usize)]
        );
        assert!(trace(&mut iter, [()]).is_empty());
    }

    #[test]
    fn test_monoidal_transducer() {
        type M = AdditiveOperation<usize>;

        let mut monoidal = MonoidalTransducer::<M>::new();
        assert_eq!(
            trace(&mut monoidal, [1usize, 2usize, 3usize]),
            vec![(1usize, ()), (3usize, ()), (6usize, ())]
        );
        assert!(monoidal.accept(&0usize));
    }

    #[test]
    fn test_always_accepting_transducer() {
        let mut always_generic = transducer!(@<usize>);
        let mut always_inferred = transducer!(@);
        assert_eq!(trace(&mut always_generic, [5usize]), vec![((), ())]);
        assert_eq!(trace(&mut always_inferred, ["anything"]), vec![((), ())]);
    }
}
