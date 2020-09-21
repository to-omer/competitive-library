pub use competitive::algebra::AdditiveOperation;
pub use competitive::data_structure::{BinaryIndexedTree, SegmentTree};
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/point_add_range_sum")]
pub fn point_add_range_sum_binary_indexed_tree(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64]);
    let mut bit = BinaryIndexedTree::new(n, AdditiveOperation::new());
    for (i, a) in a.take(n).enumerate() {
        bit.update(i, a);
    }
    for _ in 0..q {
        scan!(scanner, ty);
        if ty == 0 {
            scan!(scanner, p, x: i64);
            bit.update(p, x);
        } else {
            scan!(scanner, l, r);
            writeln!(writer, "{}", bit.fold(l, r)).ok();
        }
    }
}

#[verify::verify("https://judge.yosupo.jp/problem/point_add_range_sum")]
pub fn point_add_range_sum_segment_tree(reader: &mut impl Read, writer: &mut impl Write) {
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
            writeln!(writer, "{}", seg.fold(l, r)).ok();
        }
    }
}
