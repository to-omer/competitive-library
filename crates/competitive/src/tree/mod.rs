//! tree algorithms

use crate::{
    algebra::{LazyMapMonoid, Magma, Monoid, Unital},
    data_structure::{
        Allocator, MemoryPool, RangeMinimumQuery, binary_search_tree, splay_operations,
    },
    graph::UndirectedSparseGraph,
    math::{ConvolveSteps, U64Convolve},
    tools::{IterScan, MarkedIterScan, RandomSpec, Xorshift},
};

#[codesnip::entry("centroid_decomposition")]
pub use self::centroid_decomposition::ContourQueryRange;
#[codesnip::entry("tree_generator")]
pub use self::generator::*;
#[codesnip::entry("HeavyLightDecomposition")]
pub use self::heavy_light_decomposition::HeavyLightDecomposition;
#[codesnip::entry("LevelAncestor")]
pub use self::level_ancestor::LevelAncestor;
#[codesnip::entry("LinkCutTree")]
pub use self::link_cut_tree::{
    LinkCutTree, LinkCutTreePathFold, LinkCutTreePathUpdate, LinkCutTreeSpec,
    LinkCutTreeSubtreeFold, LinkCutTreeSubtreeUpdate, PathLinkCutTree,
};
pub use self::rerooting::ReRooting;
#[codesnip::entry("StaticTopTree")]
pub use self::static_top_tree::{Cluster, MonoidCluster, StaticTopTree, StaticTopTreeDp};
#[codesnip::entry("TopTree")]
pub use self::top_tree::{NoTopTreeAction, TopTree, TopTreeAction, TopTreeSpec};
pub use self::tree_center::*;
pub use self::tree_hash::TreeHasher;
#[codesnip::entry("XorLinkedRootedTree")]
pub use self::xor_linked_tree::*;

#[cfg_attr(
    nightly,
    codesnip::entry(
        "centroid_decomposition",
        include("SparseGraph", "NumberTheoreticTransform")
    )
)]
mod centroid_decomposition;
mod depth;
#[cfg_attr(
    nightly,
    codesnip::entry("EulerTour", include("RangeMinimumQuery", "SparseGraph", "tree_depth"))
)]
mod euler_tour;
#[cfg_attr(
    nightly,
    codesnip::entry("tree_generator", include("SparseGraph", "random_generator"))
)]
mod generator;
#[cfg_attr(
    nightly,
    codesnip::entry("HeavyLightDecomposition", include("algebra", "SparseGraph"))
)]
mod heavy_light_decomposition;
#[cfg_attr(nightly, codesnip::entry("LevelAncestor", include("SparseGraph")))]
mod level_ancestor;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "LinkCutTree",
        include("_splay_operations", "Allocator", "LazyMapMonoid")
    )
)]
mod link_cut_tree;
mod rerooting;
#[cfg_attr(
    nightly,
    codesnip::entry("StaticTopTree", include("algebra", "SparseGraph"))
)]
mod static_top_tree;
#[cfg_attr(
    nightly,
    codesnip::entry("TopTree", include("_splay_operations", "Allocator", "algebra"))
)]
mod top_tree;
mod tree_center;
#[cfg_attr(nightly, codesnip::entry("tree_centroid", include("SparseGraph")))]
mod tree_centroid;
mod tree_dp;
mod tree_hash;
mod tree_order;
#[cfg_attr(nightly, codesnip::entry("XorLinkedRootedTree", include("scanner")))]
mod xor_linked_tree;
