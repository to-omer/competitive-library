#[codesnip::skip]
use crate::{
    math::{Convolve998244353, ConvolveSteps, MIntConvolve},
    num::{mint_basic, MInt, MIntConvert, One, Zero},
};

#[derive(Debug, Default)]
pub struct FormalPowerSeries<T, C> {
    pub data: Vec<T>,
    _marker: std::marker::PhantomData<C>,
}

pub type Fps998244353 = FormalPowerSeries<mint_basic::MInt998244353, Convolve998244353>;
pub type Fps<M> = FormalPowerSeries<MInt<M>, MIntConvolve<M>>;

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

impl<M> FormalPowerSeriesCoefficient for MInt<M> where M: MIntConvert<usize> {}

pub trait FormalPowerSeriesCoefficientSqrt: FormalPowerSeriesCoefficient {
    fn sqrt_coefficient(&self) -> Option<Self>;
}

impl<M: MIntConvert<u32> + MIntConvert<usize>> FormalPowerSeriesCoefficientSqrt for MInt<M> {
    fn sqrt_coefficient(&self) -> Option<Self> {
        self.sqrt()
    }
}

mod formal_power_series_impls;
mod formal_power_series_nums;
