use super::{Complex, TotalOrd, Zero};
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Neg, Sub},
};

/// Counterclockwise vertices from the least `(x, y)`, without collinear intermediates.
#[derive(Clone, Debug, PartialEq)]
pub struct ConvexHull<T> {
    ps: Vec<Complex<T>>,
}

impl<T> ConvexHull<T> {
    pub fn from_points(mut ps: Vec<Complex<T>>) -> Self
    where
        T: Copy + PartialOrd + Zero + Sub<Output = T> + Mul<Output = T>,
    {
        ps.sort_unstable_by(|p1, p2| (p1.re, p1.im).partial_cmp(&(p2.re, p2.im)).unwrap());
        ps.dedup();
        if ps.len() <= 2 {
            return Self { ps };
        }
        let mut qs: Vec<Complex<T>> = Vec::new();
        for &p in &ps {
            while {
                let k = qs.len();
                k > 1 && (qs[k - 1] - qs[k - 2]).cross(p - qs[k - 2]) <= T::zero()
            } {
                qs.pop();
            }
            qs.push(p);
        }
        let lower = qs.len();
        for &p in ps.iter().rev().skip(1) {
            while {
                let k = qs.len();
                k > lower && (qs[k - 1] - qs[k - 2]).cross(p - qs[k - 2]) <= T::zero()
            } {
                qs.pop();
            }
            qs.push(p);
        }
        qs.pop();
        Self { ps: qs }
    }

    pub fn as_slice(&self) -> &[Complex<T>] {
        &self.ps
    }

    pub fn into_vec(self) -> Vec<Complex<T>> {
        self.ps
    }

    pub fn scale(&mut self, factor: T)
    where
        T: Copy + PartialOrd + Zero + Mul<Output = T>,
    {
        if factor == T::zero() {
            self.ps.truncate(1);
        }
        for p in &mut self.ps {
            *p *= factor;
        }
        if factor < T::zero()
            && let Some(first) = (0..self.ps.len()).min_by_key(|&i| TotalOrd(self.ps[i]))
        {
            self.ps.rotate_left(first);
        }
    }

    pub fn translate(&mut self, by: Complex<T>)
    where
        T: Copy + Add<Output = T>,
    {
        for p in &mut self.ps {
            *p += by;
        }
    }

    /// Squared diameter; `None` for an empty hull.
    pub fn diameter(&self) -> Option<T>
    where
        T: Copy + PartialOrd + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
    {
        let ps = self.as_slice();
        if ps.is_empty() {
            return None;
        }
        let n = ps.len();
        let mut i = (0..n).max_by_key(|&i| TotalOrd(ps[i].re)).unwrap();
        let mut j = 0;
        let mut res = (ps[i] - ps[j]).norm();
        for _ in 0..2 * n {
            let (ni, nj) = ((i + 1) % n, (j + 1) % n);
            if (ps[ni] - ps[i]).cross(ps[nj] - ps[j]) < T::zero() {
                i = ni;
            } else {
                j = nj;
            }
            let d = (ps[i] - ps[j]).norm();
            if res < d {
                res = d;
            }
        }
        Some(res)
    }

    pub fn minkowski_sum(&self, other: &Self) -> Self
    where
        T: Copy
            + Ord
            + Zero
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Neg<Output = T>,
    {
        let (ps, qs) = (&self.ps, &other.ps);
        if ps.is_empty() || qs.is_empty() {
            return Self { ps: Vec::new() };
        }
        let mut result = Vec::with_capacity(ps.len() + qs.len());
        let (mut i, mut j) = (0, 0);
        while i < ps.len() || j < qs.len() {
            result.push(ps[i % ps.len()] + qs[j % qs.len()]);
            let ordering = if i == ps.len() {
                Ordering::Greater
            } else if j == qs.len() {
                Ordering::Less
            } else {
                (ps[(i + 1) % ps.len()] - ps[i])
                    .transpose()
                    .map(|x| -x)
                    .cmp_by_arg((qs[(j + 1) % qs.len()] - qs[j]).transpose().map(|x| -x))
                    .reverse()
            };
            match ordering {
                Ordering::Less => i += 1,
                Ordering::Greater => j += 1,
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
            }
        }
        Self { ps: result }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Approx, Ccw};
    use std::collections::BTreeSet;

    type P = Complex<i128>;

    fn strict_hull(ps: Vec<P>) -> Vec<P> {
        let ps: Vec<_> = ps
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if ps.len() < 2 {
            return ps;
        }
        let first = ps[0];
        let mut result = vec![first];
        let mut current = first;
        loop {
            let mut next = *ps.iter().find(|&&p| p != current).unwrap();
            for &p in &ps {
                let cross = (next - current).cross(p - current);
                if cross < 0 || cross == 0 && (p - current).norm() > (next - current).norm() {
                    next = p;
                }
            }
            if next == first {
                break;
            }
            result.push(next);
            current = next;
        }
        result
    }

    #[test]
    fn test_convex_hull() {
        let points: Vec<_> = (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| P::new(x, y)))
            .collect();
        for mask in 0..1usize << points.len() {
            let ps: Vec<_> = points
                .iter()
                .enumerate()
                .filter(|&(i, _)| mask >> i & 1 != 0)
                .map(|(_, &p)| p)
                .collect();
            let expected = strict_hull(ps.clone());
            let repeated: Vec<_> = ps.iter().flat_map(|&p| [p, p]).collect();
            let diameter = ps
                .iter()
                .flat_map(|&a| ps.iter().map(move |&b| (a - b).norm()))
                .max();
            let mut reversed = repeated.clone();
            reversed.reverse();
            for input in [ps.clone(), repeated, reversed] {
                let hull = ConvexHull::from_points(input.clone());
                assert_eq!(hull.as_slice(), expected);
                assert_eq!(hull.diameter(), diameter);
                assert_eq!(hull.into_vec(), expected);
                let hull =
                    ConvexHull::from_points(input.iter().map(|p| p.map(|x| x as f64)).collect());
                assert_eq!(
                    hull.diameter().map(Approx),
                    diameter.map(|x| Approx(x as f64))
                );
                for (x, y) in [
                    (1.0, 1.0),
                    (1e-10, 1.0),
                    (1.0, 9e-9),
                    (1e8, 1.0),
                    (1000.0, 1e-13),
                ] {
                    let mut points: Vec<_> = input
                        .iter()
                        .map(|p| Complex::new(p.re as f64 * x, p.im as f64 * y))
                        .collect();
                    if let Some(&p) = points.first() {
                        points.push(p + Complex::new(1e-10, 0.0));
                    }
                    let diameter = points
                        .iter()
                        .flat_map(|&a| points.iter().map(move |&b| (a - b).norm()))
                        .max_by_key(|&x| TotalOrd(x));
                    let hull = ConvexHull::from_points(points);
                    assert_eq!(hull.diameter().map(Approx), diameter.map(Approx));
                    let vertices = hull.as_slice();
                    for i in 0..vertices.len() {
                        assert_ne!(
                            Ccw::new(
                                vertices[i],
                                vertices[(i + 1) % vertices.len()],
                                vertices[(i + 2) % vertices.len()],
                            ),
                            Ccw::Clockwise,
                            "vertices={vertices:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_transform() {
        let points: Vec<_> = (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| P::new(x, y)))
            .collect();
        for mask in 0..1usize << points.len() {
            let ps: Vec<_> = points
                .iter()
                .enumerate()
                .filter(|&(i, _)| mask >> i & 1 != 0)
                .map(|(_, &p)| p)
                .collect();
            for factor in -2..=2 {
                for &by in &points {
                    let expected = strict_hull(ps.iter().map(|&p| p * factor + by).collect());
                    let mut hull = ConvexHull::from_points(ps.clone());
                    hull.scale(factor);
                    hull.translate(by);
                    assert_eq!(hull.as_slice(), expected);
                    let mut hull =
                        ConvexHull::from_points(ps.iter().map(|p| p.map(|x| x as f64)).collect());
                    hull.scale(factor as f64);
                    hull.translate(by.map(|x| x as f64));
                    assert_eq!(
                        hull.into_vec(),
                        expected
                            .iter()
                            .map(|p| p.map(|x| x as f64))
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    #[test]
    fn test_minkowski_sum() {
        let points: Vec<_> = (-1..=1)
            .flat_map(|x| (-1..=1).map(move |y| P::new(x, y)))
            .collect();
        let sets: Vec<Vec<_>> = (0..1usize << points.len())
            .map(|mask| {
                points
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| mask >> i & 1 != 0)
                    .map(|(_, &p)| p)
                    .collect()
            })
            .collect();
        let (left_by, right_by) = (P::new(2, -3), P::new(-5, 1));
        for (p, q) in [(1, 1), (4, 9), (-6, 3), (0, 10)] {
            let left: Vec<_> = sets
                .iter()
                .map(|ps| {
                    let mut hull = ConvexHull::from_points(ps.clone());
                    hull.scale(p);
                    hull.translate(left_by);
                    hull
                })
                .collect();
            let right: Vec<_> = sets
                .iter()
                .map(|ps| {
                    let mut hull = ConvexHull::from_points(ps.clone());
                    hull.scale(q);
                    hull.translate(right_by);
                    hull
                })
                .collect();
            for (i, ps) in sets.iter().enumerate() {
                for (j, qs) in sets.iter().enumerate() {
                    let result = left[i].minkowski_sum(&right[j]).into_vec();
                    let expected = strict_hull(
                        ps.iter()
                            .flat_map(|&a| {
                                qs.iter().map(move |&b| a * p + left_by + b * q + right_by)
                            })
                            .collect(),
                    );
                    assert_eq!(result, expected, "p={p}, q={q}, ps={ps:?}, qs={qs:?}");
                }
            }
        }
    }
}
