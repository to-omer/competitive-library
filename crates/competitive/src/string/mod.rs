//! string algorithems

use crate::algebra::{Invertible, Ring, SemiRing};
use crate::tools::Xorshift;

#[codesnip::entry("KnuthMorrisPratt")]
pub use self::knuth_morris_pratt::KnuthMorrisPratt;
#[codesnip::entry("RollingHash")]
pub use self::rolling_hash::{MultipleRollingHash, RollingHash};
#[codesnip::entry("SuffixArray")]
pub use self::suffix_array::SuffixArray;
#[codesnip::entry("ZAlgorithm")]
pub use self::z_algorithm::Zarray;

#[cfg_attr(nightly, codesnip::entry("KnuthMorrisPratt"))]
mod knuth_morris_pratt;
#[cfg_attr(
    nightly,
    codesnip::entry("RollingHash", include("Xorshift", "algebra", "ring"))
)]
mod rolling_hash;
#[cfg_attr(nightly, codesnip::entry("SuffixArray"))]
mod suffix_array;
#[cfg_attr(nightly, codesnip::entry("ZAlgorithm"))]
mod z_algorithm;
