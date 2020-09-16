pub use crate::algebra::{AdditiveOperation, CartesianOperation, LinearOperation};
pub use crate::data_structure::LazySegmentTree;
pub use crate::num::{modulus::MInt998244353, MInt};
use crate::prelude::*;

#[verify_attr::verify("https://judge.yosupo.jp/problem/range_affine_range_sum")]
pub fn range_affine_range_sum(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [MInt998244353]);
    let mut seg = LazySegmentTree::from_vec(
        a.take(n).map(|x| (x, 1u32)).collect::<_>(),
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        LinearOperation::new(),
        |x, y| (y.0 * x.0 + y.1 * MInt998244353::new(x.1), x.1),
    );
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, l, r, bc: (MInt998244353, MInt998244353));
            seg.update(l, r, bc);
        } else {
            scan!(scanner, l, r);
            writeln!(writer, "{}", seg.fold(l, r).0).ok();
        }
    }
}
