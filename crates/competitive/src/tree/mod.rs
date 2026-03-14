//! tree algorithms

use crate::{
    algebra::{Magma, Monoid, Unital},
    data_structure::RangeMinimumQuery,
    graph::UndirectedSparseGraph,
    math::{ConvolveSteps, U64Convolve},
    tools::{RandomSpec, Xorshift},
};

#[codesnip::entry("centroid_decomposition")]
pub use self::centroid_decomposition::ContourQueryRange;
#[codesnip::entry("tree_generator")]
pub use self::generator::*;
#[codesnip::entry("HeavyLightDecomposition")]
pub use self::heavy_light_decomposition::HeavyLightDecomposition;
#[codesnip::entry("LevelAncestor")]
pub use self::level_ancestor::LevelAncestor;
pub use self::rerooting::ReRooting;
#[codesnip::entry("StaticTopTree")]
pub use self::static_top_tree::{Cluster, MonoidCluster, StaticTopTree, StaticTopTreeDp};
pub use self::tree_center::*;
pub use self::tree_hash::TreeHasher;

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
mod rerooting;
#[cfg_attr(
    nightly,
    codesnip::entry("StaticTopTree", include("algebra", "SparseGraph"))
)]
mod static_top_tree;
mod tree_center;
#[cfg_attr(nightly, codesnip::entry("tree_centroid", include("SparseGraph")))]
mod tree_centroid;
mod tree_dp;
mod tree_hash;
mod tree_order;
