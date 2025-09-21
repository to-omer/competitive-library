use super::{Approx, Ccw, Ccwable, Complex, Float};

#[derive(Clone, Debug, PartialEq)]
pub struct Line<T> {
    p1: Complex<T>,
    p2: Complex<T>,
}
impl<T> Line<T> {
    pub fn new(p1: Complex<T>, p2: Complex<T>) -> Self {
        Line { p1, p2 }
    }
}
impl<T> Line<T>
where
    T: Ccwable,
{
    pub fn dir(&self) -> Complex<T> {
        self.p2 - self.p1
    }
    pub fn ccw(&self, p: Complex<T>) -> Ccw {
        Ccw::new(self.p1, self.p2, p)
    }
    pub fn is_parallel(&self, other: &Self) -> bool {
        Approx(self.dir().cross(other.dir())) == Approx(T::zero())
    }
    pub fn is_orthogonal(&self, other: &Self) -> bool {
        Approx(self.dir().dot(other.dir())) == Approx(T::zero())
    }
}
impl<T> Line<T>
where
    T: Ccwable + Float,
{
    pub fn projection(&self, p: Complex<T>) -> Complex<T> {
        let e = self.dir().unit();
        self.p1 + e * (p - self.p1).dot(e)
    }
    pub fn reflection(&self, p: Complex<T>) -> Complex<T> {
        let d = self.projection(p) - p;
        p + d + d
    }
    pub fn distance_point(&self, p: Complex<T>) -> T {
        (p / self.dir().unit()).re
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineSegment<T> {
    p1: Complex<T>,
    p2: Complex<T>,
}
impl<T> LineSegment<T> {
    pub fn new(p1: Complex<T>, p2: Complex<T>) -> Self {
        LineSegment { p1, p2 }
    }
}
impl<T> LineSegment<T>
where
    T: Ccwable,
{
    pub fn dir(&self) -> Complex<T> {
        self.p2 - self.p1
    }
    pub fn ccw(&self, p: Complex<T>) -> Ccw {
        Ccw::new(self.p1, self.p2, p)
    }
    pub fn is_parallel(&self, other: &Self) -> bool {
        Approx(self.dir().cross(other.dir())) == Approx(T::zero())
    }
    pub fn is_orthogonal(&self, other: &Self) -> bool {
        Approx(self.dir().dot(other.dir())) == Approx(T::zero())
    }
    pub fn intersect(&self, other: &Self) -> bool {
        self.ccw(other.p1) as i8 * self.ccw(other.p2) as i8 <= 0
            && other.ccw(self.p1) as i8 * other.ccw(self.p2) as i8 <= 0
    }
    pub fn intersect_point(&self, p: Complex<T>) -> bool {
        self.ccw(p) == Ccw::OnSegment
    }
}
impl<T> LineSegment<T>
where
    T: Ccwable + Float,
{
    pub fn projection(&self, p: Complex<T>) -> Complex<T> {
        let e = self.dir().unit();
        self.p1 + e * (p - self.p1).dot(e)
    }
    pub fn reflection(&self, p: Complex<T>) -> Complex<T> {
        let d = self.projection(p) - p;
        p + d + d
    }
    pub fn cross_point(&self, other: &Self) -> Option<Complex<T>> {
        if self.intersect(other) {
            let a = self.dir().cross(other.dir());
            let b = self.dir().cross(self.p2 - other.p1);
            if Approx(a.abs()) == Approx(T::zero()) && Approx(b.abs()) == Approx(T::zero()) {
                Some(other.p1)
            } else {
                Some(other.p1 + (other.dir() * b / a))
            }
        } else {
            None
        }
    }
    pub fn distance_point(&self, p: Complex<T>) -> T {
        let r = self.projection(p);
        if self.intersect_point(r) {
            (r - p).abs()
        } else {
            (self.p1 - p).abs().min((self.p2 - p).abs())
        }
    }
    pub fn distance(&self, other: &Self) -> T {
        if self.intersect(other) {
            T::zero()
        } else {
            let d1 = self.distance_point(other.p1);
            let d2 = self.distance_point(other.p2);
            let d3 = other.distance_point(self.p1);
            let d4 = other.distance_point(self.p2);
            d1.min(d2).min(d3).min(d4)
        }
    }
}
