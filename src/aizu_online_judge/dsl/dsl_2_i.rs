pub use crate::algebra::effect::AnyMonoidEffect;
pub use crate::algebra::operations::{AdditiveOperation, CartesianOperation, LastOperation};
pub use crate::data_structure::lazy_segment_tree::LazySegmentTree;
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_I")]
pub fn dsl_2_i(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::from_vec(
        vec![(0, 1); n],
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AnyMonoidEffect::new(LastOperation::new(), |x: &(i64, i64), y| {
            (x.1 * y.unwrap_or(x.0), x.1)
        }),
    );
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, s, t, x: {i64 => Some});
            seg.update(s, t + 1, x);
        } else {
            scan!(scanner, s, t);
            writeln!(writer, "{}", seg.fold(s, t + 1).0)?;
        }
    }
    Ok(())
}
