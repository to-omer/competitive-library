use super::{
    berlekamp_massey, montgomery::MInt998244353, Convolve998244353, ConvolveSteps, MInt,
    MIntConvert, MIntConvolve, MemorizedFactorial, One, PartialIgnoredOrd, Zero,
};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Default)]
pub struct FormalPowerSeries<T, C> {
    pub data: Vec<T>,
    _marker: PhantomData<C>,
}

impl<T, C> Debug for FormalPowerSeries<T, C>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.data, f)
    }
}

pub type Fps998244353 = FormalPowerSeries<MInt998244353, Convolve998244353>;
pub type Fps<M> = FormalPowerSeries<MInt<M>, MIntConvolve<M>>;

pub trait FormalPowerSeriesCoefficient:
    Sized
    + Clone
    + Zero
    + PartialEq
    + One
    + From<usize>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + for<'r> Add<&'r Self, Output = Self>
    + for<'r> Sub<&'r Self, Output = Self>
    + for<'r> Mul<&'r Self, Output = Self>
    + for<'r> Div<&'r Self, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + for<'r> AddAssign<&'r Self>
    + for<'r> SubAssign<&'r Self>
    + for<'r> MulAssign<&'r Self>
    + for<'r> DivAssign<&'r Self>
    + Neg<Output = Self>
{
}

impl<M> FormalPowerSeriesCoefficient for MInt<M> where M: MIntConvert<usize> {}

pub trait FormalPowerSeriesCoefficientSqrt: FormalPowerSeriesCoefficient {
    fn sqrt_coefficient(&self) -> Option<Self>;
}

impl<M> FormalPowerSeriesCoefficientSqrt for MInt<M>
where
    M: MIntConvert<u32> + MIntConvert<usize>,
{
    fn sqrt_coefficient(&self) -> Option<Self> {
        self.sqrt()
    }
}

mod formal_power_series_impls;
mod formal_power_series_nums;
