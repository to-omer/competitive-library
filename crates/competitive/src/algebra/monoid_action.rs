use super::{magma::*, operations::*};
use crate::num::{Bounded, One, Zero};

#[codesnip::entry("MonoidAction", include("algebra"))]
pub trait MonoidAction {
    type MT: Clone;
    type AT: Clone;
    type M: Monoid<T = Self::MT>;
    type A: Monoid<T = Self::AT>;
    fn act(x: &Self::MT, a: &Self::AT) -> Self::MT;
    #[inline]
    fn act_assign(x: &mut Self::MT, a: &Self::AT) {
        *x = Self::act(x, a);
    }
    #[inline]
    fn munit() -> Self::MT {
        <Self::M as Unital>::unit()
    }
    #[inline]
    fn aunit() -> Self::AT {
        <Self::A as Unital>::unit()
    }
    #[inline]
    fn moperate(x: &Self::MT, y: &Self::MT) -> Self::MT {
        <Self::M as Magma>::operate(x, y)
    }
    #[inline]
    fn aoperate(x: &Self::AT, y: &Self::AT) -> Self::AT {
        <Self::A as Magma>::operate(x, y)
    }
}

#[codesnip::entry(
    "monoid_action_impls",
    inline,
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
    #[codesnip::skip]
    use super::*;
    pub struct RangeSumRangeAdd<T> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<T: Copy + Zero + std::ops::Add<Output = T> + std::ops::Mul<Output = T>> MonoidAction
        for RangeSumRangeAdd<T>
    {
        type MT = (T, T);
        type AT = T;
        type M = (AdditiveOperation<T>, AdditiveOperation<T>);
        type A = AdditiveOperation<T>;
        fn act(&(x, y): &Self::MT, &a: &Self::AT) -> Self::MT {
            (x + a * y, y)
        }
    }

    pub struct RangeSumRangeLinear<T> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<T: Copy + Zero + One + std::ops::Add<Output = T> + std::ops::Mul<Output = T>> MonoidAction
        for RangeSumRangeLinear<T>
    {
        type MT = (T, T);
        type AT = (T, T);
        type M = (AdditiveOperation<T>, AdditiveOperation<T>);
        type A = LinearOperation<T>;
        fn act(&(x, y): &Self::MT, &(a, b): &Self::AT) -> Self::MT {
            (a * x + b * y, y)
        }
    }

    pub struct RangeSumRangeUpdate<T> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<T: Copy + Zero + std::ops::Add<Output = T> + std::ops::Mul<Output = T> + PartialEq>
        MonoidAction for RangeSumRangeUpdate<T>
    {
        type MT = (T, T);
        type AT = Option<T>;
        type M = (AdditiveOperation<T>, AdditiveOperation<T>);
        type A = LastOperation<T>;
        fn act(&(x, y): &Self::MT, a: &Self::AT) -> Self::MT {
            (a.unwrap_or(x) * y, y)
        }
    }

    pub struct RangeMaxRangeUpdate<T> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<T: Clone + PartialEq + Ord + Bounded> MonoidAction for RangeMaxRangeUpdate<T> {
        type MT = T;
        type AT = Option<T>;
        type M = MaxOperation<T>;
        type A = LastOperation<T>;
        fn act(x: &Self::MT, a: &Self::AT) -> Self::MT {
            a.as_ref().unwrap_or(x).clone()
        }
    }

    pub struct RangeMinRangeUpdate<T> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<T: Clone + PartialEq + Ord + Bounded> MonoidAction for RangeMinRangeUpdate<T> {
        type MT = T;
        type AT = Option<T>;
        type M = MinOperation<T>;
        type A = LastOperation<T>;
        fn act(x: &Self::MT, a: &Self::AT) -> Self::MT {
            a.as_ref().unwrap_or(x).clone()
        }
    }

    pub struct RangeMinRangeAdd<T> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }
    impl<T: Copy + Ord + Bounded + Zero + std::ops::Add<Output = T>> MonoidAction
        for RangeMinRangeAdd<T>
    {
        type MT = T;
        type AT = T;
        type M = MinOperation<T>;
        type A = AdditiveOperation<T>;
        fn act(&x: &Self::MT, &a: &Self::AT) -> Self::MT {
            x + a
        }
    }
}
