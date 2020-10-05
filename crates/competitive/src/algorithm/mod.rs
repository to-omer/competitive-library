//! algorithm

mod combinations;
#[cfg_attr(nightly, snippet::entry("ConvexHullTrick", inline))]
mod convex_hull_trick;
mod mo;
mod other;
mod search;
mod slide_minimum;
mod zeta_transform;

pub use combinations::*;
pub use convex_hull_trick::*;
pub use mo::MoSolver;
pub use other::*;
pub use search::*;
pub use slide_minimum::*;
pub use zeta_transform::*;
