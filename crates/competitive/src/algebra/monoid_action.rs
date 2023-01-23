#![allow(clippy::manual_clamp)]

use super::{magma::*, operations::*};
use crate::num::{Bounded, One, Zero};

#[codesnip::entry("MonoidAction", include("algebra"))]
pub trait MonoidAction {
    type Key;
    type Agg: Clone;
    type Act: Clone;
    type AggMonoid: Monoid<T = Self::Agg>;
    type ActMonoid: Monoid<T = Self::Act>;
    fn single_agg(key: &Self::Key) -> Self::Agg;
    fn act_key(x: &Self::Key, a: &Self::Act) -> Self::Key;
    fn act_agg(x: &Self::Agg, a: &Self::Act) -> Option<Self::Agg>;
    fn toggle(_x: &mut Self::Agg) {}

    #[inline]
    fn agg_unit() -> Self::Agg {
        <Self::AggMonoid as Unital>::unit()
    }
    #[inline]
    fn act_unit() -> Self::Act {
        <Self::ActMonoid as Unital>::unit()
    }
    #[inline]
    fn agg_operate(x: &Self::Agg, y: &Self::Agg) -> Self::Agg {
        <Self::AggMonoid as Magma>::operate(x, y)
    }
    #[inline]
    fn act_operate(x: &Self::Act, y: &Self::Act) -> Self::Act {
        <Self::ActMonoid as Magma>::operate(x, y)
    }
    #[inline]
    fn agg_operate_assign(x: &mut Self::Agg, y: &Self::Agg) {
        *x = <Self::AggMonoid as Magma>::operate(x, y);
    }
    #[inline]
    fn act_operate_assign(x: &mut Self::Act, y: &Self::Act) {
        *x = <Self::ActMonoid as Magma>::operate(x, y);
    }
}

#[codesnip::entry("monoid_action_impls")]
pub use self::monoid_action_impls::*;

#[codesnip::entry(
    "monoid_action_impls",
    include(
        "MonoidAction",
        "AdditiveOperation",
        "TupleOperation",
        "LastOperation",
        "LinearOperation",
        "MaxOperation",
        "MinOperation",
        "bounded",
        "zero_one"
    )
)]
pub mod monoid_action_impls {
    use super::*;
    use std::{
        cmp::Ordering,
        marker::PhantomData,
        ops::{Add, Mul, Sub},
    };
    pub struct EmptyLazy<M> {
        _marker: PhantomData<fn() -> M>,
    }
    impl<M> MonoidAction for EmptyLazy<M>
    where
        M: Monoid,
    {
        type Key = M::T;
        type Agg = M::T;
        type Act = ();
        type AggMonoid = M;
        type ActMonoid = ();
        fn single_agg(key: &Self::Key) -> Self::Agg {
            key.clone()
        }
        fn act_key(x: &Self::Key, _a: &Self::Act) -> Self::Key {
            x.clone()
        }
        fn act_agg(x: &Self::Agg, _a: &Self::Act) -> Option<Self::Agg> {
            Some(x.clone())
        }
    }
    pub struct EmptyAction<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for EmptyAction<T>
    where
        T: Clone,
    {
        type Key = T;
        type Agg = ();
        type Act = ();
        type AggMonoid = ();
        type ActMonoid = ();
        fn single_agg(_key: &Self::Key) -> Self::Agg {}
        fn act_key(x: &Self::Key, _a: &Self::Act) -> Self::Key {
            x.clone()
        }
        fn act_agg(_x: &Self::Agg, _a: &Self::Act) -> Option<Self::Agg> {
            Some(())
        }
    }

    pub struct RangeSumRangeAdd<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for RangeSumRangeAdd<T>
    where
        T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
    {
        type Key = T;
        type Agg = (T, T);
        type Act = T;
        type AggMonoid = (AdditiveOperation<T>, AdditiveOperation<T>);
        type ActMonoid = AdditiveOperation<T>;
        fn single_agg(key: &Self::Key) -> Self::Agg {
            (*key, T::one())
        }
        fn act_key(&x: &Self::Key, &a: &Self::Act) -> Self::Key {
            if <Self::ActMonoid as Unital>::is_unit(&a) {
                x
            } else {
                x + a
            }
        }
        fn act_agg(&(x, y): &Self::Agg, &a: &Self::Act) -> Option<Self::Agg> {
            Some(if <Self::ActMonoid as Unital>::is_unit(&a) {
                (x, y)
            } else {
                (x + a * y, y)
            })
        }
    }

    pub struct RangeSumRangeLinear<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for RangeSumRangeLinear<T>
    where
        T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
    {
        type Key = T;
        type Agg = (T, T);
        type Act = (T, T);
        type AggMonoid = (AdditiveOperation<T>, AdditiveOperation<T>);
        type ActMonoid = LinearOperation<T>;
        fn single_agg(key: &Self::Key) -> Self::Agg {
            (*key, T::one())
        }
        fn act_key(&x: &Self::Key, &(a, b): &Self::Act) -> Self::Key {
            if <Self::ActMonoid as Unital>::is_unit(&(a, b)) {
                x
            } else {
                a * x + b
            }
        }
        fn act_agg(&(x, y): &Self::Agg, &(a, b): &Self::Act) -> Option<Self::Agg> {
            Some(if <Self::ActMonoid as Unital>::is_unit(&(a, b)) {
                (x, y)
            } else {
                (a * x + b * y, y)
            })
        }
    }

    pub struct RangeSumRangeUpdate<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for RangeSumRangeUpdate<T>
    where
        T: Copy + Zero + One + Add<Output = T> + Mul<Output = T> + PartialEq,
    {
        type Key = T;
        type Agg = (T, T);
        type Act = Option<T>;
        type AggMonoid = (AdditiveOperation<T>, AdditiveOperation<T>);
        type ActMonoid = LastOperation<T>;
        fn single_agg(key: &Self::Key) -> Self::Agg {
            (*key, T::one())
        }
        fn act_key(&x: &Self::Key, &a: &Self::Act) -> Self::Key {
            a.unwrap_or(x)
        }
        fn act_agg(&(x, y): &Self::Agg, a: &Self::Act) -> Option<Self::Agg> {
            Some((a.map(|a| a * y).unwrap_or(x), y))
        }
    }

    pub struct RangeMaxRangeUpdate<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for RangeMaxRangeUpdate<T>
    where
        T: Clone + PartialEq + Ord + Bounded,
    {
        type Key = T;
        type Agg = T;
        type Act = Option<T>;
        type AggMonoid = MaxOperation<T>;
        type ActMonoid = LastOperation<T>;
        fn single_agg(key: &Self::Key) -> Self::Agg {
            key.clone()
        }
        fn act_key(x: &Self::Key, a: &Self::Act) -> Self::Key {
            a.as_ref().unwrap_or(x).clone()
        }
        fn act_agg(x: &Self::Agg, a: &Self::Act) -> Option<Self::Agg> {
            Some(a.as_ref().unwrap_or(x).clone())
        }
    }

    pub struct RangeMinRangeUpdate<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for RangeMinRangeUpdate<T>
    where
        T: Clone + PartialEq + Ord + Bounded,
    {
        type Key = T;
        type Agg = T;
        type Act = Option<T>;
        type AggMonoid = MinOperation<T>;
        type ActMonoid = LastOperation<T>;
        fn single_agg(key: &Self::Key) -> Self::Agg {
            key.clone()
        }
        fn act_key(x: &Self::Key, a: &Self::Act) -> Self::Key {
            a.as_ref().unwrap_or(x).clone()
        }
        fn act_agg(x: &Self::Agg, a: &Self::Act) -> Option<Self::Agg> {
            Some(a.as_ref().unwrap_or(x).clone())
        }
    }

    pub struct RangeMinRangeAdd<T> {
        _marker: PhantomData<fn() -> T>,
    }
    impl<T> MonoidAction for RangeMinRangeAdd<T>
    where
        T: Copy + Ord + Bounded + Zero + Add<Output = T>,
    {
        type Key = T;
        type Agg = T;
        type Act = T;
        type AggMonoid = MinOperation<T>;
        type ActMonoid = AdditiveOperation<T>;
        fn single_agg(key: &Self::Key) -> Self::Agg {
            *key
        }
        fn act_key(&x: &Self::Key, &a: &Self::Act) -> Self::Key {
            if <Self::ActMonoid as Unital>::is_unit(&a) {
                x
            } else {
                x + a
            }
        }
        fn act_agg(&x: &Self::Agg, &a: &Self::Act) -> Option<Self::Agg> {
            Some(if <Self::ActMonoid as Unital>::is_unit(&a) {
                x
            } else {
                x + a
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RangeChminChmaxAdd<T> {
        lb: T,
        ub: T,
        bias: T,
    }
    impl<T> RangeChminChmaxAdd<T>
    where
        T: Zero + Bounded,
    {
        pub fn chmin(x: T) -> Self {
            Self {
                lb: T::minimum(),
                ub: x,
                bias: T::zero(),
            }
        }
        pub fn chmax(x: T) -> Self {
            Self {
                lb: x,
                ub: T::maximum(),
                bias: T::zero(),
            }
        }
        pub fn add(x: T) -> Self {
            Self {
                lb: T::minimum(),
                ub: T::maximum(),
                bias: x,
            }
        }
    }
    impl<T> Magma for RangeChminChmaxAdd<T>
    where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq,
    {
        type T = Self;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            Self {
                lb: (x.lb + x.bias).min(y.ub).max(y.lb) - x.bias,
                ub: (x.ub + x.bias).max(y.lb).min(y.ub) - x.bias,
                bias: x.bias + y.bias,
            }
        }
    }
    impl<T> Associative for RangeChminChmaxAdd<T> where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq
    {
    }
    impl<T> Unital for RangeChminChmaxAdd<T>
    where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq,
    {
        fn unit() -> Self::T {
            Self {
                lb: T::minimum(),
                ub: T::maximum(),
                bias: T::zero(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RangeSumRangeChminChmaxAdd<T> {
        min: T,
        max: T,
        min2: T,
        max2: T,
        pub sum: T,
        size: T,
        n_min: T,
        n_max: T,
    }

    impl<T> RangeSumRangeChminChmaxAdd<T>
    where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq,
    {
        pub fn single(key: T, size: T) -> Self {
            Self {
                min: key,
                max: key,
                min2: T::maximum(),
                max2: T::minimum(),
                sum: key * size,
                size,
                n_min: size,
                n_max: size,
            }
        }
    }
    impl<T> Magma for RangeSumRangeChminChmaxAdd<T>
    where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq,
    {
        type T = Self;
        fn operate(x: &Self::T, y: &Self::T) -> Self::T {
            Self {
                min: x.min.min(y.min),
                max: x.max.max(y.max),
                min2: if x.min == y.min {
                    x.min2.min(y.min2)
                } else if x.min2 <= y.min {
                    x.min2
                } else if y.min2 <= x.min {
                    y.min2
                } else {
                    x.min.max(y.min)
                },
                max2: if x.max == y.max {
                    x.max2.max(y.max2)
                } else if x.max2 >= y.max {
                    x.max2
                } else if y.max2 >= x.max {
                    y.max2
                } else {
                    x.max.min(y.max)
                },
                sum: x.sum + y.sum,
                size: x.size + y.size,
                n_min: match x.min.cmp(&y.min) {
                    Ordering::Less => x.n_min,
                    Ordering::Equal => x.n_min + y.n_min,
                    Ordering::Greater => y.n_min,
                },
                n_max: match x.max.cmp(&y.max) {
                    Ordering::Less => y.n_max,
                    Ordering::Equal => x.n_max + y.n_max,
                    Ordering::Greater => x.n_max,
                },
            }
        }
    }
    impl<T> Associative for RangeSumRangeChminChmaxAdd<T> where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq
    {
    }
    impl<T> Unital for RangeSumRangeChminChmaxAdd<T>
    where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq,
    {
        fn unit() -> Self::T {
            Self {
                min: T::maximum(),
                max: T::minimum(),
                min2: T::maximum(),
                max2: T::minimum(),
                sum: T::zero(),
                size: T::zero(),
                n_min: T::zero(),
                n_max: T::zero(),
            }
        }
    }

    impl<T> MonoidAction for RangeSumRangeChminChmaxAdd<T>
    where
        T: Copy
            + Zero
            + One
            + Ord
            + Bounded
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + PartialEq,
    {
        type Key = T;
        type Agg = Self;
        type Act = RangeChminChmaxAdd<T>;
        type AggMonoid = Self;
        type ActMonoid = RangeChminChmaxAdd<T>;
        fn single_agg(&key: &Self::Key) -> Self::Agg {
            Self::single(key, T::one())
        }
        fn act_key(&x: &Self::Key, a: &Self::Act) -> Self::Key {
            if <Self::ActMonoid as Unital>::is_unit(a) {
                x
            } else {
                x.max(a.lb).min(a.ub) + a.bias
            }
        }
        fn act_agg(x: &Self::Agg, a: &Self::Act) -> Option<Self::Agg> {
            Some(if <Self::ActMonoid as Unital>::is_unit(a) {
                x.clone()
            } else if x.size.is_zero() {
                Self::unit()
            } else if x.min == x.max || a.lb == a.ub || a.lb >= x.max || a.ub <= x.min {
                Self::single(x.min.max(a.lb).min(a.ub) + a.bias, x.size)
            } else if x.min2 == x.max {
                let mut x = x.clone();
                let min = x.min.max(a.lb) + a.bias;
                let max = x.max.min(a.ub) + a.bias;
                x.min = min;
                x.max2 = min;
                x.max = max;
                x.min2 = max;
                x.sum = min * x.n_min + max * x.n_max;
                x
            } else if a.lb < x.min2 && x.max2 < a.ub {
                let mut x = x.clone();
                let min = x.min.max(a.lb);
                let max = x.max.min(a.ub);
                x.sum = x.sum + (min - x.min) * x.n_min + (max - x.max) * x.n_max + a.bias * x.size;
                x.min = min + a.bias;
                x.max = max + a.bias;
                x.min2 = x.min2 + a.bias;
                x.max2 = x.max2 + a.bias;
                x
            } else {
                return None;
            })
        }
    }
}
