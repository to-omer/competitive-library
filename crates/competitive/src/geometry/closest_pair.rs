use super::Point;
use crate::tools::TotalOrd;

#[codesnip::entry("closest_pair", include("Point", "TotalOrd"))]
pub fn closest_pair(a: Vec<Point>) -> f64 {
    let mut a = a;
    a.sort_by_key(|&p| TotalOrd(p.re));
    closest_pair_inner(&mut a[..])
}
#[codesnip::entry("closest_pair")]
fn closest_pair_inner(a: &mut [Point]) -> f64 {
    use std::cmp::min;
    let n = a.len();
    if n <= 1 {
        return std::f64::INFINITY;
    }
    let m = n / 2;
    let x = a[m].re;
    let mut d = min(
        TotalOrd(closest_pair_inner(&mut a[0..m])),
        TotalOrd(closest_pair_inner(&mut a[m..n])),
    )
    .0;
    a.sort_by_key(|&p| TotalOrd(p.im));
    let mut b: Vec<Point> = vec![];
    for a in a.iter() {
        if (a.re - x).abs() >= d {
            continue;
        }
        let k = b.len();
        for j in 0..k {
            let p = *a - b[k - j - 1];
            if p.im >= d {
                break;
            }
            d = min(TotalOrd(d), TotalOrd(p.abs())).0;
        }
        b.push(*a);
    }
    d
}
