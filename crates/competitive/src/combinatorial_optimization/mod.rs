#[cfg_attr(nightly, codesnip::entry("knapsack_problem"))]
mod knapsack_problem;
#[cfg_attr(nightly, codesnip::entry("largest_pattern"))]
mod largest_pattern;
#[cfg_attr(nightly, codesnip::entry("levenshtein_distance"))]
mod levenshtein_distance;
#[cfg_attr(nightly, codesnip::entry("LexicographicalSubsequence"))]
mod lexicographical_subsequence;
#[cfg_attr(nightly, codesnip::entry("LongestIncreasingSubsequence"))]
mod longest_increasing_subsequence;

#[codesnip::entry("knapsack_problem")]
pub use knapsack_problem::*;
#[codesnip::entry("largest_pattern")]
pub use largest_pattern::*;
#[codesnip::entry("levenshtein_distance")]
pub use levenshtein_distance::levenshtein_distance;
#[codesnip::entry("LexicographicalSubsequence")]
pub use lexicographical_subsequence::LexicographicalSubsequence;
#[codesnip::entry("LongestIncreasingSubsequence")]
pub use longest_increasing_subsequence::LongestIncreasingSubsequence;
