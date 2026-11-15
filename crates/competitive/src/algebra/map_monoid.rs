use super::{Magma, Monoid, ReverseOperation, Unital};
use std::marker::PhantomData;

pub type MapAgg<M> = <<M as MapMonoid>::AggMonoid as Magma>::T;

pub trait MapMonoid {
    type Key;
    type AggMonoid: Monoid;
    fn single_agg(key: &Self::Key) -> MapAgg<Self>;

    #[inline]
    fn agg_unit() -> MapAgg<Self> {
        <Self::AggMonoid as Unital>::unit()
    }
    #[inline]
    fn agg_operate(x: &MapAgg<Self>, y: &MapAgg<Self>) -> MapAgg<Self> {
        <Self::AggMonoid as Magma>::operate(x, y)
    }
    #[inline]
    fn agg_operate_assign(x: &mut MapAgg<Self>, y: &MapAgg<Self>) {
        *x = Self::agg_operate(x, y);
    }
    #[inline]
    fn agg_operate_key(agg: &MapAgg<Self>, key: &Self::Key) -> MapAgg<Self> {
        Self::agg_operate(agg, &Self::single_agg(key))
    }
}

/// Reverses aggregate order while leaving single-key aggregates unchanged.
pub trait ReversibleMapMonoid: MapMonoid {
    fn toggle(agg: &mut MapAgg<Self>);
}

pub struct BidirectionalMap<M> {
    _marker: PhantomData<fn() -> M>,
}

impl<M> MapMonoid for BidirectionalMap<M>
where
    M: Monoid,
{
    type Key = M::T;
    type AggMonoid = (M, ReverseOperation<M>);
    fn single_agg(key: &Self::Key) -> MapAgg<Self> {
        (key.clone(), key.clone())
    }
    fn agg_operate_key(agg: &MapAgg<Self>, key: &Self::Key) -> MapAgg<Self> {
        (M::operate(&agg.0, key), M::operate(key, &agg.1))
    }
}

impl<M> ReversibleMapMonoid for BidirectionalMap<M>
where
    M: Monoid,
{
    fn toggle(agg: &mut MapAgg<Self>) {
        std::mem::swap(&mut agg.0, &mut agg.1);
    }
}
