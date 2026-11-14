#[cfg(target_arch = "x86_64")]
use crate::tools::avx512_enabled;
use crate::{
    data_structure::{BitSet, DoublyLinkedList, WaveletMatrix},
    num::Zero,
};

#[codesnip::entry("knapsack_problem")]
pub use self::knapsack_problem::*;
#[codesnip::entry("largest_pattern")]
pub use self::largest_pattern::*;
#[codesnip::entry("levenshtein_distance")]
pub use self::levenshtein_distance::{
    LevenshteinEdit, levenshtein_distance, levenshtein_distance_with_edits,
};
#[codesnip::entry("LexicographicalSubsequence")]
pub use self::lexicographical_subsequence::LexicographicalSubsequence;
#[codesnip::entry("longest_common_subsequence")]
pub use self::longest_common_subsequence::{
    longest_common_subsequence, longest_common_subsequence_length,
};
#[codesnip::entry("LongestIncreasingSubsequence")]
pub use self::longest_increasing_subsequence::LongestIncreasingSubsequence;
#[codesnip::entry("maximum_scoring_segment_sets")]
pub use self::maximum_scoring_segment_sets::maximum_scoring_segment_sets;
#[codesnip::entry("RangeLongestIncreasingSubsequence")]
pub use self::range_longest_increasing_subsequence::{
    RangeLongestIncreasingSubsequence, WeightedRangeLongestIncreasingSubsequence,
};
#[codesnip::entry("SubsetSumProblem")]
pub use self::subset_sum_problem::SubsetSumProblem;

#[codesnip::entry("knapsack_problem")]
mod knapsack_problem;
#[codesnip::entry("largest_pattern")]
mod largest_pattern;
#[codesnip::entry("levenshtein_distance", include("avx_helper"))]
mod levenshtein_distance;
#[codesnip::entry("LexicographicalSubsequence")]
mod lexicographical_subsequence;
#[codesnip::entry("longest_common_subsequence", include("avx_helper"))]
mod longest_common_subsequence;
#[codesnip::entry("LongestIncreasingSubsequence")]
mod longest_increasing_subsequence;
#[codesnip::entry(
    "maximum_scoring_segment_sets",
    include("DoublyLinkedList", "zero_one")
)]
mod maximum_scoring_segment_sets;
#[codesnip::entry(
    "RangeLongestIncreasingSubsequence",
    include("WaveletMatrix", "zero_one")
)]
mod range_longest_increasing_subsequence;
#[codesnip::entry("SubsetSumProblem", include("BitSet"))]
mod subset_sum_problem;
