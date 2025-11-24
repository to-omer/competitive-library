//! string algorithms

use crate::algebra::{Gf2_63, Invertible, Mersenne61, Monoid, Ring, SemiRing};
use crate::algorithm::binary_search;
use crate::math::{Convolve, ConvolveSteps};
use crate::num::{montgomery, Zero};
use crate::tools::Xorshift;

#[codesnip::entry("KnuthMorrisPratt")]
pub use self::knuth_morris_pratt::KnuthMorrisPratt;
#[codesnip::entry("RollingHash")]
pub use self::rolling_hash::{
    Gf2_63x1, Gf2_63x2, Gf2_63x3, HashedRangeChained, Mersenne61x1, Mersenne61x2, Mersenne61x3,
    RollingHasher,
};
#[codesnip::entry("SuffixArray")]
pub use self::suffix_array::SuffixArray;
#[codesnip::entry("SuffixAutomaton")]
pub use self::suffix_automaton::SuffixAutomaton;
#[codesnip::entry("wildcard_pattern_matching")]
pub use self::wildcard_pattern_matching::wildcard_pattern_matching;
#[codesnip::entry("ZAlgorithm")]
pub use self::z_algorithm::Zarray;

#[cfg_attr(nightly, codesnip::entry("KnuthMorrisPratt"))]
mod knuth_morris_pratt;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "RollingHash",
        include("Xorshift", "algebra", "ring", "Gf2_63", "Mersenne61")
    )
)]
pub mod rolling_hash;
#[cfg_attr(nightly, codesnip::entry("SuffixArray", include("binary_search")))]
mod suffix_array;
#[cfg_attr(nightly, codesnip::entry("SuffixAutomaton"))]
mod suffix_automaton;
#[cfg_attr(nightly, codesnip::entry(include("NumberTheoreticTransform")))]
mod wildcard_pattern_matching;
#[cfg_attr(nightly, codesnip::entry("ZAlgorithm"))]
mod z_algorithm;
