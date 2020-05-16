pub use crate::algebra::operations::AdditiveOperation;
pub use crate::data_structure::binary_indexed_tree::BinaryIndexedTree;
pub use crate::data_structure::segment_tree::SegmentTree;
pub use crate::tools::scanner::{read_all, Scanner};
use std::io::{self, Read, Write};

#[verify_attr::verify("https://judge.yosupo.jp/problem/point_add_range_sum")]
pub fn point_add_range_sum_binary_indexed_tree(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> io::Result<()> {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let a: Vec<i64> = scanner.scan_vec(n);
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    for i in 0..n {
        bit.update(i + 1, a[i]);
    }
    for _ in 0..q {
        let ty: usize = scanner.scan();
        if ty == 0 {
            let (p, x): (usize, i64) = scanner.scan();
            bit.update(p + 1, x);
        } else {
            let (l, r): (usize, usize) = scanner.scan();
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
    let n: usize = scanner.scan();
    let q: usize = scanner.scan();
    let a: Vec<i64> = scanner.scan_vec(n);
    let mut seg = SegmentTree::from_vec(a, AdditiveOperation::new());
    for _ in 0..q {
        let ty: usize = scanner.scan();
        if ty == 0 {
            let (p, x): (usize, i64) = scanner.scan();
            seg.update(p, x);
        } else {
            let (l, r): (usize, usize) = scanner.scan();
            writeln!(writer, "{}", seg.fold(l, r))?;
        }
    }
    Ok(())
}
