//! algorithm

use crate::algebra::{Field, Invertible, Magma, Monoid, Unital};
use crate::data_structure::{BitSet, UnionFindBase, union_find};
use crate::math::Matrix;
use crate::num::{MInt, MIntBase, One, RangeBoundsExt, URational, Unsigned, Zero};
use crate::tools::{RandomSpec, SerdeByteStr, Xorshift};

#[cfg_attr(nightly, codesnip::entry("automata_learning"))]
pub use self::automata_learning::*;
#[cfg_attr(nightly, codesnip::entry("baby_step_giant_step"))]
pub use self::baby_step_giant_step::baby_step_giant_step;
#[cfg_attr(nightly, codesnip::entry("binary_search"))]
pub use self::binary_search::{Bisect, SliceBisectExt, binary_search, parallel_binary_search};
#[codesnip::entry("BitDp")]
pub use self::bitdp::{BitDpExt, Combinations, Subsets};
#[codesnip::entry("CartesianTree")]
pub use self::cartesian_tree::CartesianTree;
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
#[codesnip::entry("01_on_tree")]
pub use self::solve_01_on_tree::solve_01_on_tree;
#[codesnip::entry("sort")]
pub use self::sort::SliceSortExt;
#[codesnip::entry("SqrtDecomposition")]
pub use self::sqrt_decomposition::{
    RangeUpdateRangeFoldSqrtDecomposition, SqrtDecomposition, SqrtDecompositionBuckets,
};
#[codesnip::entry("stern_brocot_tree")]
pub use self::stern_brocot_tree::{SbtNode, SbtPath, SternBrocotTree};
#[codesnip::entry("ternary_search")]
pub use self::ternary_search::{piecewise_ternary_search, ternary_search};
#[codesnip::entry("XorBasis")]
pub use self::xorbasis::XorBasis;
#[codesnip::entry("ZeroSumGame")]
pub use self::zero_sum_game::{ZeroSumGame, ZeroSumGameAnalyzer, ZeroSumGamer};

#[cfg_attr(
    nightly,
    codesnip::entry(
        "automata_learning",
        include("BitSet", "coding", "Matrix", "random_generator")
    )
)]
mod automata_learning;
#[cfg_attr(nightly, codesnip::entry("baby_step_giant_step", include("algebra")))]
mod baby_step_giant_step;
#[cfg_attr(nightly, codesnip::entry)]
mod binary_search;
#[cfg_attr(nightly, codesnip::entry("BitDp", include("zero_one")))]
mod bitdp;
#[cfg_attr(nightly, codesnip::entry("CartesianTree"))]
mod cartesian_tree;
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
#[cfg_attr(nightly, codesnip::entry("01_on_tree", include("UnionFind")))]
mod solve_01_on_tree;
#[cfg_attr(nightly, codesnip::entry("sort"))]
mod sort;
#[cfg_attr(
    nightly,
    codesnip::entry("SqrtDecomposition", include("algebra", "discrete_steps"))
)]
mod sqrt_decomposition;
#[cfg_attr(nightly, codesnip::entry("stern_brocot_tree", include("URational")))]
mod stern_brocot_tree;
#[cfg_attr(nightly, codesnip::entry)]
mod syakutori;
#[cfg_attr(nightly, codesnip::entry)]
pub mod ternary_search;
#[cfg_attr(nightly, codesnip::entry("XorBasis"))]
mod xorbasis;
#[cfg_attr(nightly, codesnip::entry("ZeroSumGame"))]
mod zero_sum_game;
