//! algorithm

mod compress;
mod convex_hull_trick;
mod search;
mod slide_minimum;
mod zeta_transform;

pub use compress::Compress;
pub use convex_hull_trick::*;
pub use search::*;
pub use slide_minimum::slide_minimum;
pub use zeta_transform::*;
