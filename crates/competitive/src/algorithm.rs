//! algorithm

use crate::num::{MInt, MIntBase, One, Zero};

#[codesnip::entry("BitDp")]
pub use self::bitdp::{BitDp, Combinations, Subsets};
#[codesnip::entry("chromatic_number")]
pub use self::chromatic_number::IndependentSubSet;
pub use self::combinations::*;
#[codesnip::entry("ConvexHullTrick")]
pub use self::convex_hull_trick::ConvexHullTrick;
pub use self::mo::MoSolver;
pub use self::other::*;
#[codesnip::entry("RhoPath")]
pub use self::rho_path::RhoPath;
pub use self::search::*;
pub use self::slide_minimum::*;
#[codesnip::entry("XorBasis")]
pub use self::xorbasis::XorBasis;
pub use self::zeta_transform::*;

#[cfg_attr(nightly, codesnip::entry("BitDp"))]
mod bitdp;
#[cfg_attr(
    nightly,
    codesnip::entry("chromatic_number", include("MIntBase", "binary_search"))
)]
mod chromatic_number;
mod combinations;
#[cfg_attr(nightly, codesnip::entry("ConvexHullTrick"))]
mod convex_hull_trick;
mod mo;
mod other;
#[cfg_attr(nightly, codesnip::entry("RhoPath"))]
mod rho_path;
mod search;
mod slide_minimum;
#[cfg_attr(nightly, codesnip::entry("XorBasis"))]
mod xorbasis;
mod zeta_transform;
