pub use crate::algebra::effect::AnyMonoidEffect;
pub use crate::algebra::operations::{AdditiveOperation, CartesianOperation, LastOperation};
pub use crate::data_structure::lazy_segment_tree::LazySegmentTree;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_I")]
pub fn dsl_2_i(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let mut seg = LazySegmentTree::from_vec(
        vec![(0, 1); n],
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AnyMonoidEffect::new(LastOperation::new(), |x: &(i64, i64), y| {
            (x.1 * y.unwrap_or(x.0), x.1)
        }),
    );
    for _ in 0..q {
        let ty: usize = scanner.scan();
        if ty == 0 {
            let (s, t, x): (usize, usize, i64) = scanner.scan();
            seg.update(s, t + 1, Some(x));
        } else {
            let (s, t): (usize, usize) = scanner.scan();
            writeln!(writer, "{}", seg.fold(s, t + 1).0)?;
        }
    }
    Ok(())
}
