//! string algorithems

#[cfg_attr(nightly, snippet::entry("KnuthMorrisPratt", inline))]
mod knuth_morris_pratt;
#[cfg_attr(nightly, snippet::entry("RollingHash", inline, include("Xorshift")))]
mod rolling_hash;
#[cfg_attr(nightly, snippet::entry("SuffixArray", inline))]
mod suffix_array;
#[cfg_attr(nightly, snippet::entry("ZAlgorithm", inline))]
mod z_algorithm;

pub use knuth_morris_pratt::KnuthMorrisPratt;
pub use rolling_hash::{MultipleRollingHash, RollingHash};
pub use suffix_array::SuffixArray;
pub use z_algorithm::Zarray;
