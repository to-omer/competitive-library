use crate::data_structure::BitSet;

#[codesnip::entry("knapsack_problem")]
pub use self::knapsack_problem::*;
#[codesnip::entry("largest_pattern")]
pub use self::largest_pattern::*;
#[codesnip::entry("levenshtein_distance")]
pub use self::levenshtein_distance::levenshtein_distance;
#[codesnip::entry("LexicographicalSubsequence")]
pub use self::lexicographical_subsequence::LexicographicalSubsequence;
#[codesnip::entry("LongestIncreasingSubsequence")]
pub use self::longest_increasing_subsequence::LongestIncreasingSubsequence;
#[codesnip::entry("SubsetSumProblem")]
pub use self::subset_sum_problem::SubsetSumProblem;

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
#[cfg_attr(nightly, codesnip::entry("SubsetSumProblem"))]
mod subset_sum_problem;
