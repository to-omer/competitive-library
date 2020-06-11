pub use crate::algebra::operations::{AdditiveOperation, MinOperation};
pub use crate::data_structure::lazy_segment_tree::LazySegmentTree;
pub use crate::scan;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_H")]
pub fn dsl_2_h(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::from_vec(
        vec![0; n],
        MinOperation::new(),
        AdditiveOperation::new(),
        |x: &i64, &y| x + y,
    );
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, s, t, x: i64);
            seg.update(s, t + 1, x);
        } else {
            scan!(scanner, s, t);
            writeln!(writer, "{}", seg.fold(s, t + 1))?;
        }
    }
    Ok(())
}
