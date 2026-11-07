//! algorithm

use crate::algebra::{Field, Group, Invertible, Magma, Monoid, Unital};
use crate::data_structure::{
    BitSet, DaryPrefixSumTreeU32, FibHashMap, UnionFind, UnionFindBase, union_find,
};
use crate::graph::{
    Graph, Neighbor, TopologicalSortExt, UndirectedSparseGraph, UsizeGraph, VertexMap,
};
use crate::math::{Convolve998244353, ConvolveSteps, Matrix};
use crate::num::{
    Dyadic, MInt, MIntBase, One, RangeBoundsExt, URational, Unsigned, Zero, montgomery,
};
use crate::tools::{RandomSpec, SerdeByteStr, Xorshift};
use crate::tree::LevelAncestor;

#[codesnip::entry("automata_learning")]
pub use self::automata_learning::*;
#[codesnip::entry("baby_step_giant_step")]
pub use self::baby_step_giant_step::baby_step_giant_step;
#[codesnip::entry("binary_search")]
pub use self::binary_search::{Bisect, SliceBisectExt, binary_search, parallel_binary_search};
#[codesnip::entry("BitDp")]
pub use self::bitdp::{BitDpExt, Combinations, Subsets};
#[codesnip::entry("CartesianTree")]
pub use self::cartesian_tree::CartesianTree;
#[codesnip::entry("chromatic_number")]
pub use self::chromatic_number::chromatic_number;
#[codesnip::entry("combinations")]
pub use self::combinations::SliceCombinationsExt;
#[codesnip::entry("ConvexHullTrick")]
pub use self::convex_hull_trick::ConvexHullTrick;
#[codesnip::entry("Doubling")]
pub use self::doubling::{Doubling, FunctionalGraphDoubling};
#[codesnip::entry("dyadic_ranges")]
pub use self::dyadic_ranges::dyadic_ranges;
#[codesnip::entry("esper")]
pub use self::esper::{EsperEstimator, EsperSolver};
#[codesnip::entry("GameAnalyzer")]
pub use self::game_analyzer::*;
#[codesnip::entry("HornSatisfiability")]
pub use self::horn_satisfiability::HornSatisfiability;
#[codesnip::entry("number_of_increasing_sequences_between")]
pub use self::number_of_increasing_sequences_between::{
    number_of_increasing_sequences_between, number_of_increasing_sequences_between_998244353,
};
pub use self::other::*;
#[codesnip::entry("QuotientIndex")]
pub use self::quotient_index::{CeilQuotientIndex, FloorQuotientIndex};
#[codesnip::entry("RhoPath")]
pub use self::rho_path::RhoPath;
#[codesnip::entry("01_on_tree")]
pub use self::solve_01_on_tree::solve_01_on_tree;
#[codesnip::entry("sort")]
pub use self::sort::{RadixSortKey, SliceSortExt};
#[codesnip::entry("SqrtDecomposition")]
pub use self::sqrt_decomposition::{
    RangeUpdateRangeFoldSqrtDecomposition, SqrtDecomposition, SqrtDecompositionBuckets,
};
#[codesnip::entry("static_range_count_distinct")]
pub use self::static_range_count_distinct::static_range_count_distinct;
#[codesnip::entry("stern_brocot_tree")]
pub use self::stern_brocot_tree::{SbtNode, SbtPath, SternBrocotTree, rational_binary_search};
#[codesnip::entry("ternary_search")]
pub use self::ternary_search::{golden_ternary_search, piecewise_ternary_search, ternary_search};
#[codesnip::entry("ToddCoxeter")]
pub use self::todd_coxeter::ToddCoxeter;
#[codesnip::entry("XorBasis")]
pub use self::xorbasis::XorBasis;

#[codesnip::entry(
    "automata_learning",
    include("BitSet", "coding", "Matrix", "random_generator")
)]
mod automata_learning;
#[codesnip::entry("baby_step_giant_step", include("algebra"))]
mod baby_step_giant_step;
#[codesnip::entry]
mod binary_search;
#[codesnip::entry("BitDp", include("zero_one"))]
mod bitdp;
#[codesnip::entry("CartesianTree")]
mod cartesian_tree;
#[codesnip::entry("chromatic_number")]
mod chromatic_number;
#[codesnip::entry("combinations")]
mod combinations;
#[codesnip::entry("ConvexHullTrick")]
mod convex_hull_trick;
#[codesnip::entry("Doubling", include("LevelAncestor", "SparseGraph", "algebra"))]
mod doubling;
#[codesnip::entry("dyadic_ranges")]
mod dyadic_ranges;
#[codesnip::entry("esper", include("Matrix"))]
mod esper;
#[codesnip::entry(
    "GameAnalyzer",
    include("Graph", "Dyadic", "ClosureGraph", "topological_sort")
)]
mod game_analyzer;
#[codesnip::entry("HornSatisfiability")]
mod horn_satisfiability;
#[cfg_attr(nightly, codesnip::entry)]
mod mo_algorithm;
#[codesnip::entry(
    "number_of_increasing_sequences_between",
    include("NumberTheoreticTransform")
)]
mod number_of_increasing_sequences_between;
mod other;
#[codesnip::entry("QuotientIndex")]
mod quotient_index;
#[codesnip::entry("RhoPath")]
mod rho_path;
#[codesnip::entry("01_on_tree", include("UnionFind"))]
mod solve_01_on_tree;
#[codesnip::entry("sort")]
mod sort;
#[codesnip::entry("SqrtDecomposition", include("algebra", "discrete_steps"))]
mod sqrt_decomposition;
#[codesnip::entry(
    "static_range_count_distinct",
    include("DaryPrefixSumTree", "FibonacciHash")
)]
mod static_range_count_distinct;
#[codesnip::entry("stern_brocot_tree", include("URational"))]
mod stern_brocot_tree;
#[cfg_attr(nightly, codesnip::entry)]
mod syakutori;
#[codesnip::entry]
pub mod ternary_search;
#[codesnip::entry("ToddCoxeter", include("UnionFind"))]
mod todd_coxeter;
#[codesnip::entry("XorBasis")]
mod xorbasis;
