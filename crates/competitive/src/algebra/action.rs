use super::*;
use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

pub trait MonoidAct {
    type Key;
    type Act: Clone;
    type ActMonoid: Monoid<T = Self::Act>;

    fn act(x: &Self::Key, a: &Self::Act) -> Self::Key;

    fn unit() -> Self::Act {
        <Self::ActMonoid as Unital>::unit()
    }
    fn operate(x: &Self::Act, y: &Self::Act) -> Self::Act {
        <Self::ActMonoid as Magma>::operate(x, y)
    }
    fn operate_assign(x: &mut Self::Act, y: &Self::Act) {
        *x = <Self::ActMonoid as Magma>::operate(x, y);
    }
}

pub struct EmptyAct<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> MonoidAct for EmptyAct<T>
where
    T: Clone,
{
    type Key = T;
    type Act = ();
    type ActMonoid = ();

    fn act(x: &Self::Key, _a: &Self::Act) -> Self::Key {
        x.clone()
    }
}

pub struct FlattenAct<M> {
    _marker: PhantomData<fn() -> M>,
}
impl<M> MonoidAct for FlattenAct<M>
where
    M: Monoid,
{
    type Key = M::T;
    type Act = M::T;
    type ActMonoid = M;
    fn act(x: &Self::Key, a: &Self::Act) -> Self::Key {
        M::operate(x, a)
    }
}

pub struct LinearAct<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> MonoidAct for LinearAct<T>
where
    T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>,
{
    type Key = T;
    type Act = (T, T);
    type ActMonoid = LinearOperation<T>;

    fn act(x: &Self::Key, (a, b): &Self::Act) -> Self::Key {
        a.clone() * x.clone() + b.clone()
    }
}

pub struct UpdateAct<T> {
    _marker: PhantomData<fn() -> T>,
}
impl<T> MonoidAct for UpdateAct<T>
where
    T: Clone,
{
    type Key = T;
    type Act = Option<T>;
    type ActMonoid = LastOperation<T>;

    fn act(x: &Self::Key, a: &Self::Act) -> Self::Key {
        a.as_ref().unwrap_or(x).clone()
    }
}
