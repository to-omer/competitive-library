//! algorithm

use crate::algebra::{Group, Invertible, Magma, Monoid, Ring, Unital};
use crate::math::{with_prime_list, ConvolveSteps, Matrix};
use crate::num::{MInt, MIntBase, One, Zero};

#[cfg_attr(nightly, codesnip::entry("baby_step_giant_step"))]
pub use self::baby_step_giant_step::baby_step_giant_step;
#[cfg_attr(nightly, codesnip::entry("binary_search"))]
pub use self::binary_search::{binary_search, parallel_binary_search, Bisect, SliceBisectExt};
#[codesnip::entry("BitDp")]
pub use self::bitdp::{BitDp, Combinations, Subsets};
#[codesnip::entry("BitwiseandConvolve")]
pub use self::bitwiseand_convolve::BitwiseandConvolve;
#[codesnip::entry("BitwiseorConvolve")]
pub use self::bitwiseor_convolve::BitwiseorConvolve;
#[codesnip::entry("chromatic_number")]
pub use self::chromatic_number::IndependentSubSet;
#[codesnip::entry("combinations")]
pub use self::combinations::SliceCombinationsExt;
#[codesnip::entry("ConvexHullTrick")]
pub use self::convex_hull_trick::ConvexHullTrick;
#[codesnip::entry("esper")]
pub use self::esper::{EsperEstimator, EsperSolver};
#[codesnip::entry("GcdConvolve")]
pub use self::gcd_convolve::GcdConvolve;
#[codesnip::entry("ImpartialGame")]
pub use self::impartial_game::{ImpartialGame, ImpartialGameAnalyzer, ImpartialGamer};
#[codesnip::entry("LcmConvolve")]
pub use self::lcm_convolve::LcmConvolve;
pub use self::other::*;
#[codesnip::entry("PartisanGame")]
pub use self::partisan_game::{PartisanGame, PartisanGameAnalyzer, PartisanGamer};
#[codesnip::entry("RhoPath")]
pub use self::rho_path::RhoPath;
pub use self::slide_minimum::*;
#[codesnip::entry("sort")]
pub use self::sort::SliceSortExt;
#[codesnip::entry("SqrtDecomposition")]
pub use self::sqrt_decomposition::{SqrtDecomposition, SqrtDecompositionBuckets};
#[codesnip::entry("ternary_search")]
pub use self::ternary_search::ternary_search;
#[codesnip::entry("XorBasis")]
pub use self::xorbasis::XorBasis;
#[codesnip::entry("ZeroSumGame")]
pub use self::zero_sum_game::{ZeroSumGame, ZeroSumGameAnalyzer, ZeroSumGamer};

#[cfg_attr(nightly, codesnip::entry("baby_step_giant_step", include("algebra")))]
mod baby_step_giant_step;
#[cfg_attr(nightly, codesnip::entry)]
mod binary_search;
#[cfg_attr(nightly, codesnip::entry("BitDp"))]
mod bitdp;
#[cfg_attr(
    nightly,
    codesnip::entry("BitwiseandConvolve", include("_zeta_transform", "avx_helper"))
)]
mod bitwiseand_convolve;
#[cfg_attr(
    nightly,
    codesnip::entry("BitwiseorConvolve", include("_zeta_transform", "avx_helper"))
)]
mod bitwiseor_convolve;
#[cfg_attr(
    nightly,
    codesnip::entry("chromatic_number", include("MIntBase", "binary_search"))
)]
mod chromatic_number;
#[cfg_attr(nightly, codesnip::entry("combinations"))]
mod combinations;
#[cfg_attr(nightly, codesnip::entry("ConvexHullTrick"))]
mod convex_hull_trick;
#[cfg_attr(nightly, codesnip::entry("esper", include("Matrix")))]
mod esper;
#[cfg_attr(
    nightly,
    codesnip::entry("GcdConvolve", include("_zeta_transform", "PrimeList"))
)]
mod gcd_convolve;
#[cfg_attr(nightly, codesnip::entry("ImpartialGame"))]
mod impartial_game;
#[cfg_attr(
    nightly,
    codesnip::entry("LcmConvolve", include("_zeta_transform", "PrimeList"))
)]
mod lcm_convolve;
#[cfg_attr(nightly, codesnip::entry)]
mod mo_algorithm;
mod other;
#[cfg_attr(nightly, codesnip::entry("PartisanGame"))]
mod partisan_game;
#[cfg_attr(nightly, codesnip::entry("RhoPath"))]
mod rho_path;
mod slide_minimum;
#[cfg_attr(nightly, codesnip::entry("sort"))]
mod sort;
#[cfg_attr(nightly, codesnip::entry("SqrtDecomposition", include("algebra")))]
mod sqrt_decomposition;
#[cfg_attr(nightly, codesnip::entry)]
mod syakutori;
#[cfg_attr(nightly, codesnip::entry)]
mod ternary_search;
#[cfg_attr(nightly, codesnip::entry("XorBasis"))]
mod xorbasis;
#[cfg_attr(nightly, codesnip::entry("ZeroSumGame"))]
mod zero_sum_game;

#[codesnip::entry("_zeta_transform", include("algebra", "ring", "ConvolveSteps"))]
#[codesnip::skip]
#[allow(dead_code)]
#[doc(hidden)]
enum ZetaTransformSnippets {}
