//! tree algorithems

use crate::{
    graph::UndirectedSparseGraph,
    tools::{RandomSpec, Xorshift},
};

pub use self::euler_tour::*;
#[codesnip::entry("tree_generator")]
pub use self::generator::*;
pub use self::heavy_light_decomposition::*;
pub use self::rerooting::ReRooting;
pub use self::tree_center::*;
pub use self::tree_hash::TreeHasher;
pub use self::tree_rec::TreeRec;

mod depth;
mod euler_tour;
#[cfg_attr(
    nightly,
    codesnip::entry("tree_generator", include("SparseGraph", "random_generator"))
)]
mod generator;
mod heavy_light_decomposition;
mod rerooting;
mod tree_center;
mod tree_dp;
mod tree_hash;
mod tree_order;
mod tree_rec;
