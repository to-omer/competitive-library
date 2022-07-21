//! mathematical datas

use crate::num::{montgomery, Complex, MInt, MIntBase, MIntConvert, One, Zero};
use crate::tools::{AssociatedValue, PartialIgnoredOrd};

#[codesnip::entry("berlekamp_massey")]
pub use self::berlekamp_massey::berlekamp_massey;
pub use self::factorial::*;
#[codesnip::entry("fast_fourier_transform")]
pub use self::fast_fourier_transform::convolve_fft;
#[codesnip::entry("FormalPowerSeries")]
pub use self::formal_power_series::{
    FormalPowerSeries, FormalPowerSeriesCoefficient, FormalPowerSeriesCoefficientSqrt, Fps,
    Fps998244353,
};
pub use self::gcd::*;
#[codesnip::entry("lagrange_interpolation")]
pub use self::lagrange_interpolation::{lagrange_interpolation, lagrange_interpolation_polynomial};
#[codesnip::entry("Matrix")]
pub use self::matrix::Matrix;
#[codesnip::entry("NumberTheoreticTransform")]
pub use self::number_theoretic_transform::{
    Convolve, Convolve998244353, ConvolveSteps, MIntConvolve,
};
pub use self::nums::*;
pub use self::polynomial::*;
pub use self::prime::*;
#[codesnip::entry("PrimeList")]
pub use self::prime_list::PrimeList;
#[codesnip::entry("PrimeTable")]
pub use self::prime_table::PrimeTable;
pub use self::special_modulo::*;

#[cfg_attr(nightly, codesnip::entry("berlekamp_massey", include("zero_one")))]
mod berlekamp_massey;
mod factorial;
#[cfg_attr(
    nightly,
    codesnip::entry("fast_fourier_transform", include("Complex", "AssociatedValue"))
)]
mod fast_fourier_transform;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "FormalPowerSeries",
        include(
            "NumberTheoreticTransform",
            "montgomery",
            "mod_sqrt",
            "factorial",
            "PartialIgnoredOrd",
            "berlekamp_massey"
        )
    )
)]
mod formal_power_series;
mod gcd;
#[cfg_attr(
    nightly,
    codesnip::entry("lagrange_interpolation", include("factorial", "MIntBase"))
)]
mod lagrange_interpolation;
#[cfg_attr(nightly, codesnip::entry("Matrix", include("zero_one")))]
mod matrix;
#[cfg_attr(nightly, codesnip::entry("mod_sqrt", include("MIntBase")))]
mod mod_sqrt;
#[cfg_attr(
    nightly,
    codesnip::entry("NumberTheoreticTransform", include("montgomery", "AssociatedValue"))
)]
mod number_theoretic_transform;
mod nums;
mod polynomial;
mod prime;
#[cfg_attr(nightly, codesnip::entry("PrimeList"))]
pub mod prime_list;
#[cfg_attr(nightly, codesnip::entry("PrimeTable"))]
mod prime_table;
mod special_modulo;
