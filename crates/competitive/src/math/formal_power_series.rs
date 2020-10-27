use crate::{
    math::{convolve3, NTTModulus, NumberTheoreticTransform},
    num::{modulus, MInt, Modulus, One, Zero},
};

#[derive(Debug, Default)]
pub struct FormalPowerSeries<T, Multiplier> {
    pub data: Vec<T>,
    _marker: std::marker::PhantomData<Multiplier>,
}

pub type FPS998244353 = FormalPowerSeries<modulus::MInt998244353, modulus::Modulo998244353>;
pub type FPS<M> = FormalPowerSeries<MInt<M>, M>;

pub trait FormalPowerSeriesCoefficient:
    Sized
    + Clone
    + Zero
    + PartialEq
    + One
    + From<usize>
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + for<'r> std::ops::Add<&'r Self, Output = Self>
    + for<'r> std::ops::Sub<&'r Self, Output = Self>
    + for<'r> std::ops::Mul<&'r Self, Output = Self>
    + for<'r> std::ops::Div<&'r Self, Output = Self>
    + std::ops::AddAssign<Self>
    + std::ops::SubAssign<Self>
    + std::ops::MulAssign<Self>
    + std::ops::DivAssign<Self>
    + for<'r> std::ops::AddAssign<&'r Self>
    + for<'r> std::ops::SubAssign<&'r Self>
    + for<'r> std::ops::MulAssign<&'r Self>
    + for<'r> std::ops::DivAssign<&'r Self>
    + std::ops::Neg<Output = Self>
{
}

impl<M: Modulus> FormalPowerSeriesCoefficient for MInt<M> {}

pub trait FormalPowerSeriesMultiplier: Sized {
    type T;
    fn convolve(
        x: &FormalPowerSeries<Self::T, Self>,
        y: &FormalPowerSeries<Self::T, Self>,
    ) -> FormalPowerSeries<Self::T, Self>;
}

struct DefaultFormalPowerSeriesMultiplier<M>(std::marker::PhantomData<M>);

impl<M: Modulus> FormalPowerSeriesMultiplier for DefaultFormalPowerSeriesMultiplier<M> {
    type T = MInt<M>;
    fn convolve(
        x: &FormalPowerSeries<Self::T, Self>,
        y: &FormalPowerSeries<Self::T, Self>,
    ) -> FormalPowerSeries<Self::T, Self> {
        let z = convolve3(
            x.data.iter().map(|x| x.inner()).collect(),
            y.data.iter().map(|x| x.inner()).collect(),
        );
        FormalPowerSeries::from_vec(z)
    }
}

impl<M: NTTModulus> FormalPowerSeriesMultiplier for M {
    type T = MInt<M>;
    fn convolve(
        x: &FormalPowerSeries<Self::T, Self>,
        y: &FormalPowerSeries<Self::T, Self>,
    ) -> FormalPowerSeries<Self::T, Self> {
        let z = NumberTheoreticTransform::<M>::convolve(x.data.clone(), y.data.clone());
        FormalPowerSeries::from_vec(z)
    }
}

pub trait FormalPowerSeriesCoefficientSqrt: FormalPowerSeriesCoefficient {
    fn sqrt(&self) -> Option<Self>;
}

impl<M: Modulus> FormalPowerSeriesCoefficientSqrt for MInt<M> {
    fn sqrt(&self) -> Option<Self> {
        self.clone().sqrt()
    }
}

mod formal_power_series_impls;
mod formal_power_series_nums;
