//! mathematical datas

use crate::algebra::{
    AddMulOperation, Associative, Field, Group, Invertible, Magma, Monoid, Ring, SemiRing, Unital,
};
use crate::array;
use crate::num::{
    BarrettReduction, Complex, ExtendedGcd, MInt, MIntBase, MIntConvert, One, RangeBoundsExt,
    Signed, Unsigned, Wrapping, Zero, montgomery,
};
use crate::tools::{AssociatedValue, PartialIgnoredOrd, SerdeByteStr, Xorshift};

#[codesnip::entry("ArbitraryModBinomial")]
pub use self::arbitrary_mod_binomial::ArbitraryModBinomial;
#[codesnip::entry("berlekamp_massey")]
pub use self::berlekamp_massey::berlekamp_massey;
#[codesnip::entry("bitwise_transform")]
pub use self::bitwise_transform::bitwise_transform;
#[codesnip::entry("BitwiseandConvolve")]
pub use self::bitwiseand_convolve::BitwiseandConvolve;
#[codesnip::entry("BitwiseorConvolve")]
pub use self::bitwiseor_convolve::BitwiseorConvolve;
#[codesnip::entry("BitwisexorConvolve")]
pub use self::bitwisexor_convolve::BitwisexorConvolve;
#[codesnip::entry("BlackBoxMatrix")]
pub use self::black_box_matrix::{
    BlackBoxMIntMatrix, BlackBoxMatrix, BlackBoxMatrixImpl, SparseMatrix,
};
#[codesnip::entry("ConvolveSteps")]
pub use self::convolve_steps::ConvolveSteps;
#[codesnip::entry("discrete_logarithm")]
pub use self::discrete_logarithm::{discrete_logarithm, discrete_logarithm_prime_mod};
#[codesnip::entry("factorial")]
pub use self::factorial::MemorizedFactorial;
#[codesnip::entry("fast_fourier_transform")]
pub use self::fast_fourier_transform::ConvolveRealFft;
#[codesnip::entry("floor_sum")]
pub use self::floor_sum::{
    floor_sum, floor_sum_i64, floor_sum_polynomial, floor_sum_polynomial_i64, floor_sum_range_freq,
};
#[codesnip::entry("FormalPowerSeries")]
pub use self::formal_power_series::{
    FormalPowerSeries, FormalPowerSeriesCoefficient, FormalPowerSeriesCoefficientSqrt, Fps,
    Fps998244353,
};
#[codesnip::entry("garner")]
pub use self::garner::Garner;
pub use self::gcd::*;
#[codesnip::entry("GcdConvolve")]
pub use self::gcd_convolve::GcdConvolve;
#[codesnip::entry("lagrange_interpolation")]
pub use self::lagrange_interpolation::{lagrange_interpolation, lagrange_interpolation_polynomial};
#[codesnip::entry("LcmConvolve")]
pub use self::lcm_convolve::LcmConvolve;
#[codesnip::entry("linear_congruence")]
pub use self::linear_congruence::{solve_linear_congruence, solve_simultaneous_linear_congruence};
#[codesnip::entry("linear_diophantine")]
pub use self::linear_diophantine::solve_linear_diophantine;
#[codesnip::entry("Matrix")]
pub use self::matrix::Matrix;
#[codesnip::entry("miller_rabin")]
pub use self::miller_rabin::{miller_rabin, miller_rabin_with_br};
#[codesnip::entry("MIntMatrix")]
pub use self::mint_matrix::MIntMatrix;
#[codesnip::entry("NumberTheoreticTransform")]
pub use self::number_theoretic_transform::{Convolve, Convolve998244353, MIntConvolve};
pub use self::polynomial::*;
#[codesnip::entry("PowPrec")]
pub use self::pow_prec::PowPrec;
pub use self::prime::*;
#[codesnip::entry("prime_factors")]
pub use self::prime_factors::{divisors, prime_factors, prime_factors_flatten};
#[codesnip::entry("PrimeList")]
pub use self::prime_list::{PrimeList, with_prime_list};
#[codesnip::entry("PrimeTable")]
pub use self::prime_table::PrimeTable;
#[codesnip::entry("primitive_root")]
pub use self::primitive_root::{check_primitive_root, primitive_root};
#[codesnip::entry("QuotientArray")]
pub use self::quotient_array::QuotientArray;
#[codesnip::entry("SmallModMemorizedFactorial")]
pub use self::small_factorial::SmallModMemorizedFactorial;
#[codesnip::entry("SubsetConvolve")]
pub use self::subset_convolve::SubsetConvolve;

#[cfg_attr(
    nightly,
    codesnip::entry(
        "ArbitraryModBinomial",
        include("BarrettReduction", "integer", "linear_congruence", "prime_factors")
    )
)]
mod arbitrary_mod_binomial;
#[cfg_attr(nightly, codesnip::entry("berlekamp_massey", include("zero_one")))]
mod berlekamp_massey;
#[cfg_attr(nightly, codesnip::entry("bitwise_transform"))]
mod bitwise_transform;
#[cfg_attr(
    nightly,
    codesnip::entry("BitwiseandConvolve", include("_zeta_transform", "bitwise_transform"))
)]
mod bitwiseand_convolve;
#[cfg_attr(
    nightly,
    codesnip::entry("BitwiseorConvolve", include("_zeta_transform", "bitwise_transform"))
)]
mod bitwiseor_convolve;
#[cfg_attr(
    nightly,
    codesnip::entry("BitwisexorConvolve", include("_zeta_transform", "bitwise_transform"))
)]
mod bitwisexor_convolve;
#[cfg_attr(
    nightly,
    codesnip::entry("BlackBoxMatrix", include("FormalPowerSeries", "Matrix", "Xorshift"))
)]
mod black_box_matrix;
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
#[cfg_attr(nightly, codesnip::entry("factorial", include("MIntBase")))]
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
    codesnip::entry("floor_sum", include("algebra", "ring", "integer", "BarrettReduction"))
)]
mod floor_sum;
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
#[cfg_attr(nightly, codesnip::entry(include("integer")))]
mod garner;
mod gcd;
#[cfg_attr(
    nightly,
    codesnip::entry("GcdConvolve", include("_zeta_transform", "PrimeList"))
)]
mod gcd_convolve;
#[cfg_attr(
    nightly,
    codesnip::entry("lagrange_interpolation", include("factorial", "MIntBase"))
)]
mod lagrange_interpolation;
#[cfg_attr(
    nightly,
    codesnip::entry("LcmConvolve", include("_zeta_transform", "PrimeList"))
)]
mod lcm_convolve;
#[cfg_attr(nightly, codesnip::entry(include("integer")))]
mod linear_congruence;
#[cfg_attr(nightly, codesnip::entry(include("integer", "discrete_steps")))]
mod linear_diophantine;
#[cfg_attr(
    nightly,
    codesnip::entry("Matrix", include("zero_one", "ring", "coding"))
)]
mod matrix;
#[cfg_attr(nightly, codesnip::entry("miller_rabin", include("BarrettReduction")))]
mod miller_rabin;
#[cfg_attr(
    nightly,
    codesnip::entry("MIntMatrix", include("Matrix", "factorial", "Xorshift"))
)]
mod mint_matrix;
#[cfg_attr(nightly, codesnip::entry("mod_sqrt", include("MIntBase")))]
mod mod_sqrt;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "NumberTheoreticTransform",
        include("montgomery", "ConvolveSteps", "avx_helper")
    )
)]
mod number_theoretic_transform;
mod polynomial;
#[cfg_attr(
    nightly,
    codesnip::entry("PowPrec", include("MIntBase", "prime_factors"))
)]
mod pow_prec;
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
#[cfg_attr(
    nightly,
    codesnip::entry("QuotientArray", include("algebra", "ring", "PrimeList"))
)]
mod quotient_array;
#[cfg_attr(
    nightly,
    codesnip::entry("SmallModMemorizedFactorial", include("MIntBase", "prime_factors"))
)]
mod small_factorial;
#[cfg_attr(
    nightly,
    codesnip::entry("SubsetConvolve", include("BitwiseorConvolve"))
)]
mod subset_convolve;

#[codesnip::entry("_zeta_transform", include("algebra", "ring", "ConvolveSteps"))]
#[codesnip::skip]
#[allow(dead_code)]
#[doc(hidden)]
enum ZetaTransformSnippets {}
