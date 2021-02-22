use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::MinOperation,
    data_structure::{DisjointSparseTable, SegmentTree},
};

#[verify::verify("https://judge.yosupo.jp/problem/staticrmq")]
pub fn staticrmq_disjoint_sparse_table(reader: impl Read, mut writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], lr: [(usize, usize)]);
    let table = DisjointSparseTable::new(a, MinOperation::new());
    for (l, r) in lr.take(q) {
        writeln!(writer, "{}", table.fold(l, r)).ok();
    }
}

#[verify::verify("https://judge.yosupo.jp/problem/staticrmq")]
pub fn staticrmq_segment_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], lr: [(usize, usize)]);
    let seg = SegmentTree::from_vec(a, MinOperation::new());
    for (l, r) in lr.take(q) {
        writeln!(writer, "{}", seg.fold(l, r)).ok();
    }
}
