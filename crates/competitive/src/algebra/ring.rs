use super::*;
use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

pub trait SemiRing {
    type T: Clone;
    type Additive: AbelianMonoid<T = Self::T>;
    type Multiplicative: Monoid<T = Self::T>;
    /// additive identity: $0$
    fn zero() -> Self::T {
        <Self::Additive as Unital>::unit()
    }
    /// checks if the element is zero
    fn is_zero(x: &Self::T) -> bool
    where
        Self::T: PartialEq,
    {
        *x == Self::zero()
    }
    /// multiplicative identity: $1$
    fn one() -> Self::T {
        <Self::Multiplicative as Unital>::unit()
    }
    /// checks if the element is one
    fn is_one(x: &Self::T) -> bool
    where
        Self::T: PartialEq,
    {
        *x == Self::one()
    }
    /// additive operaion: $+$
    fn add(x: &Self::T, y: &Self::T) -> Self::T {
        <Self::Additive as Magma>::operate(x, y)
    }
    /// multiplicative operaion: $+$
    fn mul(x: &Self::T, y: &Self::T) -> Self::T {
        <Self::Multiplicative as Magma>::operate(x, y)
    }

    fn add_assign(x: &mut Self::T, y: &Self::T) {
        <Self::Additive as Magma>::operate_assign(x, y);
    }

    fn mul_assign(x: &mut Self::T, y: &Self::T) {
        <Self::Multiplicative as Magma>::operate_assign(x, y);
    }
}

pub trait Ring: SemiRing<Additive: Invertible> {
    /// additive inverse: $-$
    fn neg(x: &Self::T) -> Self::T {
        <Self::Additive as Invertible>::inverse(x)
    }
    /// additive right inversed operaion: $-$
    fn sub(x: &Self::T, y: &Self::T) -> Self::T {
        <Self::Additive as Invertible>::rinv_operate(x, y)
    }

    fn sub_assign(x: &mut Self::T, y: &Self::T) {
        <Self::Additive as Invertible>::rinv_operate_assign(x, y);
    }
}

impl<R> Ring for R where R: SemiRing<Additive: Invertible> {}

pub trait Field: Ring<Multiplicative: Invertible> {
    /// multiplicative inverse: $-$
    fn inv(x: &Self::T) -> Self::T {
        <Self::Multiplicative as Invertible>::inverse(x)
    }
    /// multiplicative right inversed operaion: $-$
    fn div(x: &Self::T, y: &Self::T) -> Self::T {
        <Self::Multiplicative as Invertible>::rinv_operate(x, y)
    }

    fn div_assign(x: &mut Self::T, y: &Self::T) {
        <Self::Multiplicative as Invertible>::rinv_operate_assign(x, y);
    }
}

impl<F> Field for F where F: Ring<Multiplicative: Invertible> {}

/// $+,\times$
pub struct AddMulOperation<T>
where
    T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>,
{
    _marker: PhantomData<fn() -> T>,
}
impl<T> SemiRing for AddMulOperation<T>
where
    T: Clone + Zero + One + Add<Output = T> + Mul<Output = T>,
{
    type T = T;
    type Additive = AdditiveOperation<T>;
    type Multiplicative = MultiplicativeOperation<T>;
}
