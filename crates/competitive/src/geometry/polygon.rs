use super::{Approx, Ccw, Ccwable, Complex, TotalOrd};

pub fn convex_hull<T>(mut ps: Vec<Complex<T>>) -> Vec<Complex<T>>
where
    T: PartialOrd + Ccwable,
{
    ps.sort_by(|p1, p2| (p1.re, p1.im).partial_cmp(&(p2.re, p2.im)).unwrap());
    let mut qs = Vec::new();
    for &p in ps.iter().chain(ps.iter().rev().skip(1)) {
        while {
            let k = qs.len();
            k > 1 && matches!(Ccw::ccw(qs[k - 2], qs[k - 1], p), Ccw::Clockwise)
        } {
            qs.pop();
        }
        qs.push(p);
    }
    qs.pop();
    qs
}

/// Return norm
pub fn convex_diameter<T>(ps: &[Complex<T>]) -> T
where
    T: PartialOrd + Ccwable,
{
    let n = ps.len();
    let mut i = (0..n).max_by_key(|&i| TotalOrd(ps[i].re)).unwrap_or(0);
    let mut j = (0..n).min_by_key(|&i| TotalOrd(ps[i].re)).unwrap_or(0);
    let mut res = (ps[i] - ps[j]).norm();
    let (maxi, maxj) = (i, j);
    loop {
        let (ni, nj) = ((i + 1) % n, (j + 1) % n);
        if Approx((ps[ni] - ps[i]).cross(ps[nj] - ps[j])) < Approx(T::zero()) {
            i = ni;
        } else {
            j = nj;
        }
        let d = (ps[i] - ps[j]).norm();
        if res < d {
            res = d;
        }
        if i == maxi && j == maxj {
            break;
        }
    }
    res
}
