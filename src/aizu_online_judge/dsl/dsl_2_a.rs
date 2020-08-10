pub use crate::algebra::MinOperation;
pub use crate::data_structure::SegmentTree;
use crate::scan;
use crate::tools::{read_all, Scanner};
use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_A")]
pub fn dsl_2_a(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = SegmentTree::new(n, MinOperation::new());
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            seg.set(x, y as i32);
        } else {
            writeln!(writer, "{}", seg.fold(x, y + 1)).ok();
        }
    }
}
