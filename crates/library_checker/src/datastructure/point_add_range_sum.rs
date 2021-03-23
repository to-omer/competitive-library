use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::AdditiveOperation,
    data_structure::{BinaryIndexedTree, SegmentTree},
};

#[verify::verify("https://judge.yosupo.jp/problem/point_add_range_sum")]
pub fn point_add_range_sum_binary_indexed_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64]);
    let mut bit = BinaryIndexedTree::<AdditiveOperation<_>>::new(n);
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
pub fn point_add_range_sum_segment_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [i64; n]);
    let mut seg = SegmentTree::<AdditiveOperation<_>>::from_vec(a);
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
