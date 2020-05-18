pub use crate::algebra::effect::AnyMonoidEffect;
pub use crate::algebra::operations::{LastOperation, MinOperation};
pub use crate::data_structure::lazy_segment_tree::LazySegmentTree;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/3/DSL/2/DSL_2_D")]
pub fn dsl_2_d(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let mut seg = LazySegmentTree::new(
        n,
        MinOperation::new(),
        AnyMonoidEffect::new(LastOperation::new(), |&x, y| y.unwrap_or(x)),
    );
    for _ in 0..q {
        let ty: usize = scanner.scan();
        if ty == 0 {
            let (s, t, x): (usize, usize, i32) = scanner.scan();
            seg.update(s, t + 1, Some(x));
        } else {
            let i: usize = scanner.scan();
            writeln!(writer, "{}", seg.fold(i, i + 1))?;
        }
    }
    Ok(())
}
