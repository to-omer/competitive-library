//! algorithm

use crate::algebra::{Group, Magma, Monoid, Unital};
use crate::math::Matrix;
use crate::num::{MInt, MIntBase, One, Zero};

#[cfg_attr(nightly, codesnip::entry("baby_step_giant_step"))]
pub use self::baby_step_giant_step::baby_step_giant_step;
#[cfg_attr(nightly, codesnip::entry("binary_search"))]
pub use self::binary_search::*;
#[codesnip::entry("BitDp")]
pub use self::bitdp::{BitDp, Combinations, Subsets};
#[codesnip::entry("chromatic_number")]
pub use self::chromatic_number::IndependentSubSet;
#[codesnip::entry("combinations")]
pub use self::combinations::SliceCombinationsExt;
#[codesnip::entry("ConvexHullTrick")]
pub use self::convex_hull_trick::ConvexHullTrick;
#[codesnip::entry("esper")]
pub use self::esper::{EsperEstimator, EsperSolver};
#[codesnip::entry("ImpartialGame")]
pub use self::impartial_game::{ImpartialGame, ImpartialGameAnalyzer, ImpartialGamer};
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
pub use self::ternary_search::*;
#[codesnip::entry("XorBasis")]
pub use self::xorbasis::XorBasis;
#[codesnip::entry("ZeroSumGame")]
pub use self::zero_sum_game::{ZeroSumGame, ZeroSumGameAnalyzer, ZeroSumGamer};
#[codesnip::entry("zeta_transform")]
pub use self::zeta_transform::{
    DivisorTransform, MultipleTransform, SubsetTransform, SupersetTransform,
};

#[cfg_attr(nightly, codesnip::entry("BabyStepGiantStep", include("algebra")))]
mod baby_step_giant_step;
#[cfg_attr(nightly, codesnip::entry)]
mod binary_search;
#[cfg_attr(nightly, codesnip::entry("BitDp"))]
mod bitdp;
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
#[cfg_attr(nightly, codesnip::entry("ImpartialGame"))]
mod impartial_game;
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
#[cfg_attr(nightly, codesnip::entry("zeta_transform", include("algebra")))]
mod zeta_transform;
