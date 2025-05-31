//! tree algorithems

use crate::{
    algebra::Monoid,
    graph::UndirectedSparseGraph,
    tools::{RandomSpec, Xorshift},
};

pub use self::euler_tour::*;
#[codesnip::entry("tree_generator")]
pub use self::generator::*;
#[codesnip::entry("HeavyLightDecomposition")]
pub use self::heavy_light_decomposition::HeavyLightDecomposition;
pub use self::rerooting::ReRooting;
pub use self::tree_center::*;
pub use self::tree_hash::TreeHasher;

mod depth;
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
mod rerooting;
mod tree_center;
#[cfg_attr(nightly, codesnip::entry("tree_centroid", include("SparseGraph")))]
mod tree_centroid;
mod tree_dp;
mod tree_hash;
mod tree_order;
