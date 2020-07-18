pub use crate::algebra::{AdditiveOperation, CartesianOperation};
pub use crate::data_structure::LazySegmentTree;
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_E")]
pub fn dsl_2_e(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::from_vec(
        vec![(0, 1); n],
        CartesianOperation::new(AdditiveOperation::new(), AdditiveOperation::new()),
        AdditiveOperation::new(),
        |x: &(u64, u64), &y| (x.0 + x.1 * y, x.1),
    );
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, s, t, x: u64);
            seg.update(s - 1, t, x);
        } else {
            scan!(scanner, i);
            writeln!(writer, "{}", seg.fold(i - 1, i).0)?;
        }
    }
    Ok(())
}
