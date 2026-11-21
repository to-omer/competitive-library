use super::{MonoidAct, SimdMonoid, UpdateAct, simd_monoid};

#[codesnip::entry("SimdMonoidAct")]
pub use self::monoid_action::{SimdAggregateAct, SimdMonoidAct};

#[codesnip::entry("SimdMonoidAct", include("SimdMonoid", "MonoidAct"))]
mod monoid_action {
    use super::{MonoidAct, SimdMonoid, UpdateAct};

    pub trait SimdMonoidAct: MonoidAct<Key: Copy, Act: Copy> {
        /// Applies a broadcast tag to every key. Tags compose in old-then-later order.
        ///
        /// # Safety
        /// The ISA contract is the same as for [`SimdMonoid::operate_lanes`].
        /// The slice may be unaligned or have any length.
        unsafe fn act_lanes<const ISA: u8>(keys: &mut [Self::Key], tag: Self::Act);

        /// Appends a broadcast tag to each pending tag.
        ///
        /// # Safety
        /// The ISA and slice contract is the same as for [`Self::act_lanes`].
        unsafe fn compose_lanes<const ISA: u8>(tags: &mut [Self::Act], later: Self::Act) {
            for tag in tags {
                Self::operate_assign(tag, &later);
            }
        }
    }

    /// Acting on an ordered fold must equal acting on each key, lifting, and folding.
    pub trait SimdAggregateAct<M>: SimdMonoidAct
    where
        M: SimdMonoid,
    {
        /// Enables aggregate queries to check for tags that replace the old fold.
        const HAS_CONSTANT_TAGS: bool = false;

        fn lift(key: Self::Key) -> M::T;

        fn act_aggregate(value: M::T, tag: Self::Act, len: usize) -> M::T;

        /// Returns the mapped aggregate if it depends only on the tag and length.
        fn constant_aggregate(_tag: Self::Act, _len: usize) -> Option<M::T> {
            None
        }

        /// Applies a broadcast tag to aggregates which all cover `len` keys.
        ///
        /// # Safety
        /// The ISA and slice contract is the same as for [`Self::act_lanes`].
        unsafe fn act_aggregate_lanes<const ISA: u8>(
            values: &mut [M::T],
            tag: Self::Act,
            len: usize,
        ) {
            for value in values {
                *value = Self::act_aggregate(*value, tag, len);
            }
        }
    }

    impl<T> SimdMonoidAct for UpdateAct<T>
    where
        T: Copy,
    {
        #[inline]
        unsafe fn act_lanes<const ISA: u8>(keys: &mut [T], tag: Option<T>) {
            if let Some(key) = tag {
                keys.fill(key);
            }
        }

        #[inline]
        unsafe fn compose_lanes<const ISA: u8>(tags: &mut [Option<T>], later: Option<T>) {
            if later.is_some() {
                tags.fill(later);
            }
        }
    }

    impl<M> SimdAggregateAct<M> for UpdateAct<M::T>
    where
        M: SimdMonoid,
    {
        const HAS_CONSTANT_TAGS: bool = true;

        #[inline]
        fn lift(key: M::T) -> M::T {
            key
        }

        #[inline]
        fn act_aggregate(value: M::T, tag: Option<M::T>, len: usize) -> M::T {
            <Self as SimdAggregateAct<M>>::constant_aggregate(tag, len).unwrap_or(value)
        }

        #[inline]
        fn constant_aggregate(tag: Option<M::T>, len: usize) -> Option<M::T> {
            tag.map(|key| M::pow(key, len))
        }

        #[inline]
        unsafe fn act_aggregate_lanes<const ISA: u8>(
            values: &mut [M::T],
            tag: Option<M::T>,
            len: usize,
        ) {
            if !values.is_empty()
                && let Some(repeated) = <Self as SimdAggregateAct<M>>::constant_aggregate(tag, len)
            {
                values.fill(repeated);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{SimdAggregateAct, SimdMonoid, SimdMonoidAct};
        use crate::{
            algebra::{LinearOperation, Magma, MonoidAct, UpdateAct},
            data_structure::{
                DaryDualSegmentTree, DaryLazySegmentTree, DarySegmentTree, SimdAddI64,
            },
            tools::Xorshift,
        };
        use std::num::Wrapping;

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Key(i64);

        struct Affine;

        impl MonoidAct for Affine {
            type Key = Key;
            type Act = (Wrapping<i64>, Wrapping<i64>);
            type ActMonoid = LinearOperation<Wrapping<i64>>;

            fn act(x: &Key, (a, b): &Self::Act) -> Key {
                Key((a * Wrapping(x.0) + b).0)
            }
        }

        impl SimdMonoidAct for Affine {
            unsafe fn act_lanes<const ISA: u8>(keys: &mut [Key], tag: Self::Act) {
                for key in keys {
                    Self::act_assign(key, &tag);
                }
            }
        }

        impl SimdAggregateAct<SimdAddI64> for Affine {
            fn lift(key: Key) -> Wrapping<i64> {
                Wrapping(key.0)
            }

            fn act_aggregate(value: Wrapping<i64>, (a, b): Self::Act, len: usize) -> Wrapping<i64> {
                a * value + b * Wrapping(len as i64)
            }
        }

        impl SimdMonoid for LinearOperation<Wrapping<i32>> {
            unsafe fn operate_lanes<const ISA: u8>(lhs: &mut [Self::T], rhs: &[Self::T]) {
                for (left, right) in lhs.iter_mut().zip(rhs) {
                    *left = Self::operate(left, right);
                }
            }
        }

        #[test]
        fn test_shared_point_and_aggregate_action() {
            let mut rng = Xorshift::default();
            for len in [0_usize, 1, 15, 16, 17, 31, 255, 256, 257, 273, 4097] {
                let mut keys: Vec<_> = (0..len).map(|_| Key(rng.rand64() as i64)).collect();
                let mut dual = DaryDualSegmentTree::<Affine>::from_keys(keys.iter().copied());
                let mut lazy =
                    DaryLazySegmentTree::<(SimdAddI64, Affine)>::from_keys(keys.iter().copied());
                for step in 0..384 {
                    let x = rng.rand((len + 1) as u64) as usize;
                    let y = rng.rand((len + 1) as u64) as usize;
                    let range = match step % 8 {
                        0 | 1 => 0..len,
                        2 => len.saturating_sub(1)..len,
                        3 => len / 16 * 16..len,
                        _ => x.min(y)..x.max(y),
                    };
                    let (a, b) = match step % 8 {
                        0 => (0, 5),
                        1 => (2, 1),
                        _ => (rng.rand64() as i64, rng.rand64() as i64),
                    };
                    dual.update(range.clone(), (Wrapping(a), Wrapping(b)));
                    lazy.update(range.clone(), (Wrapping(a), Wrapping(b)));
                    for key in &mut keys[range.clone()] {
                        key.0 = a.wrapping_mul(key.0).wrapping_add(b);
                    }
                    if len != 0 {
                        let i = if step % 2 == 0 {
                            len - 1
                        } else {
                            rng.rand(len as u64) as usize
                        };
                        if step % 4 <= 1 {
                            let key = Key(rng.rand64() as i64);
                            keys[i] = key;
                            dual.set(i, key);
                            lazy.set(i, key);
                        }
                        assert_eq!(dual.get(i), keys[i]);
                        assert_eq!(lazy.get(i), keys[i]);
                        for j in [0, len / 2, len - 1] {
                            assert_eq!(dual.get(j), keys[j]);
                            assert_eq!(lazy.get(j), keys[j]);
                        }
                    }
                    assert_eq!(
                        lazy.fold(range.clone()).0,
                        keys[range]
                            .iter()
                            .fold(0_i64, |acc, key| acc.wrapping_add(key.0))
                    );
                    assert_eq!(
                        lazy.fold_all().0,
                        keys.iter().fold(0_i64, |acc, key| acc.wrapping_add(key.0))
                    );
                    let query = x.min(y)..x.max(y);
                    assert_eq!(
                        lazy.fold(query.clone()).0,
                        keys[query]
                            .iter()
                            .fold(0_i64, |acc, key| acc.wrapping_add(key.0))
                    );
                }
            }
        }

        #[test]
        fn test_shared_ordered_product_and_assignment() {
            type M = LinearOperation<Wrapping<i32>>;
            let mut functions: Vec<_> = (0..257).map(|i| (Wrapping(3), Wrapping(i))).collect();
            let mut lazy =
                DaryLazySegmentTree::<(M, UpdateAct<_>)>::from_keys(functions.iter().copied());
            for (range, tag) in [(14..256, (2, 1)), (255..257, (0, -3)), (16..32, (1, 4))] {
                let tag = (Wrapping(tag.0), Wrapping(tag.1));
                lazy.update(range.clone(), Some(tag));
                functions[range].fill(tag);
                let seg = DarySegmentTree::<M>::from_keys(functions.iter().copied());
                for query in [0..0, 0..257, 11..38, 254..257, 256..257] {
                    let (a, b) = lazy.fold(query.clone());
                    assert_eq!((a, b), seg.fold(query.clone()));
                    for x in [Wrapping(0), Wrapping(1)] {
                        let expected = functions[query.clone()]
                            .iter()
                            .fold(x, |v, &(a, b)| a * v + b);
                        assert_eq!(a * x + b, expected);
                    }
                }
            }
        }
    }
}

#[codesnip::entry("_SimdMonoidActBlock", include("SimdMonoidAct"))]
pub mod simd_act_block {
    use super::SimdMonoidAct;
    use super::simd_monoid::LANES;
    use std::ops::Range;

    #[inline]
    pub fn apply_selected<T, F>(
        lanes: &mut [T],
        range: Range<usize>,
        outside: F,
        inside: F,
        mut apply: impl FnMut(&mut [T], F),
    ) where
        F: Copy,
    {
        if range.is_empty() {
            apply(lanes, outside);
        } else {
            let (before, rest) = lanes.split_at_mut(range.start);
            let (selected, after) = rest.split_at_mut(range.len());
            apply(before, outside);
            apply(selected, inside);
            apply(after, outside);
        }
    }

    pub fn leaf<T>(mut keys: impl Iterator<Item = T>) -> [T; LANES]
    where
        T: Copy,
    {
        let first = keys.next().expect("missing leaf key");
        // Only the supplied lanes participate in queries and updates.
        let mut block = [first; LANES];
        for (lane, key) in keys.enumerate() {
            block[lane + 1] = key;
        }
        block
    }

    /// # Safety
    /// ISA must satisfy the contract of `A::act_lanes`. The slice and range contain only valid keys.
    pub unsafe fn edit_keys<A, const ISA: u8>(
        keys: &mut [A::Key],
        range: Range<usize>,
        tag: A::Act,
        carry: A::Act,
    ) where
        A: SimdMonoidAct,
    {
        if range.is_empty() {
            unsafe { A::act_lanes::<ISA>(keys, carry) };
        } else {
            let combined = A::operate(&carry, &tag);
            apply_selected(keys, range, carry, combined, |keys, tag| unsafe {
                A::act_lanes::<ISA>(keys, tag);
            });
        }
    }
}
