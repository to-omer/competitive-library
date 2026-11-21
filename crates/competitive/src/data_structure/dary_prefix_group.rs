use super::{AbelianGroup, DarySegmentGroup, DarySegmentKernel, SimdMonoid, simd_monoid::LANES};
use std::{marker::PhantomData, ops::Range};

/// A prefix-cached block layout for abelian groups in `DarySegmentTree`.
pub struct DaryPrefixGroup<M>(PhantomData<fn() -> M>);

impl<M> DarySegmentKernel for DaryPrefixGroup<M>
where
    M: SimdMonoid + AbelianGroup,
{
    const B: usize = LANES;
    type Key = M::T;
    type Output = M::T;
    type Value = M::T;
    type Change = M::T;
    type Block = [M::T; LANES];

    fn encode(key: M::T) -> M::T {
        key
    }
    fn decode(value: M::T) -> M::T {
        value
    }
    fn unit() -> M::T {
        M::unit()
    }
    fn combine(left: M::T, right: M::T) -> M::T {
        M::operate(&left, &right)
    }
    fn block(values: impl Iterator<Item = M::T>) -> Self::Block {
        let mut prefix = M::unit();
        let mut values = values.fuse();
        std::array::from_fn(|_| {
            if let Some(value) = values.next() {
                prefix = M::operate(&prefix, &value);
            }
            prefix
        })
    }
    fn get(block: &Self::Block, lane: usize) -> M::T {
        let before = lane.checked_sub(1).map_or_else(M::unit, |i| block[i]);
        M::rinv_operate(&block[lane], &before)
    }
    fn repair_value(value: M::T, change: M::T) -> M::T {
        M::operate(&value, &change)
    }
    unsafe fn fold<const ISA: u8>(block: &Self::Block, range: Range<usize>) -> M::T {
        let left = range
            .start
            .checked_sub(1)
            .map_or_else(M::unit, |i| block[i]);
        let right = range.end.checked_sub(1).map_or_else(M::unit, |i| block[i]);
        M::rinv_operate(&right, &left)
    }
    unsafe fn replace<const ISA: u8>(
        block: &mut Self::Block,
        lane: usize,
        value: M::T,
    ) -> Option<M::T> {
        let change = M::rinv_operate(&value, &Self::get(block, lane));
        unsafe { Self::update::<ISA>(block, lane, change) }
    }
    unsafe fn update<const ISA: u8>(
        block: &mut Self::Block,
        lane: usize,
        change: M::T,
    ) -> Option<M::T> {
        unsafe { M::broadcast_suffix::<ISA>(block, lane, change) };
        Some(change)
    }
    unsafe fn repair<const ISA: u8>(
        block: &mut Self::Block,
        lane: usize,
        change: M::T,
    ) -> Option<M::T> {
        unsafe { Self::update::<ISA>(block, lane, change) }
    }
}

impl<M> DarySegmentGroup for DaryPrefixGroup<M>
where
    M: SimdMonoid + AbelianGroup,
{
    fn inverse(value: M::T) -> M::T {
        M::inverse(&value)
    }
}
