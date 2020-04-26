use super::*;
use cargo_snippet::snippet;

#[snippet("Circle")]
#[derive(Clone, Debug, PartialEq)]
pub struct Circle {
    c: Point,
    r: f64,
}
#[snippet("Circle")]
impl Circle {
    pub fn new(c: Point, r: f64) -> Self {
        Circle { c: c, r: r }
    }
    pub fn cross_circle(&self, other: &Self) -> Option<(Point, Point)> {
        let d = (self.c - other.c).abs();
        let rc = (d * d + self.r * self.r - other.r * other.r) / (2. * d);
        let rs2 = self.r * self.r - rc * rc;
        if rs2 < 0. {
            return None;
        }
        let rs = rs2.abs().sqrt();
        let diff = (other.c - self.c) / d;
        Some((
            self.c + diff * Point::new(rc, rs),
            self.c + diff * Point::new(rc, -rs),
        ))
    }
    pub fn contains_point(&self, p: Point) -> bool {
        Real((self.c - p).abs()) <= Real(self.r)
    }
}
