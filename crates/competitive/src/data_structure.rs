//! data structures

#[cfg_attr(nightly, codesnip::entry("Accumulate", inline, include("algebra")))]
mod accumulate;
#[cfg_attr(nightly, codesnip::entry(inline, include("algebra")))]
mod automaton;
mod binary_indexed_tree;
#[cfg_attr(nightly, codesnip::entry("RankSelectDictionaries", inline))]
mod bit_vector;
#[cfg_attr(nightly, codesnip::entry("BitSet", inline))]
mod bitset;
#[cfg_attr(
    nightly,
    codesnip::entry("DisjointSparseTable", inline, include("algebra"))
)]
mod disjoint_sparse_table;
mod kdtree;
mod lazy_segment_tree;
mod range_ap_add;
mod segment_tree;
mod sliding_winsow_aggregation;
mod trie;
mod union_find;
mod wavelet_matrix;

pub use accumulate::*;
pub use automaton::*;
pub use binary_indexed_tree::{BinaryIndexedTree, BinaryIndexedTree2D};
pub use bit_vector::*;
pub use bitset::BitSet;
pub use disjoint_sparse_table::DisjointSparseTable;
pub use kdtree::*;
pub use lazy_segment_tree::{LazySegmentTree, LazySegmentTreeMap};
pub use range_ap_add::RangeArithmeticProgressionAdd;
pub use segment_tree::{SegmentTree, SegmentTreeMap};
pub use sliding_winsow_aggregation::{DequeAggregation, QueueAggregation};
pub use trie::*;
pub use union_find::*;
pub use wavelet_matrix::*;
