mod circle;
mod closest_pair;
mod line;
mod polygon;

pub use circle::Circle;
pub use closest_pair::closest_pair;
pub use line::{Line, LineSegment};
pub use polygon::{convex_diameter, convex_hull};

use crate::num::Complex;

#[codesnip::entry("Point", include("Complex"))]
pub type Point = Complex<f64>;

#[codesnip::entry("EPS")]
pub const EPS: f64 = 1e-8;
#[codesnip::entry("Real", include("EPS"))]
#[derive(Clone, Debug)]
pub struct Real(pub f64);
#[codesnip::entry("Real")]
impl PartialEq for Real {
    fn eq(&self, other: &Real) -> bool {
        (self.0 - other.0).abs() < EPS
    }
}
#[codesnip::entry("Real")]
impl PartialOrd for Real {
    fn partial_cmp(&self, other: &Real) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(std::cmp::Ordering::Equal)
        } else {
            self.0.partial_cmp(&other.0)
        }
    }
}

#[codesnip::entry("Ccw", include("Point", "Real"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ccw {
    /// a--b--c
    OnlineFront = -2,
    /// a--b-vc
    Clockwise = -1,
    /// a--c--b
    OnSegment = 0,
    /// a--b-^c
    CounterClockwise = 1,
    /// c--a--b
    OnlineBack = 2,
}
#[codesnip::entry("Ccw")]
impl Ccw {
    pub fn ccw(a: Point, b: Point, c: Point) -> Self {
        let x = b - a;
        let y = c - a;
        if Real(x.cross(y)) > Real(0.) {
            Self::CounterClockwise
        } else if Real(x.cross(y)) < Real(0.) {
            Self::Clockwise
        } else if Real(x.dot(y)) < Real(0.) {
            Self::OnlineBack
        } else if Real(x.abs()) < Real(y.abs()) {
            Self::OnlineFront
        } else {
            Self::OnSegment
        }
    }
}
