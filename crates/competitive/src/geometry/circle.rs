use super::{Approx, Ccwable, Complex, Float};

#[derive(Clone, Debug, PartialEq)]
pub struct Circle<T> {
    c: Complex<T>,
    r: T,
}
impl<T> Circle<T>
where
    T: Ccwable + Float,
{
    pub fn new(c: Complex<T>, r: T) -> Self {
        Circle { c, r }
    }
    pub fn cross_circle(&self, other: &Self) -> Option<(Complex<T>, Complex<T>)> {
        let d = (self.c - other.c).abs();
        let rc = (d * d + self.r * self.r - other.r * other.r) / (d + d);
        let rs2 = self.r * self.r - rc * rc;
        if Approx(rs2) < Approx(T::zero()) {
            return None;
        }
        let rs = rs2.abs().sqrt();
        let diff = (other.c - self.c) / d;
        Some((
            self.c + diff * Complex::new(rc, rs),
            self.c + diff * Complex::new(rc, -rs),
        ))
    }
    pub fn contains_point(&self, p: Complex<T>) -> bool {
        Approx((self.c - p).abs()) <= Approx(self.r)
    }
}
