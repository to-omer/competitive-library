#![allow(clippy::self_named_constructors)]

use crate::{num::{Complex, Zero}, tools::TotalOrd};

#[cfg_attr(nightly, codesnip::entry("Approx"))]
mod approx;
#[cfg_attr(
    nightly,
    codesnip::entry("Ccw", include("Approx", "Complex", "zero_one"))
)]
mod ccw;
#[cfg_attr(nightly, codesnip::entry("Circle", include("Approx", "Point")))]
mod circle;
#[cfg_attr(nightly, codesnip::entry("closest_pair", include("Point", "TotalOrd")))]
mod closest_pair;
#[cfg_attr(nightly, codesnip::entry("Line", include("Approx", "Ccw", "Point")))]
mod line;
mod polygon;

#[codesnip::entry("Approx")]
pub use self::approx::{Approx, ApproxOrd};
#[codesnip::entry("Ccw")]
pub use self::ccw::Ccw;
#[codesnip::entry("Circle")]
pub use self::circle::Circle;
#[codesnip::entry("closest_pair")]
pub use self::closest_pair::closest_pair;
#[codesnip::entry("Line")]
pub use self::line::{Line, LineSegment};
pub use self::polygon::{convex_diameter, convex_hull};

#[codesnip::entry("Point", include("Complex"))]
pub type Point = Complex<f64>;
