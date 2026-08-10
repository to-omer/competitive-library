use super::*;
use std::{
    marker::PhantomData,
    num::Wrapping,
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

    fn dot_product(x: &[Self::T], y: &[Self::T]) -> Self::T {
        assert_eq!(x.len(), y.len());
        x.iter().zip(y).fold(Self::zero(), |mut sum, (x, y)| {
            Self::add_assign(&mut sum, &Self::mul(x, y));
            sum
        })
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

/// Dot product using `+` and `*`.
pub trait DotProduct: Sized + Clone + Zero + Add<Output = Self> + Mul<Output = Self> {
    fn dot_product(x: &[Self], y: &[Self]) -> Self {
        assert_eq!(x.len(), y.len());
        x.iter()
            .zip(y)
            .fold(Self::zero(), |sum, (x, y)| sum + x.clone() * y.clone())
    }
}

macro_rules! impl_dot_product {
    ($($t:ty)*) => {
        $(impl DotProduct for $t {})*
    };
}
impl_dot_product!(u8 u16 u32 u64 usize u128 i8 i16 i32 i64 isize i128 f32 f64);

impl<T> DotProduct for Wrapping<T> where
    Wrapping<T>: Clone + Zero + Add<Output = Wrapping<T>> + Mul<Output = Wrapping<T>>
{
}

/// $+,\times$
pub struct AddMulOperation<T>
where
    T: DotProduct + One,
{
    _marker: PhantomData<fn() -> T>,
}
impl<T> SemiRing for AddMulOperation<T>
where
    T: DotProduct + One,
{
    type T = T;
    type Additive = AdditiveOperation<T>;
    type Multiplicative = MultiplicativeOperation<T>;

    fn dot_product(x: &[Self::T], y: &[Self::T]) -> Self::T {
        T::dot_product(x, y)
    }
}
