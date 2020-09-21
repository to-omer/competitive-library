pub use competitive::algebra::AdditiveOperation;
pub use competitive::data_structure::SegmentTree;
use competitive::prelude::*;

#[verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_B")]
pub fn dsl_2_b(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q);
    let mut seg = SegmentTree::new(n, AdditiveOperation::new());
    for _ in 0..q {
        scan!(scanner, ty, x, y);
        if ty == 0 {
            seg.update(x - 1, y as i32);
        } else {
            writeln!(writer, "{}", seg.fold(x - 1, y)).ok();
        }
    }
}
