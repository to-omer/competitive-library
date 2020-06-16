pub use crate::algebra::operations::{AdditiveOperation, CartesianOperation, LinearOperation};
pub use crate::data_structure::lazy_segment_tree::LazySegmentTree;
pub use crate::num::mint::{modulus::Modulo998244353, MInt};
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

type M = MInt<Modulo998244353>;

#[verify_attr::verify("https://judge.yosupo.jp/problem/range_affine_range_sum")]
pub fn range_affine_range_sum(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [M; n]);
    let mut seg = LazySegmentTree::from_vec(
        a.into_iter().map(|x| (x, 1u32)).collect::<_>(),
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        LinearOperation::new(),
        |x, y| (y.0 * x.0 + y.1 * M::new(x.1), x.1),
    );
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, l, r, bc: (M, M));
            seg.update(l, r, bc);
        } else {
            scan!(scanner, l, r);
            writeln!(writer, "{}", seg.fold(l, r).0)?;
        }
    }
    Ok(())
}
