use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{
    algebra::MinOperation,
    data_structure::{DisjointSparseTable, SegmentTree},
};

#[verify::library_checker("staticrmq")]
pub fn staticrmq_disjoint_sparse_table(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], lr: [(usize, usize)]);
    let table = DisjointSparseTable::<MinOperation<_>>::new(a);
    for (l, r) in lr.take(q) {
        writeln!(writer, "{}", table.fold(l, r)).ok();
    }
}

#[verify::library_checker("staticrmq")]
pub fn staticrmq_segment_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, a: [u64; n], lr: [(usize, usize)]);
    let seg = SegmentTree::<MinOperation<_>>::from_vec(a);
    for (l, r) in lr.take(q) {
        writeln!(writer, "{}", seg.fold(l..r)).ok();
    }
}
