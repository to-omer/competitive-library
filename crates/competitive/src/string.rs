//! string algorithems

use crate::tools::Xorshift;

#[cfg_attr(nightly, codesnip::entry("KnuthMorrisPratt"))]
pub use self::knuth_morris_pratt::KnuthMorrisPratt;
#[cfg_attr(nightly, codesnip::entry("RollingHash"))]
pub use self::rolling_hash::{MultipleRollingHash, RollingHash};
#[cfg_attr(nightly, codesnip::entry("SuffixArray"))]
pub use self::suffix_array::SuffixArray;
#[cfg_attr(nightly, codesnip::entry("ZAlgorithm"))]
pub use self::z_algorithm::Zarray;

#[cfg_attr(nightly, codesnip::entry("KnuthMorrisPratt"))]
mod knuth_morris_pratt;
#[cfg_attr(nightly, codesnip::entry("RollingHash", include("Xorshift")))]
mod rolling_hash;
#[cfg_attr(nightly, codesnip::entry("SuffixArray"))]
mod suffix_array;
#[cfg_attr(nightly, codesnip::entry("ZAlgorithm"))]
mod z_algorithm;
