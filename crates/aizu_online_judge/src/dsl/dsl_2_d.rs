pub use competitive::algebra::{LastOperation, MinOperation};
pub use competitive::data_structure::LazySegmentTree;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_D")]
pub fn dsl_2_d(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = LazySegmentTree::new(n, MinOperation::new(), LastOperation::new(), |&x, y| {
        y.unwrap_or(x)
    });
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, s, t, x: i32);
            seg.update(s, t + 1, Some(x));
        } else {
            scan!(scanner, i);
            writeln!(writer, "{}", seg.fold(i, i + 1)).ok();
        }
    }
}
