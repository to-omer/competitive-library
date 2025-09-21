use crate::{
    num::{Complex, Float, Zero},
    tools::TotalOrd,
};

#[codesnip::entry("Approx")]
pub use self::approx::{Approx, ApproxOrd};
#[codesnip::entry("Ccw")]
pub use self::ccw::{Ccw, Ccwable};
#[codesnip::entry("Circle")]
pub use self::circle::Circle;
#[codesnip::entry("closest_pair")]
pub use self::closest_pair::closest_pair;
#[codesnip::entry("Line")]
pub use self::line::{Line, LineSegment};
#[codesnip::entry("polygon")]
pub use self::polygon::{convex_diameter, convex_hull};

#[cfg_attr(nightly, codesnip::entry("Approx"))]
mod approx;
#[cfg_attr(
    nightly,
    codesnip::entry("Ccw", include("Approx", "Complex", "zero_one"))
)]
mod ccw;
#[cfg_attr(nightly, codesnip::entry("Circle", include("Ccw")))]
mod circle;
#[cfg_attr(
    nightly,
    codesnip::entry("closest_pair", include("Complex", "TotalOrd"))
)]
mod closest_pair;
#[cfg_attr(nightly, codesnip::entry("Line", include("Ccw")))]
mod line;
#[cfg_attr(nightly, codesnip::entry("polygon", include("Ccw", "TotalOrd")))]
mod polygon;
