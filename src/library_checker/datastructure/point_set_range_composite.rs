pub use crate::algebra::LinearOperation;
pub use crate::data_structure::SegmentTree;
pub use crate::num::{modulus::Modulo998244353, MInt};
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

type M = MInt<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/point_set_range_composite")]
pub fn point_set_range_composite(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, ab: [(M, M); n]);
    let mut seg = SegmentTree::from_vec(ab, LinearOperation::new());
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, cd: (M, M));
            seg.set(p, cd);
        } else {
            scan!(scanner, l, r, x: M);
            let (a, b) = seg.fold(l, r);
            writeln!(writer, "{}", a * x + b).ok();
        }
    }
}
