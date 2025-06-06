//! data structures

use crate::algebra::{
    AbelianGroup, AbelianMonoid, Associative, Group, Magma, MaxOperation, MinOperation, Monoid,
    MonoidAction, SemiGroup, Unital,
};
use crate::algorithm::SliceBisectExt;
use crate::num::{Bounded, RangeBoundsExt};
use crate::tools::GetDistinctMut;

#[codesnip::entry("Accumulate")]
pub use self::accumulate::{Accumulate, Accumulate2d, AccumulateKd};
#[codesnip::entry("Allocator")]
pub use self::allocator::{Allocator, MemoryPool};
#[codesnip::entry("BinaryIndexedTree")]
pub use self::binary_indexed_tree::BinaryIndexedTree;
#[codesnip::entry("BinaryIndexedTree2D")]
pub use self::binary_indexed_tree_2d::BinaryIndexedTree2D;
#[codesnip::entry("BitVector")]
pub use self::bit_vector::{BitVector, RankSelectDictionaries};
#[codesnip::entry("BitSet")]
pub use self::bitset::BitSet;
#[codesnip::entry("BTreeMapExt")]
pub use self::btreemap_ext::{BTreeMapExt, BTreeSetExt};
#[codesnip::entry("compress")]
pub use self::compress::{Compressor, HashCompress, VecCompress};
#[codesnip::entry("CompressedBinaryIndexedTree")]
pub use self::compressed_binary_indexed_tree::{
    CompressedBinaryIndexedTree, CompressedBinaryIndexedTree1d, CompressedBinaryIndexedTree2d,
    CompressedBinaryIndexedTree3d, CompressedBinaryIndexedTree4d,
};
#[codesnip::entry("CompressedSegmentTree")]
pub use self::compressed_segment_tree::{
    CompressedSegmentTree, CompressedSegmentTree1d, CompressedSegmentTree2d,
    CompressedSegmentTree3d, CompressedSegmentTree4d,
};
#[codesnip::entry("container")]
pub use self::container::{
    BTreeMapFactory, Container, ContainerEntry, ContainerFactory, HashMapFactory,
    HashMapFactoryWithCapacity,
};
#[codesnip::entry("Counter")]
pub use self::counter::{BTreeCounter, HashCounter};
#[codesnip::entry("DisjointSparseTable")]
pub use self::disjoint_sparse_table::DisjointSparseTable;
#[codesnip::entry("FibonacciHash")]
pub use self::fibonacci_hash::{FibHashMap, FibHashSet};
#[codesnip::entry("Static2DTree")]
pub use self::kdtree::Static2DTree;
#[codesnip::entry("LazySegmentTree")]
pub use self::lazy_segment_tree::LazySegmentTree;
#[codesnip::entry("LazySegmentTreeMap")]
pub use self::lazy_segment_tree_map::LazySegmentTreeMap;
#[codesnip::entry("LineSet")]
pub use self::line_set::LineSet;
#[codesnip::entry("PartiallyRetroactivePriorityQueue")]
pub use self::partially_retroactive_priority_queue::PartiallyRetroactivePriorityQueue;
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
#[codesnip::entry("slope_trick")]
pub use self::slope_trick::SlopeTrick;
#[codesnip::entry("SparseSet")]
pub use self::sparse_set::SparseSet;
#[codesnip::entry("SplayTree")]
pub use self::splay_tree::{SplayMap, SplaySequence};
#[codesnip::entry("transducer")]
pub use self::transducer::*;
#[codesnip::entry("Trie")]
pub use self::trie::Trie;
#[codesnip::entry("UnionFind")]
pub use self::union_find::{
    MergingUnionFind, PotentializedUnionFind, UndoableUnionFind, UnionFind, UnionFindBase,
};
#[codesnip::entry("VecMap")]
pub use self::vec_map::{FixedVecMapFactory, VecMap, VecMapFactory, VecMapFactoryWithCapacity};
#[codesnip::entry("WaveletMatrix")]
pub use self::wavelet_matrix::WaveletMatrix;

#[cfg_attr(
    nightly,
    codesnip::entry("Accumulate", include("algebra", "discrete_steps"))
)]
mod accumulate;
#[cfg_attr(nightly, codesnip::entry("Allocator"))]
mod allocator;
#[cfg_attr(nightly, codesnip::entry("BinaryIndexedTree", include("algebra")))]
mod binary_indexed_tree;
#[cfg_attr(nightly, codesnip::entry("BinaryIndexedTree2D", include("algebra")))]
mod binary_indexed_tree_2d;
#[cfg_attr(nightly, codesnip::entry("BitVector"))]
mod bit_vector;
#[cfg_attr(nightly, codesnip::entry("BitSet"))]
mod bitset;
#[cfg_attr(nightly, codesnip::entry("BTreeMapExt"))]
mod btreemap_ext;
#[cfg_attr(nightly, codesnip::entry("compress", include("binary_search")))]
mod compress;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "CompressedBinaryIndexedTree",
        include("algebra", "binary_search", "GetDistinctMut")
    )
)]
mod compressed_binary_indexed_tree;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "CompressedSegmentTree",
        include("algebra", "binary_search", "GetDistinctMut")
    )
)]
mod compressed_segment_tree;
#[cfg_attr(nightly, codesnip::entry("container"))]
mod container;
#[cfg_attr(nightly, codesnip::entry("Counter"))]
mod counter;
#[cfg_attr(nightly, codesnip::entry("DisjointSparseTable", include("algebra")))]
mod disjoint_sparse_table;
#[cfg_attr(nightly, codesnip::entry("FibonacciHash"))]
mod fibonacci_hash;
#[cfg_attr(nightly, codesnip::entry("Static2DTree"))]
mod kdtree;
#[cfg_attr(
    nightly,
    codesnip::entry("LazySegmentTree", include("MonoidAction", "discrete_steps"))
)]
mod lazy_segment_tree;
#[cfg_attr(
    nightly,
    codesnip::entry("LazySegmentTreeMap", include("MonoidAction", "discrete_steps"))
)]
mod lazy_segment_tree_map;
#[cfg_attr(nightly, codesnip::entry("LineSet", include("bounded")))]
mod line_set;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "PartiallyRetroactivePriorityQueue",
        include("bounded", "SegmentTree", "MaxOperation", "MinOperation")
    )
)]
pub mod partially_retroactive_priority_queue;
#[cfg_attr(nightly, codesnip::entry("RangeArithmeticProgressionAdd"))]
mod range_ap_add;
#[cfg_attr(nightly, codesnip::entry("RangeMap"))]
mod range_map;
#[cfg_attr(
    nightly,
    codesnip::entry("SegmentTree", include("algebra", "discrete_steps"))
)]
mod segment_tree;
#[cfg_attr(
    nightly,
    codesnip::entry("SegmentTreeMap", include("algebra", "discrete_steps"))
)]
mod segment_tree_map;
#[cfg_attr(
    nightly,
    codesnip::entry("sliding_winsow_aggregation", include("algebra"))
)]
mod sliding_winsow_aggregation;
#[cfg_attr(nightly, codesnip::entry("slope_trick"))]
mod slope_trick;
#[cfg_attr(nightly, codesnip::entry("SparseSet"))]
mod sparse_set;
#[cfg_attr(
    nightly,
    codesnip::entry("SplayTree", include("Allocator", "MonoidAction"))
)]
pub mod splay_tree;
#[cfg_attr(
    nightly,
    codesnip::entry("transducer", include("algebra", "container", "VecMap"))
)]
mod transducer;
#[cfg_attr(nightly, codesnip::entry("Trie", include("algebra")))]
mod trie;
#[cfg_attr(
    nightly,
    codesnip::entry("UnionFind", include("algebra", "TupleOperation"))
)]
pub mod union_find;
#[cfg_attr(nightly, codesnip::entry("VecMap", include("container")))]
mod vec_map;
#[cfg_attr(nightly, codesnip::entry("WaveletMatrix", include("BitVector")))]
mod wavelet_matrix;
