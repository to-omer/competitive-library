use super::*;
use crate::data_structure::TotalOrd;
use cargo_snippet::snippet;

#[snippet("convex_hull")]
pub fn convex_hull(ps: Vec<Point>) -> Vec<Point> {
    let mut ps = ps;
    ps.sort_by(|p1, p2| ((p1.re, p1.im).partial_cmp(&(p2.re, p2.im)).unwrap()));
    let mut qs = Vec::new();
    for &p in ps.iter().chain(ps.iter().rev().skip(1)) {
        while {
            let k = qs.len();
            k > 1 && ccw(qs[k - 2], qs[k - 1], p) == CCW::Clockwise
        } {
            qs.pop();
        }
        qs.push(p);
    }
    qs.pop();
    qs
}

#[snippet("convex_diameter")]
pub fn convex_diameter(ps: Vec<Point>) -> f64 {
    let n = ps.len();
    let mut i = (0..n).max_by_key(|&i| TotalOrd(ps[i].re)).unwrap_or(0);
    let mut j = (0..n).min_by_key(|&i| TotalOrd(ps[i].re)).unwrap_or(0);
    let mut res = (ps[i] - ps[j]).norm();
    let (maxi, maxj) = (i, j);
    loop {
        let (ni, nj) = ((i + 1) % n, (j + 1) % n);
        if (ps[ni] - ps[i]).cross(ps[nj] - ps[j]) < 0. {
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
    res.sqrt()
}
