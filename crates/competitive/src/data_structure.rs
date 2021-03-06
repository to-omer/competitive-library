//! data structures

use crate::algebra::{AbelianMonoid, Group, Monoid, MonoidAction, SemiGroup, Unital};

#[codesnip::entry("Accumulate")]
pub use self::accumulate::Accumulate;
#[codesnip::entry("automaton")]
pub use self::automaton::*;
#[codesnip::entry("BinaryIndexedTree")]
pub use self::binary_indexed_tree::BinaryIndexedTree;
#[codesnip::entry("BinaryIndexedTree2D")]
pub use self::binary_indexed_tree_2d::BinaryIndexedTree2D;
#[codesnip::entry("BitVector")]
pub use self::bit_vector::{BitVector, RankSelectDictionaries};
#[codesnip::entry("BitSet")]
pub use self::bitset::BitSet;
#[codesnip::entry("Counter")]
pub use self::counter::{BTreeCounter, HashCounter};
#[codesnip::entry("DisjointSparseTable")]
pub use self::disjoint_sparse_table::DisjointSparseTable;
#[codesnip::entry("Static2DTree")]
pub use self::kdtree::Static2DTree;
#[codesnip::entry("LazySegmentTree")]
pub use self::lazy_segment_tree::LazySegmentTree;
#[codesnip::entry("LazySegmentTreeMap")]
pub use self::lazy_segment_tree_map::LazySegmentTreeMap;
#[codesnip::entry("MergingUnionFind")]
pub use self::merging_union_find::MergingUnionFind;
#[codesnip::entry("RangeArithmeticProgressionAdd")]
pub use self::range_ap_add::RangeArithmeticProgressionAdd;
#[codesnip::entry("RangeMap")]
pub use self::range_map::{RangeMap, RangeSet};
#[codesnip::entry("SegmentTree")]
pub use self::segment_tree::SegmentTree;
#[codesnip::entry("SegmentTreeMap")]
pub use self::segment_tree_map::SegmentTreeMap;
#[codesnip::entry("sliding_winsow_aggregation")]
pub use self::sliding_winsow_aggregation::{DequeAggregation, QueueAggregation};
#[codesnip::entry("Trie")]
pub use self::trie::Trie;
#[codesnip::entry("UnionFind")]
pub use self::union_find::UnionFind;
#[codesnip::entry("WaveletMatrix")]
pub use self::wavelet_matrix::WaveletMatrix;
#[codesnip::entry("WeightedUnionFind")]
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
#[cfg_attr(nightly, codesnip::entry("SegmentTreeMap", include("algebra")))]
mod segment_tree_map;
#[cfg_attr(
    nightly,
    codesnip::entry("sliding_winsow_aggregation", include("algebra"))
)]
mod sliding_winsow_aggregation;
#[cfg_attr(nightly, codesnip::entry("Trie"))]
mod trie;
#[cfg_attr(nightly, codesnip::entry("UnionFind"))]
mod union_find;
#[cfg_attr(nightly, codesnip::entry("WaveletMatrix", include("BitVector")))]
mod wavelet_matrix;
#[cfg_attr(nightly, codesnip::entry("WeightedUnionFind", include("algebra")))]
mod weighted_union_find;
