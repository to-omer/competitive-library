pub use competitive::algebra::LinearOperation;
pub use competitive::data_structure::SegmentTree;
pub use competitive::num::{modulus::MInt998244353, MInt};
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/point_set_range_composite")]
pub fn point_set_range_composite(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [(MInt998244353, MInt998244353); n]);
    let mut seg = SegmentTree::from_vec(ab, LinearOperation::new());
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, cd: (MInt998244353, MInt998244353));
            seg.set(p, cd);
        } else {
            scan!(scanner, l, r, x: MInt998244353);
            let (a, b) = seg.fold(l, r);
            writeln!(writer, "{}", a * x + b).ok();
        }
    }
}
