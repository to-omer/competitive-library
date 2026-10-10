use super::{Approx, ApproxOrd, Complex, Float, Zero};
use std::ops::{Add, Mul, Sub};

/// Signed area times two; positive for counterclockwise input.
pub fn polygon_area2<T>(ps: &[Complex<T>]) -> T
where
    T: Copy + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    ps.iter()
        .zip(ps.iter().cycle().skip(1))
        .fold(T::zero(), |sum, (&a, &b)| sum + a.cross(b))
}

/// `(area2, moment6)` contribution.
pub fn polygon_edge_moments<T>(a: Complex<T>, b: Complex<T>) -> (T, Complex<T>)
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    let cross = a.cross(b);
    (cross, (a + b) * cross)
}

/// `None` if the signed area is approximately zero.
pub fn polygon_centroid<T>(ps: &[Complex<T>]) -> Option<Complex<T>>
where
    T: Float + ApproxOrd,
{
    let (area, moment) = polygon_moments(ps);
    if Approx(area) == Approx(T::zero()) {
        None
    } else {
        Some(moment / (area + area + area))
    }
}

/// Signed `(area2, moment6)`; centroid is `moment6 / (3 * area2)`.
pub fn polygon_moments<T>(ps: &[Complex<T>]) -> (T, Complex<T>)
where
    T: Copy + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    ps.iter().zip(ps.iter().cycle().skip(1)).fold(
        (T::zero(), Complex::zero()),
        |(area, moment), (&a, &b)| {
            let (cross, contribution) = polygon_edge_moments(a, b);
            (area + cross, moment + contribution)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::testutil::exhaustive_sequences;

    type P = Complex<i128>;

    #[test]
    fn test_polygon_moments() {
        let points: Vec<_> = (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| P::new(x, y)))
            .collect();
        for mut ps in exhaustive_sequences(points.iter().copied(), 0..=5) {
            let mut area = 0;
            let mut moment = P::zero();
            if let Some(&first) = ps.first() {
                for triangle in ps[1..].windows(2) {
                    let cross = (triangle[0] - first).cross(triangle[1] - first);
                    area += cross;
                    moment += (first + triangle[0] + triangle[1]) * cross;
                }
            }
            assert_eq!(polygon_area2(&ps), area);
            assert_eq!(polygon_moments(&ps), (area, moment));
            ps.reverse();
            assert_eq!(polygon_moments(&ps), (-area, -moment));
            let translation = P::new(1000000000, -2000000000);
            for p in &mut ps {
                *p += translation;
            }
            assert_eq!(
                polygon_moments(&ps),
                (-area, -moment - translation * (3 * area))
            );
            if !ps.is_empty() {
                ps.push(ps[0]);
                ps.rotate_left(1);
                assert_eq!(
                    polygon_moments(&ps),
                    (-area, -moment - translation * (3 * area))
                );
            }
        }
        let triangle = [P::new(1, 0), P::new(0, 1), P::zero()];
        let scale = 1000000000000;
        let scaled = triangle.map(|p| p * scale);
        let (area, moment) = polygon_moments(&triangle);
        assert_eq!(
            polygon_moments(&scaled),
            (area * scale * scale, moment * scale * scale * scale)
        );
    }

    #[test]
    fn test_polygon_centroid() {
        let points: Vec<_> = (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| P::new(x, y)))
            .collect();
        let empty: [Complex<f64>; 0] = [];
        assert_eq!(polygon_centroid(&empty), None);
        for &a in &points {
            for &b in &points {
                for &c in &points {
                    let mut ps = [a, b, c].map(|p| p.map(|x| x as f64));
                    if (b - a).cross(c - a) == 0 {
                        assert_eq!(polygon_centroid(&ps), None);
                    } else {
                        let expected = (ps[0] + ps[1] + ps[2]) / 3.0;
                        let got = polygon_centroid(&ps).unwrap();
                        assert_eq!(got.map(Approx), expected.map(Approx));
                        ps.reverse();
                        assert_eq!(
                            polygon_centroid(&ps).unwrap().map(Approx),
                            expected.map(Approx)
                        );
                    }
                }
            }
        }
    }
}
