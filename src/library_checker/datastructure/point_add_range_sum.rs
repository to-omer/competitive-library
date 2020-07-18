pub use crate::algebra::AdditiveOperation;
pub use crate::data_structure::{BinaryIndexedTree, SegmentTree};
pub use crate::scan;
pub use crate::tools::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/point_add_range_sum")]
pub fn point_add_range_sum_binary_indexed_tree(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    for i in 0..n {
        bit.update(i, a[i]);
    }
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, x: i64);
            bit.update(p, x);
        } else {
            scan!(scanner, l, r);
            writeln!(writer, "{}", bit.fold(l, r))?;
        }
    }
    Ok(())
}

#[verify_attr::verify("https://judge.yosupo.jp/problem/point_add_range_sum")]
pub fn point_add_range_sum_segment_tree(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seg = SegmentTree::from_vec(a, AdditiveOperation::new());
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, x: i64);
            seg.update(p, x);
        } else {
            scan!(scanner, l, r);
            writeln!(writer, "{}", seg.fold(l, r))?;
        }
    }
    Ok(())
}
