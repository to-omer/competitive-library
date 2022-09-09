//! mathematical datas

use crate::num::{montgomery, BarrettReduction, Complex, MInt, MIntBase, MIntConvert, One, Zero};
use crate::tools::{AssociatedValue, PartialIgnoredOrd, Xorshift};

#[codesnip::entry("berlekamp_massey")]
pub use self::berlekamp_massey::berlekamp_massey;
#[codesnip::entry("ConvolveSteps")]
pub use self::convolve_steps::ConvolveSteps;
#[codesnip::entry("discrete_logarithm")]
pub use self::discrete_logarithm::{discrete_logarithm, discrete_logarithm_prime_mod};
pub use self::factorial::*;
#[codesnip::entry("fast_fourier_transform")]
pub use self::fast_fourier_transform::ConvolveRealFft;
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
#[codesnip::entry("miller_rabin")]
pub use self::miller_rabin::{miller_rabin, miller_rabin_with_br};
#[codesnip::entry("NumberTheoreticTransform")]
pub use self::number_theoretic_transform::{Convolve, Convolve998244353, MIntConvolve};
pub use self::nums::*;
pub use self::polynomial::*;
pub use self::prime::*;
#[codesnip::entry("prime_factors")]
pub use self::prime_factors::{divisors, prime_factors, prime_factors_flatten};
#[codesnip::entry("PrimeList")]
pub use self::prime_list::PrimeList;
#[codesnip::entry("PrimeTable")]
pub use self::prime_table::PrimeTable;
#[codesnip::entry("primitive_root")]
pub use self::primitive_root::{check_primitive_root, primitive_root};
pub use self::special_modulo::*;

#[cfg_attr(nightly, codesnip::entry("berlekamp_massey", include("zero_one")))]
mod berlekamp_massey;
#[cfg_attr(nightly, codesnip::entry("ConvolveSteps"))]
mod convolve_steps;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "discrete_logarithm",
        include(
            "BarrettReduction",
            "lcm",
            "modinv",
            "primitive_root",
            "PrimeList",
            "Xorshift"
        )
    )
)]
mod discrete_logarithm;
mod factorial;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "fast_fourier_transform",
        include("Complex", "AssociatedValue", "ConvolveSteps")
    )
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
#[cfg_attr(nightly, codesnip::entry("miller_rabin", include("BarrettReduction")))]
mod miller_rabin;
#[cfg_attr(nightly, codesnip::entry("mod_sqrt", include("MIntBase")))]
mod mod_sqrt;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "NumberTheoreticTransform",
        include("montgomery", "AssociatedValue", "ConvolveSteps")
    )
)]
mod number_theoretic_transform;
mod nums;
mod polynomial;
mod prime;
#[cfg_attr(
    nightly,
    codesnip::entry("prime_factors", include("miller_rabin", "gcd"))
)]
mod prime_factors;
#[cfg_attr(nightly, codesnip::entry("PrimeList"))]
mod prime_list;
#[cfg_attr(nightly, codesnip::entry("PrimeTable"))]
mod prime_table;
#[cfg_attr(nightly, codesnip::entry("primitive_root", include("prime_factors")))]
mod primitive_root;
mod special_modulo;
