//! data structures

use crate::algebra::{AbelianMonoid, Group, Monoid, MonoidAction, SemiGroup, Unital};
use crate::algorithm::SliceBisectExt;
use crate::tools::GetDistinctMut;

#[cfg_attr(nightly, codesnip::entry("Accumulate"))]
pub use self::accumulate::Accumulate;
#[cfg_attr(nightly, codesnip::entry("automaton"))]
pub use self::automaton::*;
#[cfg_attr(nightly, codesnip::entry("BinaryIndexedTree"))]
pub use self::binary_indexed_tree::BinaryIndexedTree;
#[cfg_attr(nightly, codesnip::entry("BinaryIndexedTree2D"))]
pub use self::binary_indexed_tree_2d::BinaryIndexedTree2D;
#[cfg_attr(nightly, codesnip::entry("BitVector"))]
pub use self::bit_vector::{BitVector, RankSelectDictionaries};
#[cfg_attr(nightly, codesnip::entry("BitSet"))]
pub use self::bitset::BitSet;
#[cfg_attr(nightly, codesnip::entry("Counter"))]
pub use self::counter::{BTreeCounter, HashCounter};
#[cfg_attr(nightly, codesnip::entry("DisjointSparseTable"))]
pub use self::disjoint_sparse_table::DisjointSparseTable;
#[cfg_attr(nightly, codesnip::entry("Static2DTree"))]
pub use self::kdtree::Static2DTree;
#[cfg_attr(nightly, codesnip::entry("LazySegmentTree"))]
pub use self::lazy_segment_tree::LazySegmentTree;
#[cfg_attr(nightly, codesnip::entry("LazySegmentTreeMap"))]
pub use self::lazy_segment_tree_map::LazySegmentTreeMap;
#[cfg_attr(nightly, codesnip::entry("MergingUnionFind"))]
pub use self::merging_union_find::MergingUnionFind;
#[cfg_attr(nightly, codesnip::entry("RangeArithmeticProgressionAdd"))]
pub use self::range_ap_add::RangeArithmeticProgressionAdd;
#[cfg_attr(nightly, codesnip::entry("RangeMap"))]
pub use self::range_map::{RangeMap, RangeSet};
#[cfg_attr(nightly, codesnip::entry("SegmentTree"))]
pub use self::segment_tree::SegmentTree;
#[cfg_attr(nightly, codesnip::entry("SegmentTree2D"))]
pub use self::segment_tree_2d::SegmentTree2D;
#[cfg_attr(nightly, codesnip::entry("SegmentTreeMap"))]
pub use self::segment_tree_map::SegmentTreeMap;
#[cfg_attr(nightly, codesnip::entry("sliding_winsow_aggregation"))]
pub use self::sliding_winsow_aggregation::{DequeAggregation, QueueAggregation};
#[cfg_attr(nightly, codesnip::entry("slope_trick"))]
pub use self::slope_trick::SlopeTrick;
#[cfg_attr(nightly, codesnip::entry("Trie"))]
pub use self::trie::Trie;
#[cfg_attr(nightly, codesnip::entry("UnionFind"))]
pub use self::union_find::UnionFind;
#[cfg_attr(nightly, codesnip::entry("WaveletMatrix"))]
pub use self::wavelet_matrix::WaveletMatrix;
#[cfg_attr(nightly, codesnip::entry("WeightedUnionFind"))]
pub use self::weighted_union_find::WeightedUnionFind;

#[cfg_attr(nightly, codesnip::entry("Accumulate", include("algebra")))]
mod accumulate;
#[cfg_attr(nightly, codesnip::entry("automaton", include("algebra")))]
mod automaton;
#[cfg_attr(nightly, codesnip::entry("BinaryIndexedTree", include("algebra")))]
mod binary_indexed_tree;
#[cfg_attr(nightly, codesnip::entry("BinaryIndexedTree2D", include("algebra")))]
mod binary_indexed_tree_2d;
#[cfg_attr(nightly, codesnip::entry("BitVector"))]
mod bit_vector;
#[cfg_attr(nightly, codesnip::entry("BitSet"))]
mod bitset;
#[cfg_attr(nightly, codesnip::entry("Counter"))]
mod counter;
#[cfg_attr(nightly, codesnip::entry("DisjointSparseTable", include("algebra")))]
mod disjoint_sparse_table;
#[cfg_attr(nightly, codesnip::entry("Static2DTree"))]
mod kdtree;
#[cfg_attr(nightly, codesnip::entry("LazySegmentTree", include("MonoidAction")))]
mod lazy_segment_tree;
#[cfg_attr(
    nightly,
    codesnip::entry("LazySegmentTreeMap", include("MonoidAction"))
)]
mod lazy_segment_tree_map;
#[cfg_attr(nightly, codesnip::entry("MergingUnionFind"))]
mod merging_union_find;
#[cfg_attr(nightly, codesnip::entry("RangeArithmeticProgressionAdd"))]
mod range_ap_add;
#[cfg_attr(nightly, codesnip::entry("RangeMap"))]
mod range_map;
#[cfg_attr(nightly, codesnip::entry("SegmentTree", include("algebra")))]
mod segment_tree;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "SegmentTree2D",
        include("binary_search", "GetDistinctMut", "SegmentTree")
    )
)]
mod segment_tree_2d;
#[cfg_attr(nightly, codesnip::entry("SegmentTreeMap", include("algebra")))]
mod segment_tree_map;
#[cfg_attr(
    nightly,
    codesnip::entry("sliding_winsow_aggregation", include("algebra"))
)]
mod sliding_winsow_aggregation;
#[cfg_attr(nightly, codesnip::entry("slope_trick"))]
mod slope_trick;
#[cfg_attr(nightly, codesnip::entry("Trie"))]
mod trie;
#[cfg_attr(nightly, codesnip::entry("UnionFind"))]
mod union_find;
#[cfg_attr(nightly, codesnip::entry("WaveletMatrix", include("BitVector")))]
mod wavelet_matrix;
#[cfg_attr(nightly, codesnip::entry("WeightedUnionFind", include("algebra")))]
mod weighted_union_find;
