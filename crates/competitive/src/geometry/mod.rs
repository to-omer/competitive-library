use crate::{
    algebra::{ArrayOperation, Invertible, RangeMinCountRangeAdd, Ring, SemiRing},
    algorithm::SliceSortExt,
    data_structure::{BinaryIndexedTree, LazySegmentTree, StaticSearch},
    num::{Complex, Float, Zero},
    tools::TotalOrd,
};

#[codesnip::entry("Approx")]
pub use self::approx::{Approx, ApproxOrd};
#[codesnip::entry("area_of_union_of_rectangles")]
pub use self::area_of_union_of_rectangles::area_of_union_of_rectangles;
#[codesnip::entry("Ccw")]
pub use self::ccw::{Ccw, Ccwable};
#[codesnip::entry("Circle")]
pub use self::circle::Circle;
#[codesnip::entry("closest_pair")]
pub use self::closest_pair::closest_pair;
#[codesnip::entry("ConvexHull")]
pub use self::convex_hull::ConvexHull;
#[codesnip::entry("Line")]
pub use self::line::{Line, LineSegment};
#[codesnip::entry("polygon")]
pub use self::polygon::{polygon_area2, polygon_centroid, polygon_edge_moments, polygon_moments};
#[codesnip::entry("StaticRectangleAddRectangleSum")]
pub use self::static_rectangle_add_rectangle_sum::StaticRectangleAddRectangleSum;

#[cfg_attr(nightly, codesnip::entry("Approx"))]
mod approx;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "area_of_union_of_rectangles",
        include("LazySegmentTree", "StaticSearch", "sort")
    )
)]
mod area_of_union_of_rectangles;
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
#[cfg_attr(nightly, codesnip::entry("ConvexHull", include("Complex", "TotalOrd")))]
mod convex_hull;
#[cfg_attr(nightly, codesnip::entry("Line", include("Ccw")))]
mod line;
#[cfg_attr(nightly, codesnip::entry("polygon", include("Approx", "Complex")))]
mod polygon;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "StaticRectangleAddRectangleSum",
        include("BinaryIndexedTree", "StaticSearch", "sort", "ring", "ArrayOperation")
    )
)]
mod static_rectangle_add_rectangle_sum;
