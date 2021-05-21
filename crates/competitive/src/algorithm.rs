//! algorithm

#[cfg_attr(nightly, codesnip::entry("BitDp", inline))]
mod bitdp;
#[cfg_attr(nightly, codesnip::entry(inline, include("MIntBase", "binary_search")))]
mod chromatic_number;
mod combinations;
#[cfg_attr(nightly, codesnip::entry("ConvexHullTrick", inline))]
mod convex_hull_trick;
mod mo;
mod other;
#[cfg_attr(nightly, codesnip::entry("RhoPath", inline))]
mod rho_path;
mod search;
mod slide_minimum;
#[cfg_attr(nightly, codesnip::entry("XorBasis", inline))]
mod xorbasis;
mod zeta_transform;

pub use bitdp::*;
pub use chromatic_number::*;
pub use combinations::*;
pub use convex_hull_trick::*;
pub use mo::MoSolver;
pub use other::*;
pub use rho_path::RhoPath;
pub use search::*;
pub use slide_minimum::*;
pub use xorbasis::XorBasis;
pub use zeta_transform::*;
