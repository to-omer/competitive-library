//! string algorithems

#[cfg_attr(nightly, codesnip::entry("KnuthMorrisPratt", inline))]
mod knuth_morris_pratt;
#[cfg_attr(nightly, codesnip::entry("RollingHash", inline, include("Xorshift")))]
mod rolling_hash;
#[cfg_attr(nightly, codesnip::entry("SuffixArray", inline))]
mod suffix_array;
#[cfg_attr(nightly, codesnip::entry("ZAlgorithm", inline))]
mod z_algorithm;

pub use knuth_morris_pratt::KnuthMorrisPratt;
pub use rolling_hash::{MultipleRollingHash, RollingHash};
pub use suffix_array::SuffixArray;
pub use z_algorithm::Zarray;
